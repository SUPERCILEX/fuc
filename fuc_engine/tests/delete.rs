use std::{fs, fs::File, io, num::NonZeroUsize};

use ftzz::generator::{Generator, NumFilesWithRatio};
use rstest::rstest;
use tempfile::tempdir;

#[test]
fn one_file() {
    let dir = tempdir().unwrap();
    let file = dir.path().join("file");
    File::create(&file).unwrap();
    assert!(file.exists());

    fuc_engine::remove_dir_all(&file).unwrap();

    assert!(!file.exists());
    assert!(dir.as_ref().exists());
}

#[test]
fn one_dir() {
    let dir = tempdir().unwrap();
    let root = dir.path().join("dir");
    fs::create_dir(&root).unwrap();
    assert!(root.exists());

    fuc_engine::remove_dir_all(&root).unwrap();

    assert!(!root.exists());
    assert!(dir.as_ref().exists());
}

#[rstest]
fn uniform(#[values(1_000, 100_000, 1_000_000)] num_files: usize) {
    let dir = tempdir().unwrap();
    let root = dir.path().join("root");
    Generator::builder()
        .root_dir(root.clone())
        .num_files_with_ratio(NumFilesWithRatio::from_num_files(
            NonZeroUsize::new(num_files).unwrap(),
        ))
        .build()
        .generate(&mut io::sink())
        .unwrap();

    fuc_engine::remove_dir_all(&root).unwrap();

    assert!(!root.exists());
    assert!(dir.as_ref().exists());
}
