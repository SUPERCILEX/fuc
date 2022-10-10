use std::{fs, fs::File, io, num::NonZeroUsize};

use ftzz::generator::{Generator, NumFilesWithRatio};
use tempfile::tempdir;

#[test]
fn one_file() {
    let dir = tempdir().unwrap();
    let file = dir.path().join("file");
    File::create(&file).unwrap();
    assert!(file.exists());

    fuc_engine::remove_dir_all(&file).unwrap();

    assert!(!file.exists());
}

#[test]
fn one_dir() {
    let dir = tempdir().unwrap();
    let file = dir.path().join("file");
    fs::create_dir(&file).unwrap();
    assert!(file.exists());

    fuc_engine::remove_dir_all(&file).unwrap();

    assert!(!file.exists());
}

#[test]
fn large() {
    let dir = tempdir().unwrap();
    Generator::builder()
        .root_dir(dir.path().to_path_buf())
        .num_files_with_ratio(NumFilesWithRatio::from_num_files(
            NonZeroUsize::new(100_000).unwrap(),
        ))
        .build()
        .generate(&mut io::sink())
        .unwrap();

    fuc_engine::remove_dir_all(dir.path()).unwrap();

    assert!(!dir.path().exists());
}
