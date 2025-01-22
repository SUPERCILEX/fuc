use std::{borrow::Cow, io};

pub use copy::{CopyOp, copy_file};
#[cfg(target_os = "linux")]
use linux::{concat_cstrs, join_cstr_paths, path_buf_to_cstring};
pub use remove::{RemoveOp, remove_file};

use crate::Error;

mod copy;
mod remove;

trait IoErr<Out> {
    fn map_io_err<I: Into<Cow<'static, str>>>(self, f: impl FnOnce() -> I) -> Out;
}

impl<T> IoErr<Result<T, Error>> for Result<T, io::Error> {
    fn map_io_err<I: Into<Cow<'static, str>>>(
        self,
        context: impl FnOnce() -> I,
    ) -> Result<T, Error> {
        self.map_err(|error| Error::Io {
            error,
            context: context().into(),
        })
    }
}

#[cfg(target_os = "linux")]
mod linux {
    use std::{
        borrow::Cow,
        ffi::{CStr, CString, OsStr, OsString},
        io,
        os::unix::ffi::{OsStrExt, OsStringExt},
        path::{MAIN_SEPARATOR, Path, PathBuf},
    };

    use crate::{Error, ops::IoErr};

    impl<T> IoErr<Result<T, Error>> for Result<T, rustix::io::Errno> {
        fn map_io_err<I: Into<Cow<'static, str>>>(
            self,
            context: impl FnOnce() -> I,
        ) -> Result<T, Error> {
            self.map_err(io::Error::from).map_io_err(context)
        }
    }

    #[cfg_attr(feature = "tracing", tracing::instrument(level = "trace"))]
    pub fn path_buf_to_cstring(buf: PathBuf) -> Result<CString, Error> {
        CString::new(OsString::from(buf).into_vec()).map_err(|_| Error::BadPath)
    }

    #[cfg_attr(feature = "tracing", tracing::instrument(level = "trace"))]
    pub fn concat_cstrs(prefix: &CString, name: &CStr) -> CString {
        let prefix = prefix.as_bytes();
        let name = name.to_bytes_with_nul();

        let mut path = Vec::with_capacity(prefix.len() + 1 + name.len());
        path.extend_from_slice(prefix);
        path.push(u8::try_from(MAIN_SEPARATOR).unwrap());
        path.extend_from_slice(name);
        unsafe { CString::from_vec_with_nul_unchecked(path) }
    }

    #[cfg_attr(feature = "tracing", tracing::instrument(level = "trace"))]
    pub fn join_cstr_paths(path: &CString, name: &CStr) -> PathBuf {
        Path::new(OsStr::from_bytes(path.as_bytes()))
            .join(Path::new(OsStr::from_bytes(name.to_bytes())))
    }
}

mod compat {
    use crate::Error;

    pub trait DirectoryOp<T> {
        fn run(&self, dir: T) -> Result<(), Error>;

        fn finish(self) -> Result<(), Error>;
    }
}
