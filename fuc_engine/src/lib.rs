use std::{io, path::Path};

use thiserror::Error;
use tokio::task::JoinError;

pub use crate::ops::{FsOp, RemoveOp};

mod ops;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Failed to create the async runtime.")]
    RuntimeCreation(io::Error),
    #[error("An IO error occurred.")]
    Io { error: io::Error, context: String },
    #[error("Failed to retrieve subtask results.")]
    TaskJoin(JoinError),
    #[error("An internal bug occurred, please report this.")]
    Internal,
}

pub fn remove_dir_all<P: AsRef<Path>>(path: P) -> Result<(), Error> {
    RemoveOp::builder().files([path.as_ref()]).build().run()
}
