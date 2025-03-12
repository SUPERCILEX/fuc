#![allow(clippy::needless_pass_by_value)]

use std::{borrow::Cow, io, path::PathBuf};

use thiserror::Error;

pub use crate::ops::{CopyOp, RemoveOp, copy_file, remove_file, remove_file as remove_dir_all};

mod ops;

#[derive(Error, Debug)]
pub enum Error {
    #[error("An I/O error occurred")]
    Io {
        error: io::Error,
        context: Cow<'static, str>,
    },
    #[error("An attempt was made to delete `/`")]
    PreserveRoot,
    #[error("Failed to join thread")]
    Join,
    #[error("Invalid file path")]
    BadPath,
    #[error("File or directory already exists: {file:?}")]
    AlreadyExists { file: PathBuf },
    #[error("File or directory not found: {file:?}")]
    NotFound { file: PathBuf },
    #[error("An internal bug occurred, please report this")]
    Internal,
}
