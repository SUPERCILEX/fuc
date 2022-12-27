use std::{
    borrow::Cow,
    cell::RefCell,
    ffi::{CStr, CString},
    fmt::Debug,
    fs, io,
    num::NonZeroUsize,
    path::Path,
    thread,
};

use crossbeam_channel::{Receiver, Sender};
use rustix::fs::{
    copy_file_range, cwd, mkdirat, openat, statx, AtFlags, FileType, Mode, OFlags, RawDir, RawMode,
    StatxFlags,
};
use typed_builder::TypedBuilder;

use crate::{
    ops::{concat_cstrs, path_buf_to_cstring, IoErr, LazyCell},
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
        let scheduling = LazyCell::new(|| {
            let (tx, rx) = crossbeam_channel::unbounded();
            (tx, thread::spawn(|| root_worker_thread(rx)))
        });

        let result = schedule_copies(self, &scheduling);

        if let Some((tasks, thread)) = scheduling.into_inner() {
            drop(tasks);
            thread.join().map_err(|_| Error::Join)??;
        }

        result
    }
}

fn schedule_copies<'a, L>(
    CopyOp { files, force }: CopyOp<'a, impl IntoIterator<Item = (Cow<'a, Path>, Cow<'a, Path>)>>,
    scheduling: &LazyCell<(Sender<TreeNode>, L), impl FnOnce() -> (Sender<TreeNode>, L)>,
) -> Result<(), Error> {
    for (from, to) in files {
        if !force {
            match to.metadata() {
                Ok(_) => return Err(Error::AlreadyExists),
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

        if is_dir {
            let (tasks, _) = &**scheduling;
            tasks
                .send(TreeNode {
                    from: path_buf_to_cstring(from.into_owned())?,
                    to: path_buf_to_cstring(to.into_owned())?,
                    messages: tasks.clone(),
                })
                .map_err(|_| Error::Internal)?;
        } else {
            fs::copy(&from, &to).map_io_err(|| format!("Failed to copy file: {from:?}"))?;
        }
    }
    Ok(())
}

#[allow(clippy::needless_pass_by_value)]
fn root_worker_thread(tasks: Receiver<TreeNode>) -> Result<(), Error> {
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

            copy_dir(&node)?;
        }

        for thread in threads {
            thread.join().map_err(|_| Error::Join)??;
        }
        Ok(())
    })
}

fn worker_thread(tasks: Receiver<TreeNode>) -> Result<(), Error> {
    for node in tasks {
        copy_dir(&node)?;
    }
    Ok(())
}

fn copy_dir(node: &TreeNode) -> Result<(), Error> {
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
        {
            const EMPTY: &CStr = CStr::from_bytes_with_nul(b"\0").ok().unwrap();

            let from_perms = statx(&from_dir, EMPTY, AtFlags::EMPTY_PATH, StatxFlags::MODE)
                .map_io_err(|| format!("Failed to stat directory: {:?}", node.from))?;
            mkdirat(
                cwd(),
                node.to.as_c_str(),
                Mode::from_raw_mode(RawMode::from(from_perms.stx_mode)),
            )
            .map_io_err(|| format!("Failed to create directory: {:?}", node.to))?;
        }
        let to_dir = openat(
            cwd(),
            node.to.as_c_str(),
            OFlags::RDONLY | OFlags::DIRECTORY | OFlags::PATH,
            Mode::empty(),
        )
        .map_io_err(|| format!("Failed to open directory: {:?}", node.to))?;

        let mut buf = buf.borrow_mut();
        let mut raw_dir = RawDir::new(&from_dir, buf.spare_capacity_mut());
        while let Some(file) = raw_dir.next() {
            const DOT: &CStr = CStr::from_bytes_with_nul(b".\0").ok().unwrap();
            const DOT_DOT: &CStr = CStr::from_bytes_with_nul(b"..\0").ok().unwrap();

            let file = file.map_io_err(|| format!("Failed to read directory: {:?}", node.from))?;
            if file.file_name() == DOT || file.file_name() == DOT_DOT {
                continue;
            }

            if file.file_type() == FileType::Directory {
                node.messages
                    .send(TreeNode {
                        from: concat_cstrs(&node.from, file.file_name()),
                        to: concat_cstrs(&node.to, file.file_name()),
                        messages: node.messages.clone(),
                    })
                    .map_err(|_| Error::Internal)?;
            } else {
                let from = openat(&from_dir, file.file_name(), OFlags::RDONLY, Mode::empty())
                    .map_io_err(|| {
                        format!(
                            "Failed to open file: {:?}/{:?}",
                            node.from,
                            file.file_name()
                        )
                    })?;
                let from_perms = statx(
                    &from_dir,
                    file.file_name(),
                    AtFlags::empty(),
                    StatxFlags::MODE,
                )
                .map_io_err(|| {
                    format!(
                        "Failed to stat file: {:?}/{:?}",
                        node.from,
                        file.file_name()
                    )
                })?;
                let to = openat(
                    &to_dir,
                    file.file_name(),
                    OFlags::CREATE | OFlags::TRUNC | OFlags::WRONLY,
                    Mode::from_raw_mode(RawMode::from(from_perms.stx_mode)),
                )
                .map_io_err(|| {
                    format!("Failed to open file: {:?}/{:?}", node.to, file.file_name())
                })?;

                copy_file_range(&from, None, &to, None, u64::MAX).map_io_err(|| {
                    format!(
                        "Failed to copy file: {:?}/{:?}",
                        node.from,
                        file.file_name()
                    )
                })?;
            }
        }
        Ok(())
    })
}

struct TreeNode {
    from: CString,
    to: CString,
    messages: Sender<TreeNode>,
}
