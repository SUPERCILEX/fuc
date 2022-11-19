use std::{
    borrow::Cow,
    cell::UnsafeCell,
    ffi::{CStr, CString, OsString},
    fmt::Debug,
    fs, io, mem,
    num::NonZeroUsize,
    os::unix::{
        ffi::OsStringExt,
        io::{AsRawFd, FromRawFd, OwnedFd},
    },
    path::Path,
    ptr::NonNull,
    sync::atomic::{AtomicIsize, Ordering},
    thread,
};

use nix::{
    dents::RawDir,
    errno::Errno,
    fcntl::{open, openat, OFlag},
    file_type::FileType,
    sys::stat::Mode,
    unistd::{unlinkat, UnlinkatFlags},
    AT_FDCWD,
};
use sync::mpsc;
use tokio::{sync, sync::mpsc::UnboundedSender, task, task::JoinHandle};
use typed_builder::TypedBuilder;

use crate::{ops::remove::tree::TreeNode, Error};

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
            thread::available_parallelism().unwrap_or(const { NonZeroUsize::new(1).unwrap() });
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
    // TODO use futex in lockness mpsc::latest()
    let (tx, mut rx) = mpsc::unbounded_channel();

    {
        // TODO size hint
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
                        file: open(
                            file.as_ref(),
                            OFlag::O_RDONLY | OFlag::O_DIRECTORY,
                            Mode::empty(),
                        )
                        .map_io_err(|| format!("Failed to open directory: {:?}", file.as_ref()))?,
                        file_name: CString::new(OsString::from(file.into_owned()).into_vec())
                            .unwrap(),
                        parent: Some(
                            Box::leak(Box::new(TreeNode {
                                // SAFETY: We make sure to not drop this in delete_dir if parent is
                                // None.
                                file: unsafe { OwnedFd::from_raw_fd(AT_FDCWD.as_raw_fd()) },
                                file_name: CString::default(),
                                parent: None,
                                remaining_children: AtomicIsize::new(1),
                            }))
                            .into(),
                        ),
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
    // TODO don't always allocate this.
    // TODO fix memory leaks on errors
    let node = Box::leak(Box::new(node));

    {
        thread_local! {
            static BUF: UnsafeCell<Vec<u8>> = UnsafeCell::new(Vec::with_capacity(8192));
        }

        let mut children = 0;

        BUF.with(|buf| {
            let parent = Some(node.into());
            for file in RawDir::new(&node.file, unsafe { &mut *buf.get() }.spare_capacity_mut()) {
                const DOT: &CStr = CStr::from_bytes_with_nul(b".\0").ok().unwrap();
                const DOT_DOT: &CStr = CStr::from_bytes_with_nul(b"..\0").ok().unwrap();

                let file =
                    file.map_io_err(|| format!("Failed to read directory: {:?}", node.file_name))?;
                if file.name == DOT || file.name == DOT_DOT {
                    continue;
                }

                if file.file_type == FileType::Directory {
                    // TODO fix the error handling with respect to children not getting updated
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
                                    format!("Failed to open directory: {:?}", node.file_name)
                                })?,
                                file_name: file.name.to_owned(),
                                parent,
                                remaining_children: AtomicIsize::new(0),
                            };
                            let tasks = tasks.clone();
                            move || delete_dir(node, &tasks)
                        }))
                        .map_err(|_| Error::Internal)?;
                } else {
                    unlinkat(&node.file, file.name, UnlinkatFlags::NoRemoveDir)
                        .map_io_err(|| format!("Failed to delete file: {file:?}"))?;
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

    'done: {
        unsafe {
            let mut next = NonNull::from(node);
            while let Some(parent) = next.as_ref().parent {
                unlinkat(
                    &parent.as_ref().file,
                    next.as_ref().file_name.as_c_str(),
                    UnlinkatFlags::RemoveDir,
                )
                .map_io_err(|| {
                    format!("Failed to delete directory: {:?}", next.as_ref().file_name)
                })?;

                // We must be the last user of this allocation b/c:
                // - If we came from outside the loop, we would have exited above in the
                //   children check.
                // - If we're coming from inside the loop, the remaining_children check operates
                //   on the parent and would block the next iteration.
                drop(Box::from_raw(next.as_ptr()));

                // TODO using Relaxed here is almost certainly wrong. Do some more research and
                //  figure out the correct ordering.
                if parent
                    .as_ref()
                    .remaining_children
                    .fetch_sub(1, Ordering::Relaxed)
                    != 1
                {
                    // There are still active children, let the last of them do the cleanup.
                    break 'done;
                }
                next = parent;
            }

            let root = Box::from_raw(next.as_ptr());
            // See comment in run_deletion_scheduler, this isn't a real fd so don't drop it.
            debug_assert_eq!(root.file.as_raw_fd(), AT_FDCWD.as_raw_fd());
            mem::forget(root.file);
        }
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

impl<T> IoErr<Result<T, Error>> for Result<T, Errno> {
    fn map_io_err(self, context: impl FnOnce() -> String) -> Result<T, Error> {
        self.map_err(io::Error::from).map_io_err(context)
    }
}

mod tree {
    use std::{ffi::CString, os::unix::io::OwnedFd, ptr::NonNull, sync::atomic::AtomicIsize};

    pub struct TreeNode {
        // TODO get rid of this or we can run out of file descriptors
        pub file: OwnedFd,
        pub file_name: CString,
        pub parent: Option<NonNull<TreeNode>>,
        pub remaining_children: AtomicIsize,
    }

    unsafe impl Send for TreeNode {}
}
