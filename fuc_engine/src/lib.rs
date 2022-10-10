#![allow(clippy::module_name_repetitions)]
#![feature(io_error_more)]

use std::io;

use thiserror::Error;
use tokio::task::JoinError;

pub use crate::ops::{remove_dir_all, RemoveOp};

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
