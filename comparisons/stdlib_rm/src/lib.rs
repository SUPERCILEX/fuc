#![allow(clippy::pedantic)]

use std::{fs, io, path::Path};

pub fn remove_dir_all<P: AsRef<Path>>(path: P) -> Result<(), io::Error> {
    fs::remove_dir_all(path)
}
