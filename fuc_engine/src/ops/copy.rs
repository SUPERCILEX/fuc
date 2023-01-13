use std::{borrow::Cow, fmt::Debug, fs, io, path::Path};

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
pub struct CopyOp<'a, F: IntoIterator<Item = (Cow<'a, Path>, Cow<'a, Path>)>> {
    files: F,
    #[builder(default = false)]
    force: bool,
}

impl<'a, F: IntoIterator<Item = (Cow<'a, Path>, Cow<'a, Path>)>> CopyOp<'a, F> {
    /// Consume and run this copy operation.
    ///
    /// # Errors
    ///
    /// Returns the underlying I/O errors that occurred.
    pub fn run(self) -> Result<(), Error> {
        let copy = compat::copy_impl();
        let result = schedule_copies(self, &copy);
        copy.finish().and(result)
    }
}

fn schedule_copies<'a>(
    CopyOp { files, force }: CopyOp<'a, impl IntoIterator<Item = (Cow<'a, Path>, Cow<'a, Path>)>>,
    copy: &impl DirectoryOp<(Cow<'a, Path>, Cow<'a, Path>)>,
) -> Result<(), Error> {
    for (from, to) in files {
        if !force {
            match to.metadata() {
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

        let is_dir = from
            .metadata()
            .map_io_err(|| format!("Failed to read metadata for file: {from:?}"))?
            .is_dir();

        if let Some(parent) = to.parent() {
            fs::create_dir_all(parent)
                .map_io_err(|| format!("Failed to create parent directory: {parent:?}"))?;
        }

        if is_dir {
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
        cell::Cell,
        ffi::{CStr, CString},
        fs::File,
        io,
        mem::MaybeUninit,
        num::NonZeroUsize,
        os::fd::{AsFd, OwnedFd},
        path::Path,
        thread,
        thread::JoinHandle,
    };

    use crossbeam_channel::{Receiver, Sender};
    use rustix::{
        fs::{
            copy_file_range, cwd, mkdirat, openat, readlinkat, statx, symlinkat, AtFlags, FileType,
            Mode, OFlags, RawDir, RawMode, StatxFlags,
        },
        io::Errno,
    };

    use crate::{
        ops::{
            compat::DirectoryOp, concat_cstrs, join_cstr_paths, path_buf_to_cstring, IoErr,
            LazyCell,
        },
        Error,
    };

    struct Impl<LF: FnOnce() -> (Sender<TreeNode>, JoinHandle<Result<(), Error>>)> {
        #[allow(clippy::type_complexity)]
        scheduling: LazyCell<(Sender<TreeNode>, JoinHandle<Result<(), Error>>), LF>,
    }

    pub fn copy_impl<'a>() -> impl DirectoryOp<(Cow<'a, Path>, Cow<'a, Path>)> {
        let scheduling = LazyCell::new(|| {
            let (tx, rx) = crossbeam_channel::unbounded();
            (tx, thread::spawn(|| root_worker_thread(rx)))
        });

        Impl { scheduling }
    }

    impl<LF: FnOnce() -> (Sender<TreeNode>, JoinHandle<Result<(), Error>>)>
        DirectoryOp<(Cow<'_, Path>, Cow<'_, Path>)> for Impl<LF>
    {
        fn run(&self, (from, to): (Cow<Path>, Cow<Path>)) -> Result<(), Error> {
            let (tasks, _) = &*self.scheduling;
            tasks
                .send(TreeNode {
                    from: path_buf_to_cstring(from.into_owned())?,
                    to: path_buf_to_cstring(to.into_owned())?,
                    messages: tasks.clone(),
                    root_to_inode: None,
                })
                .map_err(|_| Error::Internal)
        }

        fn finish(self) -> Result<(), Error> {
            if let Some((tasks, thread)) = self.scheduling.into_inner() {
                drop(tasks);
                thread.join().map_err(|_| Error::Join)??;
            }
            Ok(())
        }
    }

    #[allow(clippy::needless_pass_by_value)]
    fn root_worker_thread(tasks: Receiver<TreeNode>) -> Result<(), Error> {
        let mut available_parallelism = thread::available_parallelism()
            .map(NonZeroUsize::get)
            .unwrap_or(1)
            - 1;

        thread::scope(|scope| {
            let mut threads = Vec::with_capacity(available_parallelism);

            {
                let mut buf = [MaybeUninit::<u8>::uninit(); 8192];
                let symlink_buf_cache = Cell::new(Vec::new());
                for node in &tasks {
                    if available_parallelism > 0 {
                        available_parallelism -= 1;
                        threads.push(scope.spawn({
                            let tasks = tasks.clone();
                            || worker_thread(tasks)
                        }));
                    }

                    copy_dir(node, &mut buf, &symlink_buf_cache)?;
                }
            }

            for thread in threads {
                thread.join().map_err(|_| Error::Join)??;
            }
            Ok(())
        })
    }

    fn worker_thread(tasks: Receiver<TreeNode>) -> Result<(), Error> {
        let mut buf = [MaybeUninit::<u8>::uninit(); 8192];
        let symlink_buf_cache = Cell::new(Vec::new());
        for node in tasks {
            copy_dir(node, &mut buf, &symlink_buf_cache)?;
        }
        Ok(())
    }

    fn copy_dir(
        mut node: TreeNode,
        buf: &mut [MaybeUninit<u8>],
        symlink_buf_cache: &Cell<Vec<u8>>,
    ) -> Result<(), Error> {
        let from_dir = openat(
            cwd(),
            node.from.as_c_str(),
            OFlags::RDONLY | OFlags::DIRECTORY,
            Mode::empty(),
        )
        .map_io_err(|| format!("Failed to open directory: {:?}", node.from))?;
        let to_dir = copy_one_dir(&from_dir, &node)?;
        let root_to_inode = maybe_compute_root_to_inode(&to_dir, &mut node)?;

        let mut raw_dir = RawDir::new(&from_dir, buf);
        while let Some(file) = raw_dir.next() {
            const DOT: &CStr = CStr::from_bytes_with_nul(b".\0").ok().unwrap();
            const DOT_DOT: &CStr = CStr::from_bytes_with_nul(b"..\0").ok().unwrap();

            let file = file.map_io_err(|| format!("Failed to read directory: {:?}", node.from))?;
            if file.file_name() == DOT || file.file_name() == DOT_DOT {
                continue;
            }
            if file.ino() == root_to_inode {
                // Block recursive descent from parent into child (e.g. cp parent parent/child).
                continue;
            }

            let file_type = file.file_type();
            if file_type == FileType::Directory {
                node.messages
                    .send(TreeNode {
                        from: concat_cstrs(&node.from, file.file_name()),
                        to: concat_cstrs(&node.to, file.file_name()),
                        messages: node.messages.clone(),
                        root_to_inode: node.root_to_inode,
                    })
                    .map_err(|_| Error::Internal)?;
            } else {
                copy_one_file(
                    &from_dir,
                    &to_dir,
                    file.file_name(),
                    file_type,
                    &node,
                    symlink_buf_cache,
                )?;
            }
        }
        Ok(())
    }

    fn copy_one_dir(from_dir: impl AsFd, node: &TreeNode) -> Result<OwnedFd, Error> {
        const EMPTY: &CStr = CStr::from_bytes_with_nul(b"\0").ok().unwrap();

        let from_mode = {
            let from_metadata = statx(from_dir, EMPTY, AtFlags::EMPTY_PATH, StatxFlags::MODE)
                .map_io_err(|| format!("Failed to stat directory: {:?}", node.from))?;
            Mode::from_raw_mode(RawMode::from(from_metadata.stx_mode))
        };
        mkdirat(cwd(), node.to.as_c_str(), from_mode)
            .map_io_err(|| format!("Failed to create directory: {:?}", node.to))?;
        openat(
            cwd(),
            node.to.as_c_str(),
            OFlags::RDONLY | OFlags::DIRECTORY | OFlags::PATH,
            Mode::empty(),
        )
        .map_io_err(|| format!("Failed to open directory: {:?}", node.to))
    }

    fn copy_one_file(
        from_dir: impl AsFd,
        to_dir: impl AsFd,
        file_name: &CStr,
        file_type: FileType,
        node: &TreeNode,
        symlink_buf_cache: &Cell<Vec<u8>>,
    ) -> Result<(), Error> {
        match file_type {
            FileType::RegularFile => copy_regular_file(from_dir, to_dir, file_name, node),
            FileType::Symlink => copy_symlink(from_dir, to_dir, file_name, node, symlink_buf_cache),
            _ => copy_any_file(from_dir, to_dir, file_name, node),
        }
    }

    fn copy_regular_file(
        from_dir: impl AsFd,
        to_dir: impl AsFd,
        file_name: &CStr,
        node: &TreeNode,
    ) -> Result<(), Error> {
        let (from, to) = prep_regular_file(from_dir, to_dir, file_name, node)?;
        let mut total_copied = 0;
        loop {
            let byte_copied = match copy_file_range(&from, None, &to, None, u64::MAX - total_copied)
            {
                Err(Errno::XDEV) if total_copied == 0 => {
                    return _copy_any_file(from, to, file_name, node);
                }
                r => r.map_io_err(|| {
                    format!(
                        "Failed to copy file: {:?}",
                        join_cstr_paths(&node.from, file_name)
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
    fn copy_any_file(
        from_dir: impl AsFd,
        to_dir: impl AsFd,
        file_name: &CStr,
        node: &TreeNode,
    ) -> Result<(), Error> {
        let (from, to) = prep_regular_file(from_dir, to_dir, file_name, node)?;
        _copy_any_file(from, to, file_name, node)
    }

    #[cold]
    fn _copy_any_file(
        from: OwnedFd,
        to: OwnedFd,
        file_name: &CStr,
        node: &TreeNode,
    ) -> Result<(), Error> {
        io::copy(&mut File::from(from), &mut File::from(to))
            .map_io_err(|| {
                format!(
                    "Failed to copy file: {:?}",
                    join_cstr_paths(&node.from, file_name)
                )
            })
            .map(|_| ())
    }

    fn prep_regular_file(
        from_dir: impl AsFd,
        to_dir: impl AsFd,
        file_name: &CStr,
        node: &TreeNode,
    ) -> Result<(OwnedFd, OwnedFd), Error> {
        let from =
            openat(&from_dir, file_name, OFlags::RDONLY, Mode::empty()).map_io_err(|| {
                format!(
                    "Failed to open file: {:?}",
                    join_cstr_paths(&node.from, file_name)
                )
            })?;
        let from_mode = {
            let from_metadata = statx(from_dir, file_name, AtFlags::empty(), StatxFlags::MODE)
                .map_io_err(|| {
                    format!(
                        "Failed to stat file: {:?}",
                        join_cstr_paths(&node.from, file_name)
                    )
                })?;
            Mode::from_raw_mode(RawMode::from(from_metadata.stx_mode))
        };

        let to = openat(
            &to_dir,
            file_name,
            OFlags::CREATE | OFlags::TRUNC | OFlags::WRONLY,
            from_mode,
        )
        .map_io_err(|| {
            format!(
                "Failed to open file: {:?}",
                join_cstr_paths(&node.to, file_name)
            )
        })?;

        Ok((from, to))
    }

    #[cold]
    fn copy_symlink(
        from_dir: impl AsFd,
        to_dir: impl AsFd,
        file_name: &CStr,
        node: &TreeNode,
        symlink_buf_cache: &Cell<Vec<u8>>,
    ) -> Result<(), Error> {
        let from_symlink =
            readlinkat(from_dir, file_name, symlink_buf_cache.take()).map_io_err(|| {
                format!(
                    "Failed to read symlink: {:?}",
                    join_cstr_paths(&node.from, file_name)
                )
            })?;

        symlinkat(from_symlink.as_c_str(), &to_dir, file_name).map_io_err(|| {
            format!(
                "Failed to create symlink: {:?}",
                join_cstr_paths(&node.to, file_name)
            )
        })?;

        symlink_buf_cache.set(from_symlink.into_bytes_with_nul());
        Ok(())
    }

    fn maybe_compute_root_to_inode(to_dir: impl AsFd, node: &mut TreeNode) -> Result<u64, Error> {
        Ok(if let Some(ino) = node.root_to_inode {
            ino
        } else {
            const EMPTY: &CStr = CStr::from_bytes_with_nul(b"\0").ok().unwrap();

            let to_stat = statx(to_dir, EMPTY, AtFlags::EMPTY_PATH, StatxFlags::INO)
                .map_io_err(|| format!("Failed to stat directory: {:?}", node.to))?;
            node.root_to_inode = Some(to_stat.stx_ino);
            to_stat.stx_ino
        })
    }

    struct TreeNode {
        from: CString,
        to: CString,
        messages: Sender<TreeNode>,
        root_to_inode: Option<u64>,
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

    struct Impl;

    pub fn copy_impl<'a>() -> impl DirectoryOp<(Cow<'a, Path>, Cow<'a, Path>)> {
        Impl
    }

    impl DirectoryOp<(Cow<'_, Path>, Cow<'_, Path>)> for Impl {
        fn run(&self, (from, to): (Cow<Path>, Cow<Path>)) -> Result<(), Error> {
            copy_dir(
                &from,
                to,
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
        #[cfg(unix)] root_to_inode: Option<u64>,
    ) -> Result<(), io::Error> {
        let to = to.as_ref();
        fs::create_dir(to)?;
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
                let file_type = dir_entry.file_type()?;

                #[cfg(unix)]
                if file_type.is_dir() {
                    copy_dir(dir_entry.path(), to, root_to_inode)?;
                } else if file_type.is_symlink() {
                    std::os::unix::fs::symlink(dir_entry.path(), to)?;
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
