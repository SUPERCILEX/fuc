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
        os::unix::{
            ffi::{OsStrExt, OsStringExt},
            io::AsFd,
        },
        path::{Path, PathBuf, MAIN_SEPARATOR},
    };

    use rustix::fs::{statx, AtFlags, FileType, StatxFlags};

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
            .map(|metadata| FileType::from_raw_mode(metadata.stx_mode.into()))
    }
}

mod compat {
    use crate::Error;

    pub trait DirectoryOp<T> {
        fn run(&self, dir: T) -> Result<(), Error>;

        fn finish(self) -> Result<(), Error>;
    }
}
