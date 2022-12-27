#![allow(clippy::pedantic)]

use std::{fs, io, path::Path};

pub fn copy_dir<P: AsRef<Path>, Q: AsRef<Path>>(from: P, to: Q) -> Result<(), io::Error> {
    let to = to.as_ref();
    fs::create_dir(to)?;
    from.as_ref()
        .read_dir()?
        .try_for_each(|dir_entry| -> io::Result<()> {
            let dir_entry = dir_entry?;
            let to = to.join(dir_entry.file_name());
            if dir_entry.file_type()?.is_dir() {
                copy_dir(dir_entry.path(), to)?;
            } else {
                fs::copy(dir_entry.path(), to)?;
            }
            Ok(())
        })
}
