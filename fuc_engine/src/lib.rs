#![feature(const_cstr_methods)]
#![feature(const_result_drop)]
#![feature(const_option)]
#![feature(once_cell)]
#![allow(clippy::module_name_repetitions)]

use std::{io, path::PathBuf};

use thiserror::Error;

pub use crate::ops::{copy_file, remove_file, remove_file as remove_dir_all, CopyOp, RemoveOp};

mod ops;

#[derive(Error, Debug)]
pub enum Error {
    #[error("An I/O error occurred")]
    Io { error: io::Error, context: String },
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
