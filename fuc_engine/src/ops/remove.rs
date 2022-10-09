use std::{
    fs,
    num::NonZeroUsize,
    path::{Path, PathBuf},
    thread,
};

use sync::mpsc;
use tokio::{sync, sync::mpsc::UnboundedSender, task, task::JoinHandle};
use typed_builder::TypedBuilder;

use crate::{Error, FsOp};

#[derive(TypedBuilder, Debug)]
pub struct RemoveOp<'a, F: IntoIterator<Item = &'a Path>> {
    // TODO make this a variant that's either owned or not. Maybe Cow?
    files: F,
}

impl<'a, F: IntoIterator<Item = &'a Path>> FsOp for RemoveOp<'a, F> {
    fn run(self) -> Result<(), Error> {
        let parallelism =
            thread::available_parallelism().unwrap_or(unsafe { NonZeroUsize::new_unchecked(1) });
        let runtime = tokio::runtime::Builder::new_current_thread()
            .max_blocking_threads(parallelism.get())
            .build()
            .map_err(Error::RuntimeCreation)?;

        runtime.block_on(run_deletion_scheduler(self))
    }
}

async fn run_deletion_scheduler<'a, F: IntoIterator<Item = &'a Path>>(
    op: RemoveOp<'a, F>,
) -> Result<(), Error> {
    let mut dirs = Vec::new();
    {
        let (tx, mut rx) = mpsc::unbounded_channel();

        {
            let mut tasks = Vec::new();
            for file in op.files {
                let is_dir = file
                    .metadata()
                    .map_err(|error| Error::Io {
                        error,
                        context: format!("Failed to read metadata for file: {file:?}"),
                    })?
                    .is_dir();

                if is_dir {
                    tasks.push(task::spawn_blocking({
                        let dir = file.to_path_buf();
                        let tx = tx.clone();
                        move || delete_dir(dir, tx)
                    }));
                } else {
                    remove_file(file)?;
                }
            }
            drop(tx);

            for task in tasks {
                dirs.push(task.await.map_err(Error::TaskJoin)??);
            }
        }

        while let Some(task) = rx.recv().await {
            dirs.push(task.await.map_err(Error::TaskJoin)??);
        }
    }

    for dir in dirs.into_iter().rev().flatten() {
        remove_dir(dir)?;
    }

    Ok(())
}

fn delete_dir(
    dir: PathBuf,
    tasks: UnboundedSender<JoinHandle<Result<Option<PathBuf>, Error>>>,
) -> Result<Option<PathBuf>, Error> {
    let mut has_children = false;

    // TODO use getdents64 on linux
    let files = fs::read_dir(&dir).map_err(|error| Error::Io {
        error,
        context: format!("Failed to read dir: {dir:?}"),
    })?;
    for file in files {
        let file = file.map_err(|error| Error::Io {
            error,
            context: format!("DirEntry fetch failed for dir: {dir:?}"),
        })?;
        let is_dir = file
            .file_type()
            .map_err(|error| Error::Io {
                error,
                context: format!("Failed to read metadata for file: {file:?}"),
            })?
            .is_dir();

        has_children |= is_dir;
        if is_dir {
            tasks
                .send(task::spawn_blocking({
                    let dir = file.path();
                    let tasks = tasks.clone();
                    move || delete_dir(dir, tasks)
                }))
                .map_err(|_| Error::Internal)?;
        } else {
            remove_file(file.path())?;
        }
    }

    if has_children {
        Ok(Some(dir))
    } else {
        remove_dir(dir)?;
        Ok(None)
    }
}

fn remove_file(file: impl AsRef<Path>) -> Result<(), Error> {
    let file = file.as_ref();
    fs::remove_file(file).map_err(|error| Error::Io {
        error,
        context: format!("Failed to delete file: {file:?}"),
    })
}

fn remove_dir(dir: impl AsRef<Path>) -> Result<(), Error> {
    let dir = dir.as_ref();
    fs::remove_dir(dir).map_err(|error| Error::Io {
        error,
        context: format!("Failed to delete directory: {dir:?}"),
    })
}
