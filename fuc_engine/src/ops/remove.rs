use std::{borrow::Cow, fmt::Debug, fs, io, path::Path};

use typed_builder::TypedBuilder;

use crate::{
    ops::{compat::DirectoryOp, IoErr},
    Error,
};

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
        let remove = compat::remove_impl();
        let result = schedule_deletions(self, &remove);
        remove.finish().and(result)
    }
}

fn schedule_deletions<'a>(
    RemoveOp {
        files,
        force,
        preserve_root,
    }: RemoveOp<'a, impl IntoIterator<Item = Cow<'a, Path>>>,
    remove: &impl DirectoryOp<Cow<'a, Path>>,
) -> Result<(), Error> {
    for file in files {
        if preserve_root && file == Path::new("/") {
            return Err(Error::PreserveRoot);
        }
        let is_dir = match file.symlink_metadata() {
            Err(e) if e.kind() == io::ErrorKind::NotFound => {
                if force {
                    continue;
                }

                return Err(Error::NotFound {
                    file: file.into_owned(),
                });
            }
            r => r,
        }
        .map_io_err(|| format!("Failed to read metadata for file: {file:?}"))?
        .is_dir();

        if is_dir {
            remove.run(file)?;
        } else {
            fs::remove_file(&file).map_io_err(|| format!("Failed to delete file: {file:?}"))?;
        }
    }
    Ok(())
}

#[cfg(target_os = "linux")]
mod compat {
    use std::{
        borrow::Cow,
        ffi::{CStr, CString},
        mem::MaybeUninit,
        num::NonZeroUsize,
        path::Path,
        sync::Arc,
        thread,
        thread::JoinHandle,
    };

    use crossbeam_channel::{Receiver, Sender};
    use rustix::{
        fs::{cwd, openat, unlinkat, AtFlags, FileType, Mode, OFlags, RawDir},
        thread::{unshare, UnshareFlags},
    };

    use crate::{
        ops::{
            compat::DirectoryOp, concat_cstrs, get_file_type, join_cstr_paths, path_buf_to_cstring,
            IoErr, LazyCell,
        },
        Error,
    };

    struct Impl<LF: FnOnce() -> (Sender<Message>, JoinHandle<Result<(), Error>>)> {
        #[allow(clippy::type_complexity)]
        scheduling: LazyCell<(Sender<Message>, JoinHandle<Result<(), Error>>), LF>,
    }

    pub fn remove_impl<'a>() -> impl DirectoryOp<Cow<'a, Path>> {
        let scheduling = LazyCell::new(|| {
            let (tx, rx) = crossbeam_channel::unbounded();
            (tx, thread::spawn(|| root_worker_thread(rx)))
        });

        Impl { scheduling }
    }

    impl<LF: FnOnce() -> (Sender<Message>, JoinHandle<Result<(), Error>>)>
        DirectoryOp<Cow<'_, Path>> for Impl<LF>
    {
        fn run(&self, dir: Cow<Path>) -> Result<(), Error> {
            let Self { ref scheduling } = *self;

            let (tasks, _) = &**scheduling;
            tasks
                .send(Message::Node(TreeNode {
                    path: path_buf_to_cstring(dir.into_owned())?,
                    _parent: None,
                    messages: tasks.clone(),
                }))
                .map_err(|_| Error::Internal)
        }

        fn finish(self) -> Result<(), Error> {
            let Self { scheduling } = self;

            if let Some((tasks, thread)) = scheduling.into_inner() {
                drop(tasks);
                thread.join().map_err(|_| Error::Join)??;
            }
            Ok(())
        }
    }

    #[allow(clippy::needless_pass_by_value)]
    fn root_worker_thread(tasks: Receiver<Message>) -> Result<(), Error> {
        let mut available_parallelism = thread::available_parallelism()
            .map(NonZeroUsize::get)
            .unwrap_or(1)
            - 1;

        thread::scope(|scope| {
            let mut threads = Vec::with_capacity(available_parallelism);

            {
                let mut buf = [MaybeUninit::<u8>::uninit(); 8192];
                for message in &tasks {
                    let maybe_spawn = || {
                        if available_parallelism > 0 {
                            available_parallelism -= 1;
                            threads.push(scope.spawn({
                                let tasks = tasks.clone();
                                || worker_thread(tasks)
                            }));
                        }
                    };

                    match message {
                        Message::Node(node) => delete_dir(node, &mut buf, maybe_spawn)?,
                        Message::Error(e) => return Err(e),
                    }
                }
            }

            for thread in threads {
                thread.join().map_err(|_| Error::Join)??;
            }
            Ok(())
        })
    }

    fn worker_thread(tasks: Receiver<Message>) -> Result<(), Error> {
        unshare(UnshareFlags::FILES).map_io_err(|| "Failed to unshare FD table.".to_string())?;

        let mut buf = [MaybeUninit::<u8>::uninit(); 8192];
        for message in tasks {
            match message {
                Message::Node(node) => delete_dir(node, &mut buf, || {})?,
                Message::Error(e) => return Err(e),
            }
        }
        Ok(())
    }

    fn delete_dir(
        node: TreeNode,
        buf: &mut [MaybeUninit<u8>],
        mut maybe_spawn: impl FnMut(),
    ) -> Result<(), Error> {
        let dir = openat(
            cwd(),
            node.path.as_c_str(),
            OFlags::RDONLY | OFlags::DIRECTORY | OFlags::NOFOLLOW,
            Mode::empty(),
        )
        .map_io_err(|| format!("Failed to open directory: {:?}", node.path))?;

        let node = LazyCell::new(|| Arc::new(node));
        let mut raw_dir = RawDir::new(&dir, buf);
        while let Some(file) = raw_dir.next() {
            // TODO here and other uses: https://github.com/rust-lang/rust/issues/105723
            const DOT: &CStr = CStr::from_bytes_with_nul(b".\0").ok().unwrap();
            const DOT_DOT: &CStr = CStr::from_bytes_with_nul(b"..\0").ok().unwrap();

            let file = file.map_io_err(|| format!("Failed to read directory: {:?}", node.path))?;
            if file.file_name() == DOT || file.file_name() == DOT_DOT {
                continue;
            }

            let file_type = match file.file_type() {
                FileType::Unknown => get_file_type(&dir, file.file_name(), &node.path)?,
                t => t,
            };
            if file_type == FileType::Directory {
                node.messages
                    .send(Message::Node(TreeNode {
                        path: concat_cstrs(&node.path, file.file_name()),
                        _parent: Some(node.clone()),
                        messages: node.messages.clone(),
                    }))
                    .map_err(|_| Error::Internal)?;
                maybe_spawn();
            } else {
                unlinkat(&dir, file.file_name(), AtFlags::empty()).map_io_err(|| {
                    format!(
                        "Failed to delete file: {:?}",
                        join_cstr_paths(&node.path, file.file_name())
                    )
                })?;
            }
        }
        Ok(())
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
            let Self {
                ref path,
                _parent: _,
                ref messages,
            } = *self;

            if let Err(e) = unlinkat(cwd(), path.as_c_str(), AtFlags::REMOVEDIR)
                .map_io_err(|| format!("Failed to delete directory: {path:?}"))
            {
                // If the receiver closed, then another error must have already occurred.
                drop(messages.send(Message::Error(e)));
            }
        }
    }
}

#[cfg(all(not(target_os = "linux"), not(target_os = "windows")))]
mod compat {
    use std::{borrow::Cow, fs, io, path::Path};

    use rayon::prelude::*;

    use crate::{
        ops::{compat::DirectoryOp, IoErr},
        Error,
    };

    struct Impl;

    pub fn remove_impl<'a>() -> impl DirectoryOp<Cow<'a, Path>> {
        Impl
    }

    impl DirectoryOp<Cow<'_, Path>> for Impl {
        fn run(&self, dir: Cow<Path>) -> Result<(), Error> {
            remove_dir_all(&dir).map_io_err(|| format!("Failed to delete directory: {dir:?}"))
        }

        fn finish(self) -> Result<(), Error> {
            Ok(())
        }
    }

    fn remove_dir_all<P: AsRef<Path>>(path: P) -> Result<(), io::Error> {
        let path = path.as_ref();
        path.read_dir()?
            .par_bridge()
            .try_for_each(|dir_entry| -> io::Result<()> {
                let dir_entry = dir_entry?;
                if dir_entry.file_type()?.is_dir() {
                    remove_dir_all(dir_entry.path())?;
                } else {
                    fs::remove_file(dir_entry.path())?;
                }
                Ok(())
            })?;
        fs::remove_dir(path)
    }
}

#[cfg(target_os = "windows")]
mod compat {
    use std::{borrow::Cow, path::Path};

    use remove_dir_all::remove_dir_all;

    use crate::{
        ops::{compat::DirectoryOp, IoErr},
        Error,
    };

    struct Impl;

    pub fn remove_impl<'a>() -> impl DirectoryOp<Cow<'a, Path>> {
        Impl
    }

    impl DirectoryOp<Cow<'_, Path>> for Impl {
        fn run(&self, dir: Cow<Path>) -> Result<(), Error> {
            remove_dir_all(&dir).map_io_err(|| format!("Failed to delete directory: {dir:?}"))
        }

        fn finish(self) -> Result<(), Error> {
            Ok(())
        }
    }
}
