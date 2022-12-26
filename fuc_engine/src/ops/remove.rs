use std::{
    borrow::Cow,
    cell::RefCell,
    ffi::{CStr, CString, OsString},
    fmt::Debug,
    fs, io,
    num::NonZeroUsize,
    os::unix::ffi::OsStringExt,
    path::{Path, MAIN_SEPARATOR},
    sync::Arc,
    thread,
};

use crossbeam_channel::{Receiver, Sender};
use rustix::{
    fs::{cwd, openat, unlinkat, AtFlags, FileType, Mode, OFlags, RawDir},
    io::Errno,
};
use typed_builder::TypedBuilder;

use crate::Error;

/// Removes a file or directory at this path, after removing all its contents.
///
/// This function does **not** follow symbolic links: it will simply remove
/// the symbolic link itself.
///
/// # Errors
///
/// Returns the underlying I/O errors that occurred.
pub fn remove_file<P: AsRef<Path>>(path: P) -> Result<(), Error> {
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
        let scheduling = LazyCell::new(|| {
            let (tx, rx) = crossbeam_channel::unbounded();
            (tx, thread::spawn(|| root_worker_thread(rx)))
        });

        let result = schedule_deletions(self, &scheduling);

        if let Some((tasks, thread)) = scheduling.into_inner() {
            drop(tasks);
            thread.join().map_err(|_| Error::Join)??;
        }

        result
    }
}

fn schedule_deletions<'a, L>(
    RemoveOp {
        files,
        force,
        preserve_root,
    }: RemoveOp<'a, impl IntoIterator<Item = Cow<'a, Path>>>,
    scheduling: &LazyCell<(Sender<Message>, L), impl FnOnce() -> (Sender<Message>, L)>,
) -> Result<(), Error> {
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
            let (tasks, _) = &**scheduling;
            drop(
                tasks.send(Message::Node(TreeNode {
                    path: CString::new(OsString::from(file.into_owned()).into_vec())
                        .map_err(|_| Error::BadPath)?,
                    _parent: None,
                    messages: tasks.clone(),
                })),
            );
        } else {
            fs::remove_file(&file).map_io_err(|| format!("Failed to delete file: {file:?}"))?;
        }
    }
    Ok(())
}

#[allow(clippy::needless_pass_by_value)]
fn root_worker_thread(tasks: Receiver<Message>) -> Result<(), Error> {
    let mut available_parallelism = thread::available_parallelism()
        .map(NonZeroUsize::get)
        .unwrap_or(1)
        - 1;

    thread::scope(|scope| {
        let mut threads = Vec::with_capacity(available_parallelism);

        for node in &tasks {
            if available_parallelism > 0 {
                available_parallelism -= 1;
                threads.push(scope.spawn({
                    let tasks = tasks.clone();
                    || worker_thread(tasks)
                }));
            }

            match node {
                Message::Node(node) => delete_dir(node)?,
                Message::Error(e) => return Err(e),
            }
        }

        for thread in threads {
            thread.join().map_err(|_| Error::Join)??;
        }
        Ok(())
    })
}

fn worker_thread(tasks: Receiver<Message>) -> Result<(), Error> {
    for node in tasks {
        match node {
            Message::Node(node) => delete_dir(node)?,
            Message::Error(e) => return Err(e),
        }
    }
    Ok(())
}

fn delete_dir(node: TreeNode) -> Result<(), Error> {
    thread_local! {
        static BUF: RefCell<Vec<u8>> = RefCell::new(Vec::with_capacity(8192));
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
        let mut buf = buf.borrow_mut();
        let mut raw_dir = RawDir::new(&dir, buf.spare_capacity_mut());
        while let Some(file) = raw_dir.next() {
            const DOT: &CStr = CStr::from_bytes_with_nul(b".\0").ok().unwrap();
            const DOT_DOT: &CStr = CStr::from_bytes_with_nul(b"..\0").ok().unwrap();

            let file = file.map_io_err(|| format!("Failed to read directory: {:?}", node.path))?;
            if file.file_name() == DOT || file.file_name() == DOT_DOT {
                continue;
            }

            if file.file_type() == FileType::Directory {
                node.messages
                    .send(Message::Node(TreeNode {
                        path: {
                            let prefix = node.path.as_bytes();
                            let name = file.file_name().to_bytes_with_nul();

                            let mut path = Vec::with_capacity(prefix.len() + 1 + name.len());
                            path.extend_from_slice(prefix);
                            path.push(u8::try_from(MAIN_SEPARATOR).unwrap());
                            path.extend_from_slice(name);
                            unsafe { CString::from_vec_with_nul_unchecked(path) }
                        },
                        _parent: Some(node.clone()),
                        messages: node.messages.clone(),
                    }))
                    .map_err(|_| Error::Internal)?;
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

enum Message {
    Node(TreeNode),
    Error(Error),
}

struct TreeNode {
    path: CString,
    // Needed for the recursive drop implementation
    _parent: Option<Arc<TreeNode>>,
    messages: Sender<Message>,
}

impl Drop for TreeNode {
    fn drop(&mut self) {
        if let Err(e) = unlinkat(cwd(), self.path.as_c_str(), AtFlags::REMOVEDIR)
            .map_io_err(|| format!("Failed to delete directory: {:?}", self.path))
        {
            // If the receiver closed, then another error must have already occurred.
            drop(self.messages.send(Message::Error(e)));
        }
    }
}

// TODO remove: https://github.com/rust-lang/rust/issues/74465#issuecomment-1364969188
struct LazyCell<T, F = fn() -> T> {
    cell: std::cell::OnceCell<T>,
    init: std::cell::Cell<Option<F>>,
}

impl<T, F> LazyCell<T, F> {
    pub const fn new(init: F) -> Self {
        Self {
            cell: std::cell::OnceCell::new(),
            init: std::cell::Cell::new(Some(init)),
        }
    }

    fn into_inner(self) -> Option<T> {
        self.cell.into_inner()
    }
}

#[allow(clippy::option_if_let_else)]
impl<T, F: FnOnce() -> T> LazyCell<T, F> {
    fn force(this: &Self) -> &T {
        this.cell.get_or_init(|| match this.init.take() {
            Some(f) => f(),
            None => panic!("`Lazy` instance has previously been poisoned"),
        })
    }
}

impl<T, F: FnOnce() -> T> std::ops::Deref for LazyCell<T, F> {
    type Target = T;
    fn deref(&self) -> &T {
        Self::force(self)
    }
}
