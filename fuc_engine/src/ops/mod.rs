use std::io;

pub use copy::{copy_file, CopyOp};
#[cfg(target_os = "linux")]
use linux::{concat_cstrs, get_file_type, join_cstr_paths, path_buf_to_cstring};
pub use remove::{remove_file, RemoveOp};

use crate::Error;

mod copy;
mod remove;

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

#[cfg(target_os = "linux")]
mod linux {
    use std::{
        ffi::{CStr, CString, OsStr, OsString},
        io,
        os::{
            fd::AsFd,
            unix::ffi::{OsStrExt, OsStringExt},
        },
        path::{Path, PathBuf, MAIN_SEPARATOR},
    };

    use rustix::fs::{statx, AtFlags, FileType, RawMode, StatxFlags};

    use crate::{ops::IoErr, Error};

    impl<T> IoErr<Result<T, Error>> for Result<T, rustix::io::Errno> {
        fn map_io_err(self, context: impl FnOnce() -> String) -> Result<T, Error> {
            self.map_err(io::Error::from).map_io_err(context)
        }
    }

    pub fn path_buf_to_cstring(buf: PathBuf) -> Result<CString, Error> {
        CString::new(OsString::from(buf).into_vec()).map_err(|_| Error::BadPath)
    }

    pub fn concat_cstrs(prefix: &CString, name: &CStr) -> CString {
        let prefix = prefix.as_bytes();
        let name = name.to_bytes_with_nul();

        let mut path = Vec::with_capacity(prefix.len() + 1 + name.len());
        path.extend_from_slice(prefix);
        path.push(u8::try_from(MAIN_SEPARATOR).unwrap());
        path.extend_from_slice(name);
        unsafe { CString::from_vec_with_nul_unchecked(path) }
    }

    pub fn join_cstr_paths(path: &CString, name: &CStr) -> PathBuf {
        Path::new(OsStr::from_bytes(path.as_bytes()))
            .join(Path::new(OsStr::from_bytes(name.to_bytes())))
    }

    #[cold]
    pub fn get_file_type(
        dir: impl AsFd,
        file_name: &CStr,
        path: &CString,
    ) -> Result<FileType, Error> {
        statx(dir, file_name, AtFlags::SYMLINK_NOFOLLOW, StatxFlags::TYPE)
            .map_io_err(|| {
                format!(
                    "Failed to stat file: {:?}",
                    join_cstr_paths(path, file_name)
                )
            })
            .map(|metadata| FileType::from_raw_mode(RawMode::from(metadata.stx_mode)))
    }
}

mod compat {
    use crate::Error;

    pub trait DirectoryOp<T> {
        fn run(&self, dir: T) -> Result<(), Error>;

        fn finish(self) -> Result<(), Error>;
    }
}

// TODO remove: https://github.com/rust-lang/rust/issues/74465#issuecomment-1364969188
#[cfg(target_os = "linux")]
struct LazyCell<T, F = fn() -> T> {
    cell: std::cell::OnceCell<T>,
    init: std::cell::Cell<Option<F>>,
}

#[cfg(target_os = "linux")]
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
#[cfg(target_os = "linux")]
impl<T, F: FnOnce() -> T> LazyCell<T, F> {
    fn force(this: &Self) -> &T {
        this.cell.get_or_init(|| match this.init.take() {
            Some(f) => f(),
            None => panic!("`Lazy` instance has previously been poisoned"),
        })
    }
}

#[cfg(target_os = "linux")]
impl<T, F: FnOnce() -> T> std::ops::Deref for LazyCell<T, F> {
    type Target = T;
    fn deref(&self) -> &T {
        Self::force(self)
    }
}
