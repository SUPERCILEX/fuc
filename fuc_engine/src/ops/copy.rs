use std::{borrow::Cow, fmt::Debug, fs, io, marker::PhantomData, path::Path};

use bon::Builder;

use crate::{
    Error,
    ops::{IoErr, compat::DirectoryOp},
};

/// Copies a file or directory at this path.
///
/// # Errors
///
/// Returns the underlying I/O errors that occurred.
pub fn copy_file<P: AsRef<Path>, Q: AsRef<Path>>(from: P, to: Q) -> Result<(), Error> {
    CopyOp::builder()
        .files([(Cow::Borrowed(from.as_ref()), Cow::Borrowed(to.as_ref()))])
        .build()
        .run()
}

#[derive(Debug, Builder)]
pub struct CopyOp<
    'a,
    'b,
    I1: Into<Cow<'a, Path>> + 'a,
    I2: Into<Cow<'b, Path>> + 'b,
    F: IntoIterator<Item = (I1, I2)>,
> {
    files: F,
    #[builder(default = false)]
    force: bool,
    #[builder(default = false)]
    follow_symlinks: bool,
    #[builder(default = false)]
    hard_link: bool,
    #[builder(skip)]
    _marker1: PhantomData<&'a I1>,
    #[builder(skip)]
    _marker2: PhantomData<&'b I2>,
}

impl<
    'a,
    'b,
    I1: Into<Cow<'a, Path>> + 'a,
    I2: Into<Cow<'b, Path>> + 'b,
    F: IntoIterator<Item = (I1, I2)>,
> CopyOp<'a, 'b, I1, I2, F>
{
    /// Consume and run this copy operation.
    ///
    /// # Errors
    ///
    /// Returns the underlying I/O errors that occurred.
    pub fn run(self) -> Result<(), Error> {
        let copy = compat::copy_impl(self.follow_symlinks, self.hard_link);
        let result = schedule_copies(self, &copy);
        copy.finish().and(result)
    }
}

#[cfg_attr(
    feature = "tracing",
    tracing::instrument(level = "trace", skip(files, copy))
)]
fn schedule_copies<
    'a,
    'b,
    I1: Into<Cow<'a, Path>> + 'a,
    I2: Into<Cow<'b, Path>> + 'b,
    F: IntoIterator<Item = (I1, I2)>,
>(
    CopyOp {
        files,
        force,
        follow_symlinks,
        hard_link,
        _marker1: _,
        _marker2: _,
    }: CopyOp<'a, 'b, I1, I2, F>,
    copy: &impl DirectoryOp<(Cow<'a, Path>, Cow<'b, Path>)>,
) -> Result<(), Error> {
    for (from, to) in files {
        let from = from.into();
        let to = to.into();
        if !force {
            match to.symlink_metadata() {
                Ok(_) => {
                    return Err(Error::AlreadyExists {
                        file: to.into_owned(),
                    });
                }
                Err(e) if e.kind() == io::ErrorKind::NotFound => {
                    // Do nothing, this is good
                }
                r => {
                    r.map_io_err(|| format!("Failed to read metadata for file: {to:?}"))?;
                }
            }
        }

        let from_metadata = if follow_symlinks {
            from.metadata()
        } else {
            from.symlink_metadata()
        }
        .map_io_err(|| format!("Failed to read metadata for file: {from:?}"))?;

        if from_metadata.is_dir() {
            #[cfg_attr(not(unix), allow(unused_mut))]
            let mut builder = fs::DirBuilder::new();
            #[cfg(unix)]
            {
                use std::os::unix::fs::{DirBuilderExt, MetadataExt};
                builder.mode(from_metadata.mode());
            }
            match builder.create(&to) {
                Err(e) if force && e.kind() == io::ErrorKind::AlreadyExists => {}
                r => r.map_io_err(|| format!("Failed to create directory: {to:?}"))?,
            }
            copy.run((from, to))?;
        } else if from_metadata.is_symlink() {
            let link =
                fs::read_link(&from).map_io_err(|| format!("Failed to read symlink: {from:?}"))?;
            match fs::remove_file(&to) {
                Err(e) if e.kind() == io::ErrorKind::NotFound => (),
                r => r.map_io_err(|| format!("Failed to remove existing file: {to:?}"))?,
            }
            if hard_link {
                fs::hard_link(&link, &to)
                    .map_io_err(|| format!("Failed to create hard link: {to:?} -> {link:?}"))?;
            } else {
                let run = || {
                    #[cfg(unix)]
                    {
                        std::os::unix::fs::symlink(&link, &to)
                    }
                    #[cfg(windows)]
                    if fs::metadata(&link)?.file_type().is_dir() {
                        std::os::windows::fs::symlink_dir(&link, &to)
                    } else {
                        std::os::windows::fs::symlink_file(&link, &to)
                    }
                };
                run().map_io_err(|| format!("Failed to create symlink: {to:?} -> {link:?}"))?;
            }
        } else if hard_link {
            match fs::remove_file(&to) {
                Err(e) if e.kind() == io::ErrorKind::NotFound => (),
                r => r.map_io_err(|| format!("Failed to remove existing file: {to:?}"))?,
            }
            fs::hard_link(&from, &to)
                .map_io_err(|| format!("Failed to create hard link: {to:?} -> {from:?}"))?;
        } else {
            fs::copy(&from, &to).map_io_err(|| format!("Failed to copy file: {from:?}"))?;
        }
    }
    Ok(())
}

#[cfg(target_os = "linux")]
mod compat {
    use std::{
        borrow::Cow,
        cell::{Cell, LazyCell},
        env,
        ffi::{CStr, CString},
        fmt::{Debug, Formatter},
        fs::File,
        io,
        mem::MaybeUninit,
        num::NonZeroUsize,
        os::unix::io::{AsFd, OwnedFd},
        path::Path,
        thread,
        thread::JoinHandle,
    };

    use crossbeam_channel::{Receiver, Sender};
    use rustix::{
        fs::{
            AtFlags, CWD, FileType, Mode, OFlags, RawDir, StatxFlags, copy_file_range, linkat,
            mkdirat, openat, readlinkat, statx, symlinkat,
        },
        io::Errno,
        thread::{UnshareFlags, unshare_unsafe},
    };

    use crate::{
        Error,
        ops::{IoErr, compat::DirectoryOp, concat_cstrs, join_cstr_paths, path_buf_to_cstring},
    };

    struct Impl<LF: FnOnce() -> (Sender<TreeNode>, JoinHandle<Result<(), Error>>)> {
        scheduling: LazyCell<(Sender<TreeNode>, JoinHandle<Result<(), Error>>), LF>,
    }

    pub fn copy_impl<'a, 'b>(
        follow_symlinks: bool,
        hard_link: bool,
    ) -> impl DirectoryOp<(Cow<'a, Path>, Cow<'b, Path>)> {
        let scheduling = LazyCell::new(move || {
            let (tx, rx) = crossbeam_channel::unbounded();
            (
                tx,
                if hard_link {
                    thread::spawn(move || root_worker_thread::<true>(rx, follow_symlinks))
                } else {
                    thread::spawn(move || root_worker_thread::<false>(rx, follow_symlinks))
                },
            )
        });

        Impl { scheduling }
    }

    impl<LF: FnOnce() -> (Sender<TreeNode>, JoinHandle<Result<(), Error>>)>
        DirectoryOp<(Cow<'_, Path>, Cow<'_, Path>)> for Impl<LF>
    {
        #[cfg_attr(feature = "tracing", tracing::instrument(level = "trace", skip(self)))]
        fn run(&self, (from, to): (Cow<Path>, Cow<Path>)) -> Result<(), Error> {
            let to = path_buf_to_cstring(to.into_owned())?;
            let root_to_inode = {
                let to_metadata = statx(CWD, &to, AtFlags::SYMLINK_NOFOLLOW, StatxFlags::INO)
                    .map_io_err(|| format!("Failed to stat directory: {to:?}"))?;
                to_metadata.stx_ino
            };

            let (tasks, _) = &*self.scheduling;
            tasks
                .send(TreeNode {
                    from: path_buf_to_cstring(from.into_owned())?,
                    to,
                    root_to_inode,
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

    fn unshare_files() -> Result<(), Error> {
        if env::var_os("NO_UNSHARE").is_none() {
            unsafe { unshare_unsafe(UnshareFlags::FILES) }
                .map_io_err(|| "Failed to unshare FD table.")?;
        }
        Ok(())
    }

    #[cfg_attr(feature = "tracing", tracing::instrument(level = "trace", skip(tasks)))]
    fn root_worker_thread<const HARD_LINK: bool>(
        tasks: Receiver<TreeNode>,
        follow_symlinks: bool,
    ) -> Result<(), Error> {
        unshare_files()?;

        let mut available_parallelism =
            thread::available_parallelism().map_or(1, NonZeroUsize::get) - 1;

        thread::scope(|scope| {
            let mut threads = Vec::with_capacity(available_parallelism);

            {
                let mut buf = [MaybeUninit::<u8>::uninit(); 8192];
                let symlink_buf_cache = Cell::new(Vec::new());
                for node in &tasks {
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
                                move || worker_thread::<HARD_LINK>(tasks, follow_symlinks)
                            }));
                        }
                    };
                    maybe_spawn();

                    copy_dir::<HARD_LINK>(
                        node,
                        follow_symlinks,
                        &mut buf,
                        &symlink_buf_cache,
                        maybe_spawn,
                    )?;
                }
            }

            for thread in threads {
                thread.join().map_err(|_| Error::Join)??;
            }
            Ok(())
        })
    }

    #[cfg_attr(feature = "tracing", tracing::instrument(level = "trace", skip(tasks)))]
    fn worker_thread<const HARD_LINK: bool>(
        tasks: Receiver<TreeNode>,
        follow_symlinks: bool,
    ) -> Result<(), Error> {
        unshare_files()?;

        let mut buf = [MaybeUninit::<u8>::uninit(); 8192];
        let symlink_buf_cache = Cell::new(Vec::new());
        for node in tasks {
            copy_dir::<HARD_LINK>(node, follow_symlinks, &mut buf, &symlink_buf_cache, || {})?;
        }
        Ok(())
    }

    #[cold]
    #[cfg_attr(feature = "tracing", tracing::instrument(level = "trace", skip(dir)))]
    pub fn get_file_type(
        dir: impl AsFd,
        file_name: &CStr,
        path: &CString,
        follow_symlinks: bool,
    ) -> Result<FileType, Error> {
        let flags = if follow_symlinks {
            AtFlags::empty()
        } else {
            AtFlags::SYMLINK_NOFOLLOW
        };
        statx(dir, file_name, flags, StatxFlags::TYPE)
            .map_io_err(|| {
                format!(
                    "Failed to stat file: {:?}",
                    join_cstr_paths(path, file_name)
                )
            })
            .map(|metadata| FileType::from_raw_mode(metadata.stx_mode.into()))
    }

    #[cfg_attr(
        feature = "tracing",
        tracing::instrument(level = "info", skip(messages, buf, symlink_buf_cache, maybe_spawn))
    )]
    fn copy_dir<const HARD_LINK: bool>(
        TreeNode {
            from,
            to,
            root_to_inode,
            messages,
        }: TreeNode,
        follow_symlinks: bool,
        buf: &mut [MaybeUninit<u8>],
        symlink_buf_cache: &Cell<Vec<u8>>,
        mut maybe_spawn: impl FnMut(),
    ) -> Result<(), Error> {
        let from_dir = openat(
            CWD,
            &from,
            OFlags::RDONLY
                | OFlags::DIRECTORY
                | if follow_symlinks {
                    OFlags::empty()
                } else {
                    OFlags::NOFOLLOW
                },
            Mode::empty(),
        )
        .map_io_err(|| format!("Failed to open directory: {from:?}"))?;
        let to_dir = openat(
            CWD,
            &to,
            OFlags::RDONLY | OFlags::DIRECTORY | OFlags::PATH,
            Mode::empty(),
        )
        .map_io_err(|| format!("Failed to open directory: {to:?}"))?;

        let mut failed_cross_device = false;
        let mut raw_dir = RawDir::new(&from_dir, buf);
        while let Some(file) = raw_dir.next() {
            let file = file.map_io_err(|| format!("Failed to read directory: {from:?}"))?;
            if file.ino() == root_to_inode {
                // Block recursive descent from parent into child (e.g. cp parent parent/child).
                continue;
            }
            {
                let name = file.file_name();
                if name == c"." || name == c".." {
                    continue;
                }
            }

            let mut file_type = file.file_type();
            if file_type == FileType::Unknown || (follow_symlinks && file_type == FileType::Symlink)
            {
                file_type = get_file_type(&from_dir, file.file_name(), &from, follow_symlinks)?;
            }
            let file_type = file_type;
            if file_type == FileType::Directory {
                let from = concat_cstrs(&from, file.file_name());
                let to = concat_cstrs(&to, file.file_name());

                copy_one_dir(&from_dir, &from, &to)?;
                maybe_spawn();
                messages
                    .send(TreeNode {
                        from,
                        to,
                        root_to_inode,
                        messages: messages.clone(),
                    })
                    .map_err(|_| Error::Internal)?;
            } else if HARD_LINK {
                let name = file.file_name();
                let flags = if follow_symlinks {
                    AtFlags::SYMLINK_FOLLOW
                } else {
                    AtFlags::empty()
                };
                linkat(&from_dir, name, &to_dir, name, flags).map_io_err(|| {
                    format!(
                        "Failed to create symlink: {:?} -> {:?}",
                        join_cstr_paths(&to, name),
                        join_cstr_paths(&from, name),
                    )
                })?;
            } else {
                copy_one_file(
                    &from_dir,
                    &to_dir,
                    file.file_name(),
                    file_type,
                    &from,
                    &to,
                    symlink_buf_cache,
                    &mut failed_cross_device,
                )?;
            }
        }
        Ok(())
    }

    #[cfg_attr(
        feature = "tracing",
        tracing::instrument(level = "trace", skip(from_dir))
    )]
    pub fn copy_one_dir(
        from_dir: impl AsFd,
        from_path: &CString,
        to_path: &CString,
    ) -> Result<(), Error> {
        let from_mode = {
            let from_metadata = statx(from_dir, c"", AtFlags::EMPTY_PATH, StatxFlags::MODE)
                .map_io_err(|| format!("Failed to stat directory: {from_path:?}"))?;
            Mode::from_raw_mode(from_metadata.stx_mode.into())
        };
        match mkdirat(CWD, to_path, from_mode) {
            Err(Errno::EXIST) => {}
            r => r.map_io_err(|| format!("Failed to create directory: {to_path:?}"))?,
        }

        Ok(())
    }

    #[cfg_attr(
        feature = "tracing",
        tracing::instrument(level = "debug", skip(from_dir, to_dir, symlink_buf_cache))
    )]
    fn copy_one_file(
        from_dir: impl AsFd,
        to_dir: impl AsFd,
        file_name: &CStr,
        file_type: FileType,
        from_path: &CString,
        to_path: &CString,
        symlink_buf_cache: &Cell<Vec<u8>>,
        failed_cross_device: &mut bool,
    ) -> Result<(), Error> {
        if file_type == FileType::Symlink {
            copy_symlink(
                from_dir,
                to_dir,
                file_name,
                from_path,
                to_path,
                symlink_buf_cache,
            )
        } else {
            let (from, to, from_size) =
                prep_regular_file(from_dir, to_dir, file_name, from_path, to_path)?;
            if file_type == FileType::RegularFile && !*failed_cross_device {
                copy_regular_file(
                    from,
                    to,
                    file_name,
                    from_path,
                    from_size,
                    failed_cross_device,
                )
            } else {
                copy_any_file(from, to, file_name, from_path)
            }
        }
    }

    #[cfg_attr(
        feature = "tracing",
        tracing::instrument(level = "trace", skip(from, to))
    )]
    fn copy_regular_file(
        from: OwnedFd,
        to: OwnedFd,
        file_name: &CStr,
        from_path: &CString,
        from_size: u64,
        failed_cross_device: &mut bool,
    ) -> Result<(), Error> {
        let mut total_copied = 0;
        loop {
            let byte_copied =
                match copy_file_range(&from, None, &to, None, usize::MAX / 2 - total_copied) {
                    Err(Errno::XDEV) if total_copied == 0 => {
                        *failed_cross_device = true;
                        return copy_any_file(from, to, file_name, from_path);
                    }
                    r => r.map_io_err(|| {
                        format!(
                            "Failed to copy file: {:?}",
                            join_cstr_paths(from_path, file_name)
                        )
                    })?,
                };
            total_copied += byte_copied;

            if u64::try_from(total_copied).unwrap() == from_size || byte_copied == 0 {
                return Ok(());
            }
        }
    }

    #[cold]
    #[cfg_attr(
        feature = "tracing",
        tracing::instrument(level = "trace", skip(from, to))
    )]
    fn copy_any_file(
        from: OwnedFd,
        to: OwnedFd,
        file_name: &CStr,
        from_path: &CString,
    ) -> Result<(), Error> {
        io::copy(&mut File::from(from), &mut File::from(to))
            .map_io_err(|| {
                format!(
                    "Failed to copy file: {:?}",
                    join_cstr_paths(from_path, file_name)
                )
            })
            .map(|_| ())
    }

    #[cfg_attr(
        feature = "tracing",
        tracing::instrument(level = "trace", skip(from_dir, to_dir))
    )]
    fn prep_regular_file(
        from_dir: impl AsFd,
        to_dir: impl AsFd,
        file_name: &CStr,
        from_path: &CString,
        to_path: &CString,
    ) -> Result<(OwnedFd, OwnedFd, u64), Error> {
        let from =
            openat(&from_dir, file_name, OFlags::RDONLY, Mode::empty()).map_io_err(|| {
                format!(
                    "Failed to open file: {:?}",
                    join_cstr_paths(from_path, file_name)
                )
            })?;

        let from_size;
        let to = {
            let from_mode = {
                let from_metadata = statx(
                    from_dir,
                    file_name,
                    AtFlags::empty(),
                    StatxFlags::MODE | StatxFlags::SIZE,
                )
                .map_io_err(|| {
                    format!(
                        "Failed to stat file: {:?}",
                        join_cstr_paths(from_path, file_name)
                    )
                })?;
                from_size = from_metadata.stx_size;
                Mode::from_raw_mode(from_metadata.stx_mode.into())
            };
            openat(
                &to_dir,
                file_name,
                OFlags::CREATE | OFlags::TRUNC | OFlags::WRONLY,
                from_mode,
            )
            .map_io_err(|| {
                format!(
                    "Failed to open file: {:?}",
                    join_cstr_paths(to_path, file_name)
                )
            })?
        };

        Ok((from, to, from_size))
    }

    #[cold]
    #[cfg_attr(
        feature = "tracing",
        tracing::instrument(level = "trace", skip(from_dir, to_dir, symlink_buf_cache))
    )]
    fn copy_symlink(
        from_dir: impl AsFd,
        to_dir: impl AsFd,
        file_name: &CStr,
        from_path: &CString,
        to_path: &CString,
        symlink_buf_cache: &Cell<Vec<u8>>,
    ) -> Result<(), Error> {
        let from_symlink =
            readlinkat(from_dir, file_name, symlink_buf_cache.take()).map_io_err(|| {
                format!(
                    "Failed to read symlink: {:?}",
                    join_cstr_paths(from_path, file_name)
                )
            })?;

        symlinkat(&from_symlink, &to_dir, file_name).map_io_err(|| {
            format!(
                "Failed to create symlink: {:?} -> {from_symlink:?}",
                join_cstr_paths(to_path, file_name),
            )
        })?;

        symlink_buf_cache.set(from_symlink.into_bytes_with_nul());
        Ok(())
    }

    struct TreeNode {
        from: CString,
        to: CString,
        root_to_inode: u64,
        messages: Sender<Self>,
    }

    impl Debug for TreeNode {
        fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
            f.debug_struct("TreeNode")
                .field("from", &self.from)
                .field("to", &self.to)
                .field("root_to_inode", &self.root_to_inode)
                .finish_non_exhaustive()
        }
    }
}

#[cfg(not(target_os = "linux"))]
mod compat {
    use std::{borrow::Cow, fmt::Debug, fs, io, path::Path};

    use rayon::prelude::*;

    use crate::{
        Error,
        ops::{IoErr, compat::DirectoryOp},
    };

    struct Impl {
        follow_symlinks: bool,
        hard_link: bool,
    }

    pub fn copy_impl<'a, 'b>(
        follow_symlinks: bool,
        hard_link: bool,
    ) -> impl DirectoryOp<(Cow<'a, Path>, Cow<'b, Path>)> {
        Impl {
            follow_symlinks,
            hard_link,
        }
    }

    impl DirectoryOp<(Cow<'_, Path>, Cow<'_, Path>)> for Impl {
        #[cfg_attr(feature = "tracing", tracing::instrument(level = "trace", skip(self)))]
        fn run(&self, (from, to): (Cow<Path>, Cow<Path>)) -> Result<(), Error> {
            #[cfg(unix)]
            let root_to_inode = {
                use std::os::unix::fs::MetadataExt;
                fs::metadata(&*to)
                    .map_io_err(|| format!("Failed to get inode: {to:?}"))?
                    .ino()
            };
            // TODO get rid of this crap once https://github.com/tokio-rs/tracing/issues/3320 is fixed
            #[cfg(not(unix))]
            let root_to_inode = 0;
            copy_dir(
                &from,
                to,
                self.follow_symlinks,
                self.hard_link,
                root_to_inode,
            )
            .map_io_err(|| format!("Failed to copy directory: {from:?}"))
        }

        #[cfg_attr(feature = "tracing", tracing::instrument(level = "trace", skip(self)))]
        fn finish(self) -> Result<(), Error> {
            Ok(())
        }
    }

    #[cfg_attr(feature = "tracing", tracing::instrument(level = "info"))]
    fn copy_dir<P: AsRef<Path> + Debug, Q: AsRef<Path> + Debug>(
        from: P,
        to: Q,
        follow_symlinks: bool,
        hard_link: bool,
        root_to_inode: u64,
    ) -> Result<(), io::Error> {
        let to = to.as_ref();
        match fs::create_dir(to) {
            Err(e) if e.kind() == io::ErrorKind::AlreadyExists => {}
            r => r?,
        }
        #[cfg(not(unix))]
        let _ = root_to_inode;

        from.as_ref()
            .read_dir()?
            .par_bridge()
            .try_for_each(|dir_entry| -> io::Result<()> {
                let dir_entry = dir_entry?;

                #[cfg(unix)]
                {
                    use std::os::unix::fs::DirEntryExt;
                    if dir_entry.ino() == root_to_inode {
                        return Ok(());
                    }
                }

                let to = to.join(dir_entry.file_name());
                let file_type = dir_entry.file_type()?;
                let file_type = if follow_symlinks && file_type.is_symlink() {
                    fs::metadata(dir_entry.path())?.file_type()
                } else {
                    file_type
                };

                if file_type.is_dir() {
                    copy_dir(
                        dir_entry.path(),
                        to,
                        follow_symlinks,
                        hard_link,
                        root_to_inode,
                    )?;
                } else if file_type.is_symlink() {
                    let from = fs::read_link(dir_entry.path())?;
                    if hard_link {
                        fs::hard_link(dir_entry.path(), to)?;
                    } else {
                        #[cfg(unix)]
                        std::os::unix::fs::symlink(from, to)?;
                        #[cfg(windows)]
                        if fs::metadata(&from)?.file_type().is_dir() {
                            std::os::windows::fs::symlink_dir(from, to)?;
                        } else {
                            std::os::windows::fs::symlink_file(from, to)?;
                        }
                    }
                } else if hard_link {
                    fs::hard_link(dir_entry.path(), to)?;
                } else {
                    fs::copy(dir_entry.path(), to)?;
                }

                Ok(())
            })
    }
}
