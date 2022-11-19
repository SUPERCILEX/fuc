#![allow(clippy::pedantic)]

use std::{fs, io, path::Path};

use rayon::prelude::*;

pub fn remove_dir_all<P: AsRef<Path>>(path: P) -> Result<(), io::Error> {
    path.as_ref()
        .read_dir()?
        .par_bridge()
        .try_for_each(|dir_entry| -> io::Result<()> {
            let dir_entry = dir_entry?;
            if dir_entry.file_type()?.is_dir() {
                remove_dir_all(dir_entry.path())?;
            } else {
                fs::remove_file(dir_entry.path())?;
            }
            Ok(())
        })?;
    fs::remove_dir(path)
}
