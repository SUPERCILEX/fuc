use std::{borrow::Cow, fs, fs::File, io, num::NonZeroU64};

use ftzz::generator::{Generator, NumFilesWithRatio};
use io_adapters::WriteExtension;
use rstest::rstest;
use tempfile::tempdir;

#[test]
fn non_existent_file_no_force() {
    let root = tempdir().unwrap();
    let file = root.path().join("file");

    fuc_engine::RemoveOp::builder()
        .files([Cow::Borrowed(file.as_path())])
        .force(false)
        .build()
        .run()
        .unwrap_err();

    assert!(!file.exists());
    assert!(root.path().exists());
}

#[test]
fn non_existent_file_force() {
    let root = tempdir().unwrap();
    let file = root.path().join("file");

    fuc_engine::RemoveOp::builder()
        .files([Cow::Borrowed(file.as_path())])
        .force(true)
        .build()
        .run()
        .unwrap();

    assert!(!file.exists());
    assert!(root.path().exists());
}

#[test]
fn one_file() {
    let root = tempdir().unwrap();
    let file = root.path().join("file");
    File::create(&file).unwrap();
    assert!(file.exists());

    fuc_engine::remove_file(&file).unwrap();

    assert!(!file.exists());
    assert!(root.path().exists());
}

#[test]
fn one_dir() {
    let root = tempdir().unwrap();
    let dir = root.path().join("dir");
    fs::create_dir(&dir).unwrap();
    assert!(dir.exists());

    fuc_engine::remove_file(&dir).unwrap();

    assert!(!dir.exists());
    assert!(root.path().exists());
}

#[test]
#[cfg(unix)]
fn symbolic_link_delete_dir() {
    let root = tempdir().unwrap();
    let dir = root.path().join("dir");
    fs::create_dir(&dir).unwrap();
    let file = dir.join("file");
    std::os::unix::fs::symlink(".", &file).unwrap();
    assert!(file.exists());

    fuc_engine::remove_file(&dir).unwrap();

    assert!(!file.exists());
    assert!(root.path().exists());
}

#[test]
#[cfg(unix)]
fn symbolic_link_delete_link() {
    let root = tempdir().unwrap();
    let file = root.path().join("file");
    std::os::unix::fs::symlink(".", &file).unwrap();
    assert!(file.exists());

    fuc_engine::remove_file(&file).unwrap();

    assert!(!file.exists());
    assert!(root.path().exists());
}

#[rstest]
fn uniform(#[values(1_000, 100_000)] num_files: u64) {
    let root = tempdir().unwrap();
    let dir = root.path().join("dir");
    Generator::builder()
        .root_dir(dir.clone())
        .num_files_with_ratio(NumFilesWithRatio::from_num_files(
            NonZeroU64::new(num_files).unwrap(),
        ))
        .build()
        .generate(&mut io::sink().write_adapter())
        .unwrap();

    fuc_engine::remove_file(&dir).unwrap();

    assert!(!dir.exists());
    assert!(root.path().exists());
}
