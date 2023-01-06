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
        cell::RefCell,
        ffi::{CStr, CString},
        num::NonZeroUsize,
        os::fd::{AsFd, OwnedFd},
        path::Path,
        thread,
        thread::JoinHandle,
    };

    use crossbeam_channel::{Receiver, Sender};
    use rustix::fs::{
        copy_file_range, cwd, mkdirat, openat, statx, AtFlags, FileType, Mode, OFlags, RawDir,
        RawMode, StatxFlags,
    };

    use crate::{
        ops::{
            compat::DirectoryOp, concat_cstrs, join_cstr_paths, path_buf_to_cstring, IoErr,
            LazyCell,
        },
        Error,
    };

    struct Impl<LF: FnOnce() -> (Sender<Message>, JoinHandle<Result<(), Error>>)> {
        #[allow(clippy::type_complexity)]
        scheduling: LazyCell<(Sender<Message>, JoinHandle<Result<(), Error>>), LF>,
    }

    pub fn copy_impl<'a>() -> impl DirectoryOp<(Cow<'a, Path>, Cow<'a, Path>)> {
        let scheduling = LazyCell::new(|| {
            let (tx, rx) = crossbeam_channel::unbounded();
            (tx, thread::spawn(|| root_worker_thread(rx)))
        });

        Impl { scheduling }
    }

    impl<LF: FnOnce() -> (Sender<Message>, JoinHandle<Result<(), Error>>)>
        DirectoryOp<(Cow<'_, Path>, Cow<'_, Path>)> for Impl<LF>
    {
        fn run(&self, (from, to): (Cow<Path>, Cow<Path>)) -> Result<(), Error> {
            let (tasks, _) = &*self.scheduling;
            tasks
                .send(Message::Node(TreeNode {
                    from: path_buf_to_cstring(from.into_owned())?,
                    to: path_buf_to_cstring(to.into_owned())?,
                    messages: tasks.clone(),
                    root_to_inode: None,
                }))
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
    fn root_worker_thread(tasks: Receiver<Message>) -> Result<(), Error> {
        let mut available_parallelism = thread::available_parallelism()
            .map(NonZeroUsize::get)
            .unwrap_or(1)
            - 1;

        thread::scope(|scope| {
            let mut threads = Vec::with_capacity(available_parallelism);

            for message in &tasks {
                if available_parallelism > 0 {
                    available_parallelism -= 1;
                    threads.push(scope.spawn({
                        let tasks = tasks.clone();
                        || worker_thread(tasks)
                    }));
                }

                match message {
                    Message::Node(node) => copy_dir(node)?,
                    Message::Copy(copy) => perform_actual_copy(copy)?,
                }
            }

            for thread in threads {
                thread.join().map_err(|_| Error::Join)??;
            }
            Ok(())
        })
    }

    fn worker_thread(tasks: Receiver<Message>) -> Result<(), Error> {
        for message in tasks {
            match message {
                Message::Node(node) => copy_dir(node)?,
                Message::Copy(copy) => perform_actual_copy(copy)?,
            }
        }
        Ok(())
    }

    fn copy_dir(mut node: TreeNode) -> Result<(), Error> {
        thread_local! {
            static BUF: RefCell<Vec<u8>> = RefCell::new(Vec::with_capacity(8192));
        }

        BUF.with(|buf| {
            let from_dir = openat(
                cwd(),
                node.from.as_c_str(),
                OFlags::RDONLY | OFlags::DIRECTORY,
                Mode::empty(),
            )
            .map_io_err(|| format!("Failed to open directory: {:?}", node.from))?;
            let to_dir = copy_one_dir(&from_dir, &node)?;
            let root_to_inode = maybe_compute_root_to_inode(&to_dir, &mut node)?;

            let mut buf = buf.borrow_mut();
            let mut raw_dir = RawDir::new(&from_dir, buf.spare_capacity_mut());
            while let Some(file) = raw_dir.next() {
                const DOT: &CStr = CStr::from_bytes_with_nul(b".\0").ok().unwrap();
                const DOT_DOT: &CStr = CStr::from_bytes_with_nul(b"..\0").ok().unwrap();

                let file =
                    file.map_io_err(|| format!("Failed to read directory: {:?}", node.from))?;
                if file.file_name() == DOT || file.file_name() == DOT_DOT {
                    continue;
                }
                if file.ino() == root_to_inode {
                    // Block recursive descent from parent into child (e.g. cp parent parent/child).
                    continue;
                }

                if file.file_type() == FileType::Directory {
                    node.messages
                        .send(Message::Node(TreeNode {
                            from: concat_cstrs(&node.from, file.file_name()),
                            to: concat_cstrs(&node.to, file.file_name()),
                            messages: node.messages.clone(),
                            root_to_inode: node.root_to_inode,
                        }))
                        .map_err(|_| Error::Internal)?;
                } else {
                    copy_one_file(&from_dir, &to_dir, file.file_name(), &node)?;
                }
            }
            Ok(())
        })
    }

    fn copy_one_dir(from_dir: impl AsFd, node: &TreeNode) -> Result<OwnedFd, Error> {
        const EMPTY: &CStr = CStr::from_bytes_with_nul(b"\0").ok().unwrap();

        let from_perms = statx(from_dir, EMPTY, AtFlags::EMPTY_PATH, StatxFlags::MODE)
            .map_io_err(|| format!("Failed to stat directory: {:?}", node.from))?;
        mkdirat(
            cwd(),
            node.to.as_c_str(),
            Mode::from_raw_mode(RawMode::from(from_perms.stx_mode)),
        )
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
        node: &TreeNode,
    ) -> Result<(), Error> {
        let from =
            openat(&from_dir, file_name, OFlags::RDONLY, Mode::empty()).map_io_err(|| {
                format!(
                    "Failed to open file: {:?}",
                    join_cstr_paths(&node.from, file_name)
                )
            })?;
        let from_perms = statx(from_dir, file_name, AtFlags::empty(), StatxFlags::MODE)
            .map_io_err(|| {
                format!(
                    "Failed to stat file: {:?}",
                    join_cstr_paths(&node.from, file_name)
                )
            })?;
        let to = openat(
            &to_dir,
            file_name,
            OFlags::CREATE | OFlags::TRUNC | OFlags::WRONLY,
            Mode::from_raw_mode(RawMode::from(from_perms.stx_mode)),
        )
        .map_io_err(|| {
            format!(
                "Failed to open file: {:?}",
                join_cstr_paths(&node.to, file_name)
            )
        })?;

        node.messages
            .send(Message::Copy(Copy { from, to }))
            .map_err(|_| Error::Internal)
    }

    fn perform_actual_copy(copy: Copy) -> Result<(), Error> {
        copy_file_range(copy.from, None, copy.to, None, u64::MAX)
            .map_io_err(|| "Failed to copy file".to_string())
            .map(|_| ())
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

    enum Message {
        Node(TreeNode),
        Copy(Copy),
    }

    struct TreeNode {
        from: CString,
        to: CString,
        messages: Sender<Message>,
        root_to_inode: Option<u64>,
    }

    struct Copy {
        from: OwnedFd,
        to: OwnedFd,
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
                if dir_entry.file_type()?.is_dir() {
                    copy_dir(
                        dir_entry.path(),
                        to,
                        #[cfg(unix)]
                        root_to_inode,
                    )?;
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
