use std::{
    borrow::Cow,
    cell::{LazyCell, UnsafeCell},
    ffi::{CStr, CString, OsString},
    fmt::Debug,
    fs, io,
    num::NonZeroUsize,
    os::unix::ffi::OsStringExt,
    path::{Path, MAIN_SEPARATOR},
    sync::Arc,
    thread,
};

use rustix::{
    fs::{cwd, openat, unlinkat, AtFlags, FileType, Mode, OFlags, RawDir},
    io::Errno,
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
    thread_local! {
        static BUF: UnsafeCell<Vec<u8>> = UnsafeCell::new(Vec::with_capacity(8192));
    }

    BUF.with(|buf| {
        let dir = openat(
            cwd(),
            node.path.as_c_str(),
            OFlags::RDONLY | OFlags::DIRECTORY,
            Mode::empty(),
        )
        .map_io_err(|| format!("Failed to open directory: {:?}", node.path))?;

        let node = LazyCell::new(|| Arc::new(node));
        let mut raw_dir = RawDir::new(&dir, unsafe { &mut *buf.get() }.spare_capacity_mut());
        while let Some(file) = raw_dir.next() {
            const DOT: &CStr = CStr::from_bytes_with_nul(b".\0").ok().unwrap();
            const DOT_DOT: &CStr = CStr::from_bytes_with_nul(b"..\0").ok().unwrap();

            let file = file.map_io_err(|| format!("Failed to read directory: {:?}", node.path))?;
            if file.file_name() == DOT || file.file_name() == DOT_DOT {
                continue;
            }

            if file.file_type() == FileType::Directory {
                tasks
                    .send(task::spawn_blocking({
                        let node = TreeNode {
                            path: {
                                let prefix = node.path.to_bytes();
                                let name = file.file_name().to_bytes_with_nul();

                                let mut path = Vec::with_capacity(prefix.len() + 1 + name.len());
                                path.extend_from_slice(prefix);
                                path.push(MAIN_SEPARATOR as u8);
                                path.extend_from_slice(name);
                                unsafe { CString::from_vec_with_nul_unchecked(path) }
                            },
                            parent: Some(node.clone()),
                        };
                        let tasks = tasks.clone();
                        move || delete_dir(node, &tasks)
                    }))
                    .map_err(|_| Error::Internal)?;
            } else {
                unlinkat(&dir, file.file_name(), AtFlags::empty())
                    .map_io_err(|| format!("Failed to delete file: {file:?}"))?;
            }
        }
        Ok(())
    })?;
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
    // TODO use this to send the done signal when None
    parent: Option<Arc<TreeNode>>,
}

impl Drop for TreeNode {
    fn drop(&mut self) {
        // TODO Send this error over lockness
        unlinkat(cwd(), self.path.as_c_str(), AtFlags::REMOVEDIR)
            .map_io_err(|| format!("Failed to delete directory: {:?}", self.path))
            .unwrap();
    }
}
