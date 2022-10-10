#![allow(clippy::module_name_repetitions)]
#![feature(io_error_more)]

use std::{borrow::Cow, io, path::Path};

use thiserror::Error;
use tokio::task::JoinError;

pub use crate::ops::RemoveOp;

mod ops;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Failed to create the async runtime.")]
    RuntimeCreation(io::Error),
    #[error("An IO error occurred.")]
    Io { error: io::Error, context: String },
    #[error("An attempt was made to delete `/`.")]
    PreserveRoot,
    #[error("Failed to retrieve subtask results.")]
    TaskJoin(JoinError),
    #[error("An internal bug occurred, please report this.")]
    Internal,
}

/// Removes a directory at this path, after removing all its contents.
///
/// This function does **not** follow symbolic links and it will simply remove
/// the symbolic link itself.
///
/// > Note: This function currently starts its own tokio runtime.
///
/// # Errors
///
/// Returns the underlying I/O errors that occurred.
pub fn remove_dir_all<P: AsRef<Path>>(path: P) -> Result<(), Error> {
    RemoveOp::builder()
        .files([Cow::Borrowed(path.as_ref())])
        .build()
        .run()
}
