use std::{borrow::Cow, fmt::Debug, fs, io, marker::PhantomData, path::Path};

use typed_builder::TypedBuilder;

use crate::{
    ops::{compat::DirectoryOp, IoErr},
    Error,
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

#[derive(TypedBuilder, Debug)]
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
    #[allow(dead_code)]
    dereference: bool,
    #[builder(default)]
    _marker1: PhantomData<&'a I1>,
    #[builder(default)]
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
        let copy = compat::copy_impl(
            #[cfg(unix)]
            self.dereference,
        );
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
        dereference: _,
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

        let from_metadata = from
            .symlink_metadata()
            .map_io_err(|| format!("Failed to read metadata for file: {from:?}"))?;

        if let Some(parent) = to.parent() {
            fs::create_dir_all(parent)
                .map_io_err(|| format!("Failed to create parent directory: {parent:?}"))?;
        }

        #[cfg(unix)]
        if from_metadata.is_dir() {
            use std::os::unix::fs::{DirBuilderExt, MetadataExt};
            match fs::DirBuilder::new()
                .mode(
                    fs::symlink_metadata(&from)
                        .map_io_err(|| format!("Failed to stat directory: {from:?}"))?
                        .mode(),
                )
                .create(&to)
            {
                Err(e) if force && e.kind() == io::ErrorKind::AlreadyExists => {}
                r => r.map_io_err(|| format!("Failed to create directory: {to:?}"))?,
            };
            copy.run((from, to))?;
        } else if from_metadata.is_symlink() {
            let link =
                fs::read_link(&from).map_io_err(|| format!("Failed to read symlink: {from:?}"))?;
            std::os::unix::fs::symlink(link, &to)
                .map_io_err(|| format!("Failed to create symlink: {to:?}"))?;
        } else {
            fs::copy(&from, &to).map_io_err(|| format!("Failed to copy file: {from:?}"))?;
        }

        #[cfg(not(unix))]
        if from_metadata.is_dir() {
            copy.run((from, to))?;
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
            copy_file_range, mkdirat, openat, readlinkat, statx, symlinkat, AtFlags, FileType,
            Mode, OFlags, RawDir, StatxFlags, CWD,
        },
        io::Errno,
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

    pub fn copy_impl<'a, 'b>(
        dereference: bool,
    ) -> impl DirectoryOp<(Cow<'a, Path>, Cow<'b, Path>)> {
        let scheduling = LazyCell::new(move || {
            let (tx, rx) = crossbeam_channel::unbounded();
            (
                tx,
                thread::spawn(move || root_worker_thread(rx, dereference)),
            )
        });

        Impl { scheduling }
    }

    impl<LF: FnOnce() -> (Sender<TreeNode>, JoinHandle<Result<(), Error>>)>
        DirectoryOp<(Cow<'_, Path>, Cow<'_, Path>)> for Impl<LF>
    {
        #[cfg_attr(feature = "tracing", tracing::instrument(level = "trace", skip(self)))]
        fn run(&self, (from, to): (Cow<Path>, Cow<Path>)) -> Result<(), Error> {
            let (tasks, _) = &*self.scheduling;
            tasks
                .send(TreeNode {
                    from: path_buf_to_cstring(from.into_owned())?,
                    to: path_buf_to_cstring(to.into_owned())?,
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
    fn root_worker_thread(tasks: Receiver<TreeNode>, dereference: bool) -> Result<(), Error> {
        let mut available_parallelism = thread::available_parallelism()
            .map(NonZeroUsize::get)
            .unwrap_or(1)
            - 1;

        thread::scope(|scope| {
            let mut threads = Vec::with_capacity(available_parallelism);

            {
                let mut root_to_inode = None;
                let mut buf = [MaybeUninit::<u8>::uninit(); 8192];
                let symlink_buf_cache = Cell::new(Vec::new());
                for node in &tasks {
                    let root_to_inode = if let Some(root_to_inode) = root_to_inode {
                        root_to_inode
                    } else {
                        let to_dir = openat(
                            CWD,
                            &node.to,
                            OFlags::RDONLY | OFlags::DIRECTORY | OFlags::PATH,
                            Mode::empty(),
                        )
                        .map_io_err(|| format!("Failed to open directory: {:?}", node.to))?;
                        let to_metadata = statx(to_dir, c"", AtFlags::EMPTY_PATH, StatxFlags::INO)
                            .map_io_err(|| format!("Failed to stat directory: {:?}", node.to))?;
                        root_to_inode = Some(to_metadata.stx_ino);
                        to_metadata.stx_ino
                    };

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
                                move || worker_thread(tasks, root_to_inode, dereference)
                            }));
                        }
                    };
                    maybe_spawn();

                    copy_dir(
                        node,
                        root_to_inode,
                        dereference,
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
    fn worker_thread(
        tasks: Receiver<TreeNode>,
        root_to_inode: u64,
        dereference: bool,
    ) -> Result<(), Error> {
        unshare(UnshareFlags::FILES).map_io_err(|| "Failed to unshare FD table.")?;

        let mut buf = [MaybeUninit::<u8>::uninit(); 8192];
        let symlink_buf_cache = Cell::new(Vec::new());
        for node in tasks {
            copy_dir(
                node,
                root_to_inode,
                dereference,
                &mut buf,
                &symlink_buf_cache,
                || {},
            )?;
        }
        Ok(())
    }

    #[cfg_attr(
        feature = "tracing",
        tracing::instrument(level = "info", skip(messages, buf, symlink_buf_cache, maybe_spawn))
    )]
    fn copy_dir(
        TreeNode { from, to, messages }: TreeNode,
        root_to_inode: u64,
        dereference: bool,
        buf: &mut [MaybeUninit<u8>],
        symlink_buf_cache: &Cell<Vec<u8>>,
        mut maybe_spawn: impl FnMut(),
    ) -> Result<(), Error> {
        let from_dir = openat(
            CWD,
            &from,
            OFlags::RDONLY
                | OFlags::DIRECTORY
                | if dereference {
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
            if file_type == FileType::Unknown || (dereference && file_type == FileType::Symlink) {
                file_type = get_file_type(&from_dir, file.file_name(), &from, dereference)?;
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
                        messages: messages.clone(),
                    })
                    .map_err(|_| Error::Internal)?;
            } else {
                copy_one_file(
                    &from_dir,
                    &to_dir,
                    file.file_name(),
                    file_type,
                    &from,
                    &to,
                    symlink_buf_cache,
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
        };

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
            let (from, to) = prep_regular_file(from_dir, to_dir, file_name, from_path, to_path)?;
            if file_type == FileType::RegularFile {
                copy_regular_file(from, to, file_name, from_path)
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
    ) -> Result<(), Error> {
        let mut total_copied = 0;
        loop {
            let byte_copied =
                match copy_file_range(&from, None, &to, None, usize::MAX / 2 - total_copied) {
                    Err(Errno::XDEV) if total_copied == 0 => {
                        return copy_any_file(from, to, file_name, from_path);
                    }
                    r => r.map_io_err(|| {
                        format!(
                            "Failed to copy file: {:?}",
                            join_cstr_paths(from_path, file_name)
                        )
                    })?,
                };

            if byte_copied == 0 {
                return Ok(());
            }
            total_copied += byte_copied;
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
    ) -> Result<(OwnedFd, OwnedFd), Error> {
        let from =
            openat(&from_dir, file_name, OFlags::RDONLY, Mode::empty()).map_io_err(|| {
                format!(
                    "Failed to open file: {:?}",
                    join_cstr_paths(from_path, file_name)
                )
            })?;

        let to = {
            let from_mode = {
                let from_metadata = statx(from_dir, file_name, AtFlags::empty(), StatxFlags::MODE)
                    .map_io_err(|| {
                        format!(
                            "Failed to stat file: {:?}",
                            join_cstr_paths(from_path, file_name)
                        )
                    })?;
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

        Ok((from, to))
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
                "Failed to create symlink: {:?}",
                join_cstr_paths(to_path, file_name)
            )
        })?;

        symlink_buf_cache.set(from_symlink.into_bytes_with_nul());
        Ok(())
    }

    struct TreeNode {
        from: CString,
        to: CString,
        messages: Sender<TreeNode>,
    }

    impl Debug for TreeNode {
        fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
            f.debug_struct("TreeNode")
                .field("from", &self.from)
                .field("to", &self.to)
                .finish_non_exhaustive()
        }
    }
}

#[cfg(not(target_os = "linux"))]
mod compat {
    use std::{borrow::Cow, fs, io, path::Path};

    use rayon::prelude::*;

    use crate::{
        ops::{compat::DirectoryOp, IoErr},
        Error,
    };

    struct Impl {
        #[cfg(unix)]
        dereference: bool,
    }

    pub fn copy_impl<'a, 'b>(
        #[cfg(unix)] dereference: bool,
    ) -> impl DirectoryOp<(Cow<'a, Path>, Cow<'b, Path>)> {
        Impl {
            #[cfg(unix)]
            dereference,
        }
    }

    impl DirectoryOp<(Cow<'_, Path>, Cow<'_, Path>)> for Impl {
        fn run(&self, (from, to): (Cow<Path>, Cow<Path>)) -> Result<(), Error> {
            copy_dir(
                &from,
                to,
                #[cfg(unix)]
                self.dereference,
                #[cfg(unix)]
                None,
            )
            .map_io_err(|| format!("Failed to copy directory: {from:?}"))
        }

        fn finish(self) -> Result<(), Error> {
            Ok(())
        }
    }

    fn copy_dir<P: AsRef<Path>, Q: AsRef<Path>>(
        from: P,
        to: Q,
        #[cfg(unix)] dereference: bool,
        #[cfg(unix)] root_to_inode: Option<u64>,
    ) -> Result<(), io::Error> {
        let to = to.as_ref();
        match fs::create_dir(to) {
            Err(e) if e.kind() == io::ErrorKind::AlreadyExists => {}
            r => r?,
        };
        #[cfg(unix)]
        let root_to_inode = Some(maybe_compute_root_to_inode(to, root_to_inode)?);

        from.as_ref()
            .read_dir()?
            .par_bridge()
            .try_for_each(|dir_entry| -> io::Result<()> {
                let dir_entry = dir_entry?;

                #[cfg(unix)]
                {
                    use std::os::unix::fs::DirEntryExt;
                    if Some(dir_entry.ino()) == root_to_inode {
                        return Ok(());
                    }
                }

                let to = to.join(dir_entry.file_name());
                let file_type = {
                    #[cfg(unix)]
                    {
                        let mut file_type = dir_entry.file_type()?;
                        if dereference && file_type.is_symlink() {
                            file_type = fs::metadata(dir_entry.path())?.file_type();
                        }
                        file_type
                    }
                    #[cfg(not(unix))]
                    {
                        dir_entry.file_type()?
                    }
                };

                #[cfg(unix)]
                if file_type.is_dir() {
                    copy_dir(dir_entry.path(), to, dereference, root_to_inode)?;
                } else if file_type.is_symlink() {
                    std::os::unix::fs::symlink(fs::read_link(dir_entry.path())?, to)?;
                } else {
                    fs::copy(dir_entry.path(), to)?;
                }

                #[cfg(not(unix))]
                if file_type.is_dir() {
                    copy_dir(dir_entry.path(), to)?;
                } else {
                    fs::copy(dir_entry.path(), to)?;
                }

                Ok(())
            })
    }

    #[cfg(unix)]
    fn maybe_compute_root_to_inode<P: AsRef<Path>>(
        to: P,
        root_to_inode: Option<u64>,
    ) -> Result<u64, io::Error> {
        Ok(if let Some(ino) = root_to_inode {
            ino
        } else {
            use std::os::unix::fs::MetadataExt;
            fs::metadata(to)?.ino()
        })
    }
}
