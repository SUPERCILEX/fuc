use std::{
    borrow::Cow,
    ffi::OsStr,
    fmt::Debug,
    fs, io,
    marker::PhantomData,
    path::{MAIN_SEPARATOR_STR, Path},
};

use bon::Builder;

use crate::{
    Error,
    ops::{IoErr, compat::DirectoryOp},
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

#[derive(Debug, Builder)]
pub struct RemoveOp<'a, I: Into<Cow<'a, Path>> + 'a, F: IntoIterator<Item = I>> {
    files: F,
    #[builder(default = false)]
    force: bool,
    #[builder(default = true)]
    preserve_root: bool,
    #[builder(skip)]
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
        env,
        env::{current_dir, set_current_dir},
        ffi::{CStr, CString, OsStr},
        fmt::{Debug, Formatter},
        fs,
        mem::MaybeUninit,
        os::{fd::AsFd, unix::ffi::OsStrExt},
        path::{Path, PathBuf},
        sync::Arc,
    };

    use lockness_executor::{
        LocknessExecutor, LocknessExecutorBuilder, Spawner,
        config::{Config, Lifo, True},
    };
    use rustix::{
        fs::{AtFlags, CWD, FileType, Mode, OFlags, RawDir, openat, unlinkat},
        io::Errno,
        thread::{UnshareFlags, unshare_unsafe},
    };

    use crate::{
        Error,
        ops::{IoErr, compat::DirectoryOp, concat_cstrs, join_cstr_paths, path_buf_to_cstring},
    };

    struct ThreadState {
        buf: [MaybeUninit<u8>; 8192],
    }

    #[derive(Clone, Debug)]
    struct Params {
        unshare_io: bool,
    }

    impl Config for Params {
        const NUM_TASK_TYPES: usize = 1;
        type AllowTasksToSpawnMoreTasks = True;
        type DequeBias = Lifo;

        type Error = Error;
        type ThreadLocalState = ThreadState;

        #[cfg_attr(feature = "tracing", tracing::instrument(level = "trace"))]
        #[inline]
        fn thread_initializer(self) -> Result<Self::ThreadLocalState, Self::Error> {
            let Self { unshare_io } = self;

            if unshare_io {
                unsafe { unshare_unsafe(UnshareFlags::FILES | UnshareFlags::FS) }
                    .map_io_err(|| "Failed to unshare I/O.")?;
            }

            Ok(ThreadState {
                buf: [MaybeUninit::uninit(); 8192],
            })
        }
    }

    struct Impl<C, LF> {
        executor: LazyCell<LocknessExecutor<C>, LF>,
    }

    pub fn remove_impl<'a>() -> impl DirectoryOp<Cow<'a, Path>> {
        let inner = LazyCell::new(|| {
            let unshare_io = env::var_os("NO_UNSHARE").is_none();
            LocknessExecutorBuilder::new().build(Params { unshare_io })
        });

        Impl { executor: inner }
    }

    impl<LF: FnOnce() -> LocknessExecutor<Params>> DirectoryOp<Cow<'_, Path>> for Impl<Params, LF> {
        #[cfg_attr(feature = "tracing", tracing::instrument(level = "trace", skip(self)))]
        fn run(&self, dir: Cow<Path>) -> Result<(), Error> {
            let Self { executor } = self;

            let node = TreeNode {
                path: path_buf_to_cstring(dir.into_owned())?,
                parent: None,
            };
            executor
                .spawner()
                .buffered()
                .spawn_recursive(|spawner, state| delete_dir(node, state, spawner));
            Ok(())
        }

        #[cfg_attr(feature = "tracing", tracing::instrument(level = "trace", skip(self)))]
        fn finish(self) -> Result<(), Error> {
            let Self { executor } = self;

            if let Ok(executor) = LazyCell::into_inner(executor)
                && let Some(e) = executor.finisher().next()
            {
                return Err(match e {
                    lockness_executor::Error::Panic(p) => Error::Join(p),
                    lockness_executor::Error::Error(e) => e,
                });
            }
            Ok(())
        }
    }

    #[cfg_attr(
        feature = "tracing",
        tracing::instrument(level = "info", skip(buf, spawner))
    )]
    fn delete_dir(
        node: TreeNode,
        ThreadState { buf }: &mut ThreadState,
        spawner: &Spawner<Params>,
    ) -> Result<(), Error> {
        delete_dir_contents(node, buf, spawner).and_then(delete_empty_dir_chain)
    }

    #[cfg_attr(
        feature = "tracing",
        tracing::instrument(level = "trace", skip(buf, spawner))
    )]
    fn delete_dir_contents(
        node: TreeNode,
        buf: &mut [MaybeUninit<u8>],
        spawner: &Spawner<Params>,
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

        let dir = openat(
            CWD,
            &node.path,
            OFlags::RDONLY | OFlags::DIRECTORY | OFlags::NOFOLLOW,
            Mode::empty(),
        )
        .map_io_err(|| format!("Failed to open directory: {:?}", node.path))?;

        let mut node = Arcable::Raw(node);
        let mut raw_dir = RawDir::new(&dir, buf);
        let tasks = spawner.buffered();
        loop {
            if raw_dir.is_buffer_empty() {
                tasks.flush();
            }
            let Some(file) = raw_dir.next() else { break };

            let file =
                file.map_io_err(|| format!("Failed to read directory: {:?}", node.as_ref().path))?;
            {
                let name = file.file_name();
                if name == c"." || name == c".." {
                    continue;
                }
            }

            if file.file_type() != FileType::Directory {
                tasks.flush();
                let file = file.file_name();
                match delete_file(&dir, file) {
                    Ok(()) => continue,
                    Err(Errno::ISDIR) => (),
                    Err(error) => {
                        return Err(Error::Io {
                            error: error.into(),
                            context: format!(
                                "Failed to delete file: {:?}",
                                join_cstr_paths(&node.as_ref().path, file)
                            )
                            .into(),
                        });
                    }
                }
            }

            if node.as_ref().path.as_bytes_with_nul().len() + file.file_name().count_bytes() > 4096
            {
                tasks.flush();
                long_path_fallback_deletion(&node.as_ref().path, file.file_name())?;
                continue;
            }

            let node = match node {
                Arcable::Raw(raw) => {
                    let arc = Arc::new(raw);
                    node = Arcable::Arced(arc.clone());
                    arc
                }
                Arcable::Arced(ref node) => node.clone(),
            };
            let node = TreeNode {
                path: concat_cstrs(&node.path, file.file_name()),
                parent: Some(node.clone()),
            };
            tasks.spawn_recursive(|executor, state| delete_dir(node, state, executor));
        }
        drop(tasks);

        Ok(Arcable::into_inner(node))
    }

    #[cfg_attr(feature = "tracing", tracing::instrument(level = "trace"))]
    fn delete_empty_dir_chain(mut node: Option<TreeNode>) -> Result<(), Error> {
        let mut result = Ok(());
        while let Some(TreeNode { ref path, parent }) = node {
            if result.is_ok() {
                result = unlinkat(CWD, path, AtFlags::REMOVEDIR)
                    .map_io_err(|| format!("Failed to delete directory: {path:?}"));
            }
            node = parent.and_then(Arc::into_inner);
        }
        result
    }

    #[cfg_attr(feature = "tracing", tracing::instrument(level = "debug", skip(dir)))]
    fn delete_file(dir: impl AsFd, file: &CStr) -> rustix::io::Result<()> {
        unlinkat(&dir, file, AtFlags::empty())
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
    }

    impl Debug for TreeNode {
        fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
            self.path.fmt(f)
        }
    }
}

#[cfg(all(not(target_os = "linux"), not(target_os = "windows")))]
mod compat {
    use std::{borrow::Cow, fmt::Debug, fs, io, path::Path};

    use rayon::prelude::*;

    use crate::{
        Error,
        ops::{IoErr, compat::DirectoryOp},
    };

    struct Impl;

    pub fn remove_impl<'a>() -> impl DirectoryOp<Cow<'a, Path>> {
        Impl
    }

    impl DirectoryOp<Cow<'_, Path>> for Impl {
        #[cfg_attr(feature = "tracing", tracing::instrument(level = "trace", skip(self)))]
        fn run(&self, dir: Cow<Path>) -> Result<(), Error> {
            remove_dir_all(&dir).map_io_err(|| format!("Failed to delete directory: {dir:?}"))
        }

        #[cfg_attr(feature = "tracing", tracing::instrument(level = "trace", skip(self)))]
        fn finish(self) -> Result<(), Error> {
            Ok(())
        }
    }

    #[cfg_attr(feature = "tracing", tracing::instrument(level = "info"))]
    fn remove_dir_all<P: AsRef<Path> + Debug>(path: P) -> Result<(), io::Error> {
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
        Error,
        ops::{IoErr, compat::DirectoryOp},
    };

    struct Impl;

    pub fn remove_impl<'a>() -> impl DirectoryOp<Cow<'a, Path>> {
        Impl
    }

    impl DirectoryOp<Cow<'_, Path>> for Impl {
        #[cfg_attr(feature = "tracing", tracing::instrument(level = "trace", skip(self)))]
        fn run(&self, dir: Cow<Path>) -> Result<(), Error> {
            remove_dir_all(&dir).map_io_err(|| format!("Failed to delete directory: {dir:?}"))
        }

        #[cfg_attr(feature = "tracing", tracing::instrument(level = "trace", skip(self)))]
        fn finish(self) -> Result<(), Error> {
            Ok(())
        }
    }
}
