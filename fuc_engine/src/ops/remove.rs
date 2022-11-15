use std::{
    borrow::Cow,
    cell::UnsafeCell,
    ffi::{CStr, CString, OsString},
    fmt::Debug,
    fs, io,
    num::NonZeroUsize,
    os::unix::ffi::OsStringExt,
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
    AT_FDCWD,
};
use sync::mpsc;
use tokio::{sync, sync::mpsc::UnboundedSender, task, task::JoinHandle};
use typed_builder::TypedBuilder;

use crate::{
    ops::remove::tree::{OwnedOrBorrowedFd, TreeNode},
    Error,
};

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
            .map_io_err(|| format!("Failed to read metadata for file: {file:?}"))?
            .is_dir();

            if is_dir {
                tasks.push(task::spawn_blocking({
                    let node = TreeNode {
                        file: OwnedOrBorrowedFd::Owned(
                            open(
                                file.as_ref(),
                                OFlag::O_RDONLY | OFlag::O_DIRECTORY,
                                Mode::empty(),
                            )
                            .map_io_err(|| {
                                format!("Failed to open directory: {:?}", file.as_ref())
                            })?,
                        ),
                        file_name: CString::new(OsString::from(file.into_owned()).into_vec())
                            .unwrap(),
                        parent: Some(Arc::new(TreeNode {
                            file: OwnedOrBorrowedFd::Borrowed(*AT_FDCWD),
                            file_name: CString::default(),
                            parent: None,
                            remaining_children: AtomicIsize::new(1),
                        })),
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
    let mut node = Arc::new(node);

    {
        thread_local! {
            static BUF: UnsafeCell<Vec<u8>> = UnsafeCell::new(Vec::with_capacity(8192));
        }

        let mut children = 0;

        BUF.with(|buf| {
            for file in RawDir::new(&node.file, unsafe { &mut *buf.get() }.spare_capacity_mut()) {
                const DOT: &CStr = CStr::from_bytes_with_nul(b".\0").ok().unwrap();
                const DOT_DOT: &CStr = CStr::from_bytes_with_nul(b"..\0").ok().unwrap();

                let file =
                    file.map_io_err(|| format!("Failed to read directory: {:?}", node.file_name))?;
                if file.name == DOT || file.name == DOT_DOT {
                    continue;
                }

                if file.file_type == FileType::Directory {
                    children += 1;
                    tasks
                        .send(task::spawn_blocking({
                            let node = TreeNode {
                                file: OwnedOrBorrowedFd::Owned(
                                    openat(
                                        &node.file,
                                        file.name,
                                        OFlag::O_RDONLY | OFlag::O_DIRECTORY,
                                        Mode::empty(),
                                    )
                                    .map_io_err(|| {
                                        format!("Failed to open directory: {:?}", node.file_name)
                                    })?,
                                ),
                                file_name: file.name.to_owned(),
                                parent: Some(node.clone()),
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

    while let Some(parent) = &node.parent {
        unlinkat(
            &parent.file,
            node.file_name.as_c_str(),
            UnlinkatFlags::RemoveDir,
        )
        .map_io_err(|| format!("Failed to delete directory: {:?}", node.file_name))?;

        if parent.remaining_children.fetch_sub(1, Ordering::Relaxed) != 1 {
            break;
        }
        node = parent.clone();
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
    use std::{
        ffi::CString,
        os::unix::io::{AsFd, BorrowedFd, OwnedFd},
        sync::{atomic::AtomicIsize, Arc},
    };

    pub struct TreeNode {
        pub file: OwnedOrBorrowedFd,
        pub file_name: CString,
        // TODO manually implement this Arc with our remaining_children atomic
        pub parent: Option<Arc<TreeNode>>,
        pub remaining_children: AtomicIsize,
    }

    pub enum OwnedOrBorrowedFd {
        Owned(OwnedFd),
        Borrowed(BorrowedFd<'static>),
    }

    impl AsFd for OwnedOrBorrowedFd {
        fn as_fd(&self) -> BorrowedFd<'_> {
            match self {
                Self::Owned(o) => o.as_fd(),
                Self::Borrowed(b) => b.as_fd(),
            }
        }
    }
}
