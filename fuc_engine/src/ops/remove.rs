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

use lockness::RawOneLatest;
use rustix::{
    fs::{cwd, openat, unlinkat, AtFlags, FileType, Mode, OFlags, RawDir},
    io::Errno,
};
use tokio::task;
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
        let mut files = self.files.into_iter();
        for file in &mut files {
            let is_dir = match file.metadata() {
                Err(e) if self.force && e.kind() == io::ErrorKind::NotFound => {
                    continue;
                }
                r => r,
            }
            .map_io_err(|| format!("Failed to read metadata for file: {file:?}"))?
            .is_dir();

            if is_dir {
                let parallelism = thread::available_parallelism()
                    .unwrap_or(const { NonZeroUsize::new(1).unwrap() });
                let runtime = tokio::runtime::Builder::new_current_thread()
                    .max_blocking_threads(parallelism.get())
                    .build()
                    .map_err(Error::RuntimeCreation)?;

                return runtime.block_on(async {
                    run_deletion_scheduler(
                        file,
                        RemoveOp {
                            files,
                            force: self.force,
                            preserve_root: self.preserve_root,
                        },
                    )
                });
            }

            fs::remove_file(&file).map_io_err(|| format!("Failed to delete file: {file:?}"))?;
        }
        Ok(())
    }
}

// TODO add tracing to each method
// TODO add debug logging for each fs op
fn run_deletion_scheduler<'a, F: IntoIterator<Item = Cow<'a, Path>>>(
    pending_dir: Cow<'a, Path>,
    RemoveOp {
        files,
        force,
        preserve_root,
    }: RemoveOp<'a, F>,
) -> Result<(), Error> {
    let error_queue = Arc::new(RawOneLatest::default());
    let mut dirs = 0;

    let mut spawn_dir_deletion = |file: Cow<Path>| {
        dirs += 1;

        let node = TreeNode {
            path: CString::new(OsString::from(file.into_owned()).into_vec()).unwrap(),
            parent: None,
            error_queue: error_queue.clone(),
        };
        task::spawn_blocking(move || delete_dir(node))
    };

    spawn_dir_deletion(pending_dir);

    for file in files {
        if preserve_root && file == Path::new("/") {
            return Err(Error::PreserveRoot);
        }
        let is_dir = match file.metadata() {
            Err(e) if force && e.kind() == io::ErrorKind::NotFound => {
                continue;
            }
            r => r,
        }
        .map_io_err(|| format!("Failed to read metadata for file: {file:?}"))?
        .is_dir();

        if is_dir {
            spawn_dir_deletion(file);
        } else {
            fs::remove_file(&file).map_io_err(|| format!("Failed to delete file: {file:?}"))?;
        }
    }

    while dirs > 0 {
        error_queue.pop()?;
        dirs -= 1;
    }
    Ok(())
}

fn delete_dir(node: TreeNode) {
    let error_queue = node.error_queue.clone();
    if let e @ Err(_) = delete_dir_internal(node) {
        error_queue.push(e);
    }
}

fn delete_dir_internal(node: TreeNode) -> Result<(), Error> {
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
                task::spawn_blocking({
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
                        error_queue: node.error_queue.clone(),
                    };
                    move || delete_dir(node)
                });
            } else {
                unlinkat(&dir, file.file_name(), AtFlags::empty())
                    .map_io_err(|| format!("Failed to delete file: {file:?}"))?;
            }
        }
        Ok(())
    })
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
    parent: Option<Arc<TreeNode>>,
    error_queue: Arc<RawOneLatest<Result<(), Error>>>,
}

impl Drop for TreeNode {
    fn drop(&mut self) {
        if let e @ Err(_) = unlinkat(cwd(), self.path.as_c_str(), AtFlags::REMOVEDIR)
            .map_io_err(|| format!("Failed to delete directory: {:?}", self.path))
        {
            self.error_queue.push(e);
        } else if self.parent.is_none() {
            self.error_queue.push(Ok(()));
        }
    }
}
