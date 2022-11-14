use std::{
    borrow::Cow,
    cell::UnsafeCell,
    ffi::CStr,
    fs, io,
    num::NonZeroUsize,
    os::unix::io::OwnedFd,
    path::Path,
    sync::{
        atomic::{AtomicIsize, Ordering},
        Arc,
    },
    thread,
};

use nix::{
    dents::RawDir,
    errno::Errno,
    fcntl::{open, openat, OFlag},
    file_type::FileType,
    sys::stat::Mode,
    unistd::{unlinkat, UnlinkatFlags},
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
        // TODO see if we should instead wait until we've seen a directory to boot up
        //  the runtime
        let parallelism =
            thread::available_parallelism().unwrap_or(unsafe { NonZeroUsize::new_unchecked(1) });
        let runtime = tokio::runtime::Builder::new_current_thread()
            .max_blocking_threads(parallelism.get())
            .build()
            .map_err(Error::RuntimeCreation)?;

        runtime.block_on(run_deletion_scheduler(self))
    }
}

// TODO add tracing to each method
// TODO add debug logging for each fs op
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
            .map_io_err(|| Cow::Owned(format!("Failed to read metadata for file: {file:?}")))?
            .is_dir();

            if is_dir {
                tasks.push(task::spawn_blocking({
                    let node = TreeNode {
                        file: open(
                            file.as_ref(),
                            OFlag::O_RDONLY | OFlag::O_DIRECTORY,
                            Mode::empty(),
                        )
                        .map_io_err(|| {
                            Cow::Owned(format!("Failed to open directory: {:?}", file.as_ref()))
                        })?,
                        parent: None,
                        remaining_children: AtomicIsize::new(0),
                    };
                    let tx = tx.clone();
                    move || delete_dir(node, &tx)
                }));
            } else {
                fs::remove_file(&file)
                    .map_io_err(|| Cow::Owned(format!("Failed to delete file: {file:?}")))?;
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
    const DOT: &CStr = CStr::from_bytes_with_nul(b".\0").ok().unwrap();
    const DOT_DOT: &CStr = CStr::from_bytes_with_nul(b"..\0").ok().unwrap();

    let mut node = Arc::new(node);

    {
        thread_local! {
            static BUF: UnsafeCell<Vec<u8>> = UnsafeCell::new(Vec::with_capacity(8192));
        }

        let mut children = 0;

        BUF.with(|buf| {
            for file in RawDir::new(&node.file, unsafe { &mut *buf.get() }.spare_capacity_mut()) {
                // TODO add a function to read the file path from /proc or fcntl
                let file = file.map_io_err(|| Cow::Borrowed("Directory read failed"))?;
                if file.name == DOT || file.name == DOT_DOT {
                    continue;
                }

                if file.file_type == FileType::Directory {
                    children += 1;
                    tasks
                        .send(task::spawn_blocking({
                            let node = TreeNode {
                                file: openat(
                                    &node.file,
                                    file.name,
                                    OFlag::O_RDONLY | OFlag::O_DIRECTORY,
                                    Mode::empty(),
                                )
                                .map_io_err(|| {
                                    Cow::Owned(format!("Failed to open directory: {:?}", file.name))
                                })?,
                                parent: Some(node.clone()),
                                remaining_children: AtomicIsize::new(0),
                            };
                            let tasks = tasks.clone();
                            move || delete_dir(node, &tasks)
                        }))
                        .map_err(|_| Error::Internal)?;
                } else {
                    unlinkat(&node.file, file.name, UnlinkatFlags::NoRemoveDir)
                        .map_io_err(|| Cow::Owned(format!("Failed to delete file: {file:?}")))?;
                }
            }
            Ok(())
        })?;

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

    unlinkat(&node.file, DOT, UnlinkatFlags::RemoveDir)
        .map_io_err(|| Cow::Borrowed("Failed to delete directory"))?;

    while let Some(parent) = &node.parent {
        if parent.remaining_children.fetch_sub(1, Ordering::Relaxed) != 1 {
            break;
        }

        node = parent.clone();
        unlinkat(&node.file, DOT, UnlinkatFlags::RemoveDir)
            .map_io_err(|| Cow::Borrowed("Failed to delete directory"))?;
    }

    Ok(())
}

trait IoErr<Out> {
    fn map_io_err(self, f: impl FnOnce() -> Cow<'static, str>) -> Out;
}

impl<T> IoErr<Result<T, Error>> for Result<T, io::Error> {
    fn map_io_err(self, context: impl FnOnce() -> Cow<'static, str>) -> Result<T, Error> {
        self.map_err(|error| Error::Io {
            error,
            context: context(),
        })
    }
}

impl<T> IoErr<Result<T, Error>> for Result<T, Errno> {
    fn map_io_err(self, context: impl FnOnce() -> Cow<'static, str>) -> Result<T, Error> {
        self.map_err(io::Error::from).map_io_err(context)
    }
}

pub struct TreeNode {
    pub file: OwnedFd,
    // TODO manually implement this Arc with our remaining_children atomic
    pub parent: Option<Arc<TreeNode>>,
    pub remaining_children: AtomicIsize,
}
