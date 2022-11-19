use std::{
    borrow::Cow,
    cell::UnsafeCell,
    ffi::{CStr, CString, OsString},
    fmt::Debug,
    fs, io,
    mem::ManuallyDrop,
    num::NonZeroUsize,
    os::unix::ffi::OsStringExt,
    path::{Path, MAIN_SEPARATOR},
    ptr::NonNull,
    sync::atomic::{AtomicIsize, Ordering},
    thread,
};

use nix::{
    dents::RawDir,
    errno::Errno,
    fcntl::{open, OFlag},
    file_type::FileType,
    sys::stat::Mode,
    unistd::{unlinkat, UnlinkatFlags},
    AT_FDCWD,
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
                        path: CString::new(OsString::from(file.into_owned()).into_vec()).unwrap(),
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
    // TODO don't always allocate this.
    // TODO fix memory leaks on errors
    let node = Box::leak(Box::new(node));

    {
        thread_local! {
            static BUF: UnsafeCell<Vec<u8>> = UnsafeCell::new(Vec::with_capacity(8192));
        }

        let mut children = 0;

        BUF.with(|buf| {
            let dir = open(
                node.path.as_c_str(),
                OFlag::O_RDONLY | OFlag::O_DIRECTORY,
                Mode::empty(),
            )
            .map_io_err(|| format!("Failed to open directory: {:?}", node.path))?;

            for file in RawDir::new(&dir, unsafe { &mut *buf.get() }.spare_capacity_mut()) {
                const DOT: &CStr = CStr::from_bytes_with_nul(b".\0").ok().unwrap();
                const DOT_DOT: &CStr = CStr::from_bytes_with_nul(b"..\0").ok().unwrap();

                let file =
                    file.map_io_err(|| format!("Failed to read directory: {:?}", node.path))?;
                if file.name == DOT || file.name == DOT_DOT {
                    continue;
                }

                if file.file_type == FileType::Directory {
                    // TODO fix the error handling with respect to children not getting updated
                    children += 1;
                    tasks
                        .send(task::spawn_blocking({
                            let node = TreeNode {
                                path: {
                                    let prefix = node.path.to_bytes();
                                    let name = file.name.to_bytes_with_nul();

                                    let mut path =
                                        Vec::with_capacity(prefix.len() + 1 + name.len());
                                    path.extend_from_slice(prefix);
                                    path.push(MAIN_SEPARATOR as u8);
                                    path.extend_from_slice(name);
                                    unsafe { CString::from_vec_with_nul_unchecked(path) }
                                },
                                parent: Some(node.into()),
                                remaining_children: AtomicIsize::new(0),
                            };
                            let tasks = tasks.clone();
                            move || delete_dir(node, &tasks)
                        }))
                        .map_err(|_| Error::Internal)?;
                } else {
                    unlinkat(&dir, file.name, UnlinkatFlags::NoRemoveDir)
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

    let mut next = Some(NonNull::from(node));
    while let Some(node) = next {
        let node = ManuallyDrop::new(unsafe { Box::from_raw(node.as_ptr()) });

        next = node.parent;
        unlinkat(AT_FDCWD, node.path.as_c_str(), UnlinkatFlags::RemoveDir)
            .map_io_err(|| format!("Failed to delete directory: {:?}", node.path))?;

        // We must be the last user of this allocation b/c:
        // - If we came from outside the loop, we would have exited above in the
        //   children check.
        // - If we're coming from inside the loop, the remaining_children check operates
        //   on the parent and would block the next iteration.
        ManuallyDrop::into_inner(node);

        if let Some(parent) = next {
            // TODO using Relaxed here is almost certainly wrong. Do some more research and
            //  figure out the correct ordering.
            if unsafe { parent.as_ref() }
                .remaining_children
                .fetch_sub(1, Ordering::Relaxed)
                != 1
            {
                // There are still active children, let the last of them do the cleanup.
                break;
            }
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

struct TreeNode {
    path: CString,
    parent: Option<NonNull<TreeNode>>,
    remaining_children: AtomicIsize,
}

unsafe impl Send for TreeNode {}
