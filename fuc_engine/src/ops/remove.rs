use std::{
    borrow::Cow,
    fs, io,
    num::NonZeroUsize,
    path::Path,
    sync::atomic::{AtomicIsize, Ordering},
    thread,
};

use sync::mpsc;
use tokio::{sync, sync::mpsc::UnboundedSender, task, task::JoinHandle};
use typed_builder::TypedBuilder;

use tree::{Arcable, TreeNode};

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
                    let node = TreeNode {
                        path: file.into_owned(),
                        parent: None,
                        remaining_children: AtomicIsize::new(0),
                    };
                    let tx = tx.clone();
                    move || delete_dir(node, &tx)
                }));
            } else {
                fs::remove_file(&file).map_io_err(|| format!("Failed to delete file: {file:?}"))?;
            }
        }
        drop(tx);

        for task in tasks {
            task.await.map_err(Error::TaskJoin)??;
        }
    }

    while let Some(task) = rx.recv().await {
        task.await.map_err(Error::TaskJoin)??;
    }

    Ok(())
}

fn delete_dir(
    node: TreeNode,
    tasks: &UnboundedSender<JoinHandle<Result<(), Error>>>,
) -> Result<(), Error> {
    let mut node = Arcable::new(node);

    {
        let mut children = 0;

        // TODO use getdents64 on linux
        let files = fs::read_dir(&node.path)
            .map_io_err(|| format!("Failed to read directory: {:?}", node.path))?;
        for file in files {
            let file = file
                .map_io_err(|| format!("DirEntry fetch failed for directory: {:?}", node.path))?;
            let is_dir = file
                .file_type()
                .map_io_err(|| format!("Failed to read metadata for file: {file:?}"))?
                .is_dir();

            if is_dir {
                children += 1;
                tasks
                    .send(task::spawn_blocking({
                        let node = TreeNode {
                            path: file.path(),
                            parent: Some(node.arc()),
                            remaining_children: AtomicIsize::new(0),
                        };
                        let tasks = tasks.clone();
                        move || delete_dir(node, &tasks)
                    }))
                    .map_err(|_| Error::Internal)?;
            } else {
                let file = file.path();
                fs::remove_file(&file).map_io_err(|| format!("Failed to delete file: {file:?}"))?;
            }
        }

        if children > 0 {
            children = children
                + node
                    .remaining_children
                    .fetch_add(children, Ordering::Relaxed);
            debug_assert!(children >= 0, "Deleted more directories than we have!");
            if children > 0 {
                return Ok(());
            }
        }
    }

    fs::remove_dir(&node.path)
        .map_io_err(|| format!("Failed to delete directory: {:?}", node.path))?;

    while let Some(parent) = &node.parent {
        if parent.remaining_children.fetch_sub(1, Ordering::Relaxed) != 1 {
            break;
        }

        node = parent.clone().into();
        fs::remove_dir(&node.path)
            .map_io_err(|| format!("Failed to delete directory: {:?}", node.path))?;
    }

    Ok(())
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

mod tree {
    use std::{
        hint::unreachable_unchecked,
        mem,
        ops::Deref,
        path::PathBuf,
        sync::{atomic::AtomicIsize, Arc},
    };

    #[derive(Default)]
    pub struct TreeNode {
        pub path: PathBuf,
        pub parent: Option<Arc<TreeNode>>,
        pub remaining_children: AtomicIsize,
    }

    pub struct Arcable<T> {
        t: Result<T, Arc<T>>,
    }

    impl<T: Default> Arcable<T> {
        pub const fn new(t: T) -> Self {
            Self { t: Ok(t) }
        }

        pub fn arc(&mut self) -> Arc<T> {
            if self.t.is_ok() {
                let t = mem::replace(&mut self.t, Ok(T::default()));
                let t = unsafe { t.unwrap_unchecked() };
                drop(mem::replace(&mut self.t, Err(Arc::new(t))));
            }

            match &self.t {
                Err(arc) => arc.clone(),
                Ok(_) => unsafe { unreachable_unchecked() },
            }
        }
    }

    impl<T> Deref for Arcable<T> {
        type Target = T;

        fn deref(&self) -> &Self::Target {
            match &self.t {
                Ok(t) => t,
                Err(arc) => arc,
            }
        }
    }

    impl<T> From<Arc<T>> for Arcable<T> {
        fn from(arc: Arc<T>) -> Self {
            Self { t: Err(arc) }
        }
    }
}
