use std::{
    borrow::Cow,
    fs, io,
    num::NonZeroUsize,
    path::{Path, PathBuf},
    thread,
};

use sync::mpsc;
use tokio::{sync, sync::mpsc::UnboundedSender, task, task::JoinHandle};
use typed_builder::TypedBuilder;

use crate::Error;

/// Removes a directory at this path, after removing all its contents.
///
/// This function does **not** follow symbolic links and it will simply remove
/// the symbolic link itself.
///
/// > Note: This function currently starts its own tokio runtime.
///
/// # Errors
///
/// Returns the underlying I/O errors that occurred.
pub fn remove_dir_all<P: AsRef<Path>>(path: P) -> Result<(), Error> {
    RemoveOp::builder()
        .files([Cow::Borrowed(path.as_ref())])
        .build()
        .run()
}

#[derive(TypedBuilder, Debug)]
pub struct RemoveOp<'a, F: IntoIterator<Item = Cow<'a, Path>>> {
    files: F,
    #[builder(default = false)]
    force: bool,
    #[builder(default = true)]
    preserve_root: bool,
}

impl<'a, F: IntoIterator<Item = Cow<'a, Path>>> RemoveOp<'a, F> {
    /// Consume and run this remove operation.
    ///
    /// # Errors
    ///
    /// Returns the underlying I/O errors that occurred.
    pub fn run(self) -> Result<(), Error> {
        let parallelism =
            thread::available_parallelism().unwrap_or(unsafe { NonZeroUsize::new_unchecked(1) });
        let runtime = tokio::runtime::Builder::new_current_thread()
            .max_blocking_threads(parallelism.get())
            .build()
            .map_err(Error::RuntimeCreation)?;

        runtime.block_on(run_deletion_scheduler(self))
    }
}

async fn run_deletion_scheduler<'a, F: IntoIterator<Item = Cow<'a, Path>>>(
    op: RemoveOp<'a, F>,
) -> Result<(), Error> {
    let mut dirs = Vec::new();
    {
        let (tx, mut rx) = mpsc::unbounded_channel();

        {
            let mut tasks = Vec::new();
            for file in op.files {
                if op.preserve_root && file == Path::new("/") {
                    return Err(Error::PreserveRoot);
                }
                let is_dir = match file.metadata() {
                    Err(e) if op.force && e.kind() == io::ErrorKind::NotFound => {
                        continue;
                    }
                    r => r,
                }
                .map_io_err(|| format!("Failed to read metadata for file: {file:?}"))?
                .is_dir();

                if is_dir {
                    tasks.push(task::spawn_blocking({
                        let dir = file.into_owned();
                        let tx = tx.clone();
                        move || delete_dir(dir, &tx)
                    }));
                } else {
                    fs::remove_file(&file)
                        .map_io_err(|| format!("Failed to delete file: {file:?}"))?;
                }
            }
            drop(tx);

            for task in tasks {
                if let Some(dir) = task.await.map_err(Error::TaskJoin)?? {
                    dirs.push(dir);
                }
            }
        }

        while let Some(task) = rx.recv().await {
            if let Some(dir) = task.await.map_err(Error::TaskJoin)?? {
                dirs.push(dir);
            }
        }
    }

    // TODO get rid of this garbage and use a tree with parallel deletions
    while let Some(dir) = dirs.pop() {
        match fs::remove_dir_all(&dir) {
            Err(e) if e.kind() == io::ErrorKind::NotFound => {
                continue;
            }
            r => r,
        }
        .map_io_err(|| format!("Failed to delete directory: {dir:?}"))?;
    }

    Ok(())
}

fn delete_dir(
    dir: PathBuf,
    tasks: &UnboundedSender<JoinHandle<Result<Option<PathBuf>, Error>>>,
) -> Result<Option<PathBuf>, Error> {
    let mut has_children = false;

    // TODO use getdents64 on linux
    let files = fs::read_dir(&dir).map_io_err(|| format!("Failed to read directory: {dir:?}"))?;
    for file in files {
        let file = file.map_io_err(|| format!("DirEntry fetch failed for directory: {dir:?}"))?;
        let is_dir = file
            .file_type()
            .map_io_err(|| format!("Failed to read metadata for file: {file:?}"))?
            .is_dir();

        has_children |= is_dir;
        if is_dir {
            tasks
                .send(task::spawn_blocking({
                    let dir = file.path();
                    let tasks = tasks.clone();
                    move || delete_dir(dir, &tasks)
                }))
                .map_err(|_| Error::Internal)?;
        } else {
            let file = file.path();
            fs::remove_file(&file).map_io_err(|| format!("Failed to delete file: {file:?}"))?;
        }
    }

    if has_children {
        Ok(Some(dir))
    } else {
        fs::remove_dir(&dir).map_io_err(|| format!("Failed to delete directory: {dir:?}"))?;
        Ok(None)
    }
}

trait IoErr<Out> {
    fn map_io_err(self, f: impl FnOnce() -> String) -> Out;
}

impl<T> IoErr<Result<T, Error>> for Result<T, io::Error> {
    fn map_io_err(self, context: impl FnOnce() -> String) -> Result<T, Error> {
        self.map_err(|error| Error::Io {
            error,
            context: context(),
        })
    }
}
