use std::{
    borrow::Cow,
    ffi::OsStr,
    fmt::Debug,
    fs, io,
    marker::PhantomData,
    path::{Path, MAIN_SEPARATOR_STR},
};

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
pub struct RemoveOp<'a, I: Into<Cow<'a, Path>> + 'a, F: IntoIterator<Item = I>> {
    files: F,
    #[builder(default = false)]
    force: bool,
    #[builder(default = true)]
    preserve_root: bool,
    #[builder(default)]
    _marker: PhantomData<&'a I>,
}

impl<'a, I: Into<Cow<'a, Path>>, F: IntoIterator<Item = I>> RemoveOp<'a, I, F> {
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

#[cfg_attr(
    feature = "tracing",
    tracing::instrument(level = "trace", skip(files, remove))
)]
fn schedule_deletions<'a, I: Into<Cow<'a, Path>>, F: IntoIterator<Item = I>>(
    RemoveOp {
        files,
        force,
        preserve_root,
        _marker: _,
    }: RemoveOp<'a, I, F>,
    remove: &impl DirectoryOp<Cow<'a, Path>>,
) -> Result<(), Error> {
    for file in files {
        let file = file.into();
        if preserve_root && file == Path::new("/") {
            return Err(Error::PreserveRoot);
        }
        let stripped_path = {
            let trailing_slash_stripped = file
                .as_os_str()
                .as_encoded_bytes()
                .strip_suffix(MAIN_SEPARATOR_STR.as_bytes())
                .unwrap_or(file.as_os_str().as_encoded_bytes());
            let path = unsafe { OsStr::from_encoded_bytes_unchecked(trailing_slash_stripped) };
            Path::new(path)
        };

        let is_dir = match stripped_path.symlink_metadata() {
            Err(e) if e.kind() == io::ErrorKind::NotFound => {
                if force {
                    continue;
                }

                return Err(Error::NotFound {
                    file: stripped_path.to_path_buf(),
                });
            }
            r => r,
        }
        .map_io_err(|| format!("Failed to read metadata for file: {stripped_path:?}"))?
        .is_dir();

        if is_dir {
            remove.run(
                if file.as_os_str().len() == stripped_path.as_os_str().len() {
                    file
                } else {
                    Cow::Owned(stripped_path.to_path_buf())
                },
            )?;
        } else {
            fs::remove_file(stripped_path)
                .map_io_err(|| format!("Failed to delete file: {stripped_path:?}"))?;
        }
    }
    Ok(())
}

#[cfg(target_os = "linux")]
mod compat {
    use std::{
        borrow::Cow,
        cell::LazyCell,
        env::{current_dir, set_current_dir},
        ffi::{CStr, CString, OsStr},
        fmt::{Debug, Formatter},
        fs,
        mem::MaybeUninit,
        num::NonZeroUsize,
        os::{
            fd::{AsFd, OwnedFd},
            unix::ffi::OsStrExt,
        },
        path::{Path, PathBuf},
        sync::Arc,
        thread,
        thread::JoinHandle,
    };

    use crossbeam_channel::{Receiver, Sender};
    use rustix::{
        fs::{openat, unlinkat, AtFlags, FileType, Mode, OFlags, RawDir, CWD},
        thread::{unshare, UnshareFlags},
    };

    use crate::{
        ops::{
            compat::DirectoryOp, concat_cstrs, get_file_type, join_cstr_paths, path_buf_to_cstring,
            IoErr,
        },
        Error,
    };

    struct Impl<LF: FnOnce() -> (Sender<TreeNode>, JoinHandle<Result<(), Error>>)> {
        #[allow(clippy::type_complexity)]
        scheduling: LazyCell<(Sender<TreeNode>, JoinHandle<Result<(), Error>>), LF>,
    }

    pub fn remove_impl<'a>() -> impl DirectoryOp<Cow<'a, Path>> {
        let scheduling = LazyCell::new(|| {
            let (tx, rx) = crossbeam_channel::unbounded();
            (tx, thread::spawn(|| root_worker_thread(rx)))
        });

        Impl { scheduling }
    }

    impl<LF: FnOnce() -> (Sender<TreeNode>, JoinHandle<Result<(), Error>>)>
        DirectoryOp<Cow<'_, Path>> for Impl<LF>
    {
        #[cfg_attr(feature = "tracing", tracing::instrument(level = "trace", skip(self)))]
        fn run(&self, dir: Cow<Path>) -> Result<(), Error> {
            let Self { ref scheduling } = *self;

            let (tasks, _) = &**scheduling;
            tasks
                .send(TreeNode {
                    path: path_buf_to_cstring(dir.into_owned())?,
                    parent: None,
                    messages: tasks.clone(),
                })
                .map_err(|_| Error::Internal)
        }

        #[cfg_attr(feature = "tracing", tracing::instrument(level = "trace", skip(self)))]
        fn finish(self) -> Result<(), Error> {
            let Self { scheduling } = self;

            if let Ok((tasks, thread)) = LazyCell::into_inner(scheduling) {
                drop(tasks);
                thread.join().map_err(|_| Error::Join)??;
            }
            Ok(())
        }
    }

    #[cfg_attr(feature = "tracing", tracing::instrument(level = "trace", skip(tasks)))]
    fn root_worker_thread(tasks: Receiver<TreeNode>) -> Result<(), Error> {
        unshare(UnshareFlags::FILES | UnshareFlags::FS).map_io_err(|| "Failed to unshare I/O.")?;

        let mut available_parallelism = thread::available_parallelism()
            .map(NonZeroUsize::get)
            .unwrap_or(1)
            - 1;

        thread::scope(|scope| {
            let mut threads = Vec::with_capacity(available_parallelism);

            {
                let mut buf = [MaybeUninit::<u8>::uninit(); 8192];
                for message in &tasks {
                    let mut maybe_spawn = || {
                        if available_parallelism > 0 && !tasks.is_empty() {
                            #[cfg(feature = "tracing")]
                            tracing::event!(
                                tracing::Level::TRACE,
                                available_parallelism,
                                "Spawning new thread."
                            );

                            available_parallelism -= 1;
                            threads.push(scope.spawn({
                                let tasks = tasks.clone();
                                || worker_thread(tasks)
                            }));
                        }
                    };
                    maybe_spawn();

                    delete_dir(message, &mut buf, maybe_spawn)?;
                }
            }

            for thread in threads {
                thread.join().map_err(|_| Error::Join)??;
            }
            Ok(())
        })
    }

    #[cfg_attr(feature = "tracing", tracing::instrument(level = "trace", skip(tasks)))]
    fn worker_thread(tasks: Receiver<TreeNode>) -> Result<(), Error> {
        unshare(UnshareFlags::FILES | UnshareFlags::FS).map_io_err(|| "Failed to unshare I/O.")?;

        let mut buf = [MaybeUninit::<u8>::uninit(); 8192];
        for message in tasks {
            delete_dir(message, &mut buf, || {})?;
        }
        Ok(())
    }

    #[cfg_attr(
        feature = "tracing",
        tracing::instrument(level = "info", skip(buf, maybe_spawn))
    )]
    fn delete_dir(
        node: TreeNode,
        buf: &mut [MaybeUninit<u8>],
        maybe_spawn: impl FnMut(),
    ) -> Result<(), Error> {
        let dir = openat(
            CWD,
            &node.path,
            OFlags::RDONLY | OFlags::DIRECTORY | OFlags::NOFOLLOW,
            Mode::empty(),
        )
        .map_io_err(|| format!("Failed to open directory: {:?}", node.path))?;
        let node = delete_dir_contents(node, dir, buf, maybe_spawn)?;
        delete_empty_dir_chain(node)
    }

    #[cfg_attr(
        feature = "tracing",
        tracing::instrument(level = "trace", skip(dir, buf, maybe_spawn))
    )]
    fn delete_dir_contents(
        node: TreeNode,
        dir: OwnedFd,
        buf: &mut [MaybeUninit<u8>],
        mut maybe_spawn: impl FnMut(),
    ) -> Result<Option<TreeNode>, Error> {
        enum Arcable<T> {
            Raw(T),
            Arced(Arc<T>),
        }

        impl<T> Arcable<T> {
            fn into_inner(this: Self) -> Option<T> {
                match this {
                    Self::Raw(t) => Some(t),
                    Self::Arced(arc) => Arc::into_inner(arc),
                }
            }
        }

        impl<T> AsRef<T> for Arcable<T> {
            fn as_ref(&self) -> &T {
                match self {
                    Self::Raw(node) => node,
                    Self::Arced(arc) => arc,
                }
            }
        }

        let mut node = Arcable::Raw(node);
        let mut raw_dir = RawDir::new(&dir, buf);
        while let Some(file) = raw_dir.next() {
            let file =
                file.map_io_err(|| format!("Failed to read directory: {:?}", node.as_ref().path))?;
            {
                let name = file.file_name();
                if name == c"." || name == c".." {
                    continue;
                }
            }

            let file_type = match file.file_type() {
                FileType::Unknown => {
                    get_file_type(&dir, file.file_name(), &node.as_ref().path, false)?
                }
                t => t,
            };
            if file_type == FileType::Directory {
                if node.as_ref().path.as_bytes_with_nul().len() + file.file_name().count_bytes()
                    > 4096
                {
                    long_path_fallback_deletion(&node.as_ref().path, file.file_name())?;
                    continue;
                }

                maybe_spawn();

                let node = match node {
                    Arcable::Raw(raw) => {
                        let arc = Arc::new(raw);
                        node = Arcable::Arced(arc.clone());
                        arc
                    }
                    Arcable::Arced(ref node) => node.clone(),
                };
                node.messages
                    .send(TreeNode {
                        path: concat_cstrs(&node.path, file.file_name()),
                        parent: Some(node.clone()),
                        messages: node.messages.clone(),
                    })
                    .map_err(|_| Error::Internal)?;
            } else {
                delete_file(node.as_ref(), &dir, file.file_name())?;
            }
        }

        Ok(Arcable::into_inner(node))
    }

    #[cfg_attr(feature = "tracing", tracing::instrument(level = "trace"))]
    fn delete_empty_dir_chain(mut node: Option<TreeNode>) -> Result<(), Error> {
        let mut result = Ok(());
        while let Some(TreeNode {
            ref path,
            parent,
            messages: _,
        }) = node
        {
            if result.is_ok() {
                result = unlinkat(CWD, path, AtFlags::REMOVEDIR)
                    .map_io_err(|| format!("Failed to delete directory: {path:?}"));
            }
            node = parent.and_then(Arc::into_inner);
        }
        result
    }

    #[cfg_attr(feature = "tracing", tracing::instrument(level = "debug", skip(dir)))]
    fn delete_file(node: &TreeNode, dir: impl AsFd, file: &CStr) -> Result<(), Error> {
        unlinkat(&dir, file, AtFlags::empty()).map_io_err(|| {
            format!(
                "Failed to delete file: {:?}",
                join_cstr_paths(&node.path, file)
            )
        })
    }

    #[cold]
    #[cfg_attr(feature = "tracing", tracing::instrument(level = "trace"))]
    fn long_path_fallback_deletion(parent: &CString, child: &CStr) -> Result<(), Error> {
        struct CurrentDir(PathBuf);

        impl CurrentDir {
            fn new() -> Result<Self, Error> {
                Ok(Self(
                    current_dir().map_io_err(|| "Failed to get current directory")?,
                ))
            }
        }

        impl Drop for CurrentDir {
            fn drop(&mut self) {
                set_current_dir(&self.0).expect("Failed to restore current dir");
            }
        }

        let _guard = CurrentDir::new()?;
        {
            let parent = Path::new(OsStr::from_bytes(parent.as_bytes()));
            set_current_dir(parent)
                .map_io_err(|| format!("Failed to set current directory: {parent:?}"))?;
        }
        {
            let child = Path::new(OsStr::from_bytes(child.to_bytes()));
            fs::remove_dir_all(child)
                .map_io_err(|| format!("Failed to delete directory and its contents: {child:?}"))?;
        }
        Ok(())
    }

    struct TreeNode {
        path: CString,
        parent: Option<Arc<TreeNode>>,
        messages: Sender<TreeNode>,
    }

    impl Debug for TreeNode {
        fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
            self.path.fmt(f)
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
