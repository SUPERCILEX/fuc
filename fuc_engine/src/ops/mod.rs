pub use remove::RemoveOp;

use crate::Error;

mod remove;

pub trait FsOp {
    fn run(self) -> Result<(), Error>;
}
