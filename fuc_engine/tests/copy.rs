use std::{borrow::Cow, fs, fs::File};

use tempfile::tempdir;

#[test]
fn pre_existing_file_no_force() {
    let root = tempdir().unwrap();
    let file1 = root.path().join("file1");
    File::create(&file1).unwrap();
    assert!(file1.exists());
    let file2 = root.path().join("file2");
    File::create(&file2).unwrap();
    assert!(file2.exists());

    fuc_engine::CopyOp::builder()
        .files([(Cow::Owned(file1), Cow::Owned(file2))])
        .force(false)
        .build()
        .run()
        .unwrap_err();
}

#[test]
fn pre_existing_file_force() {
    let root = tempdir().unwrap();
    let file1 = root.path().join("file1");
    File::create(&file1).unwrap();
    assert!(file1.exists());
    let file2 = root.path().join("file2");
    File::create(&file2).unwrap();
    assert!(file2.exists());

    fuc_engine::CopyOp::builder()
        .files([(Cow::Owned(file1), Cow::Owned(file2))])
        .force(true)
        .build()
        .run()
        .unwrap();
}

#[test]
#[cfg(unix)]
fn self_nested() {
    let root = tempdir().unwrap();
    let dir1 = root.path().join("dir1");
    fs::create_dir(&dir1).unwrap();
    let dir2 = root.path().join("dir1/dir2");

    fuc_engine::CopyOp::builder()
        .files([(Cow::Owned(dir1), Cow::Borrowed(dir2.as_path()))])
        .force(true)
        .build()
        .run()
        .unwrap();

    assert!(dir2.exists());
}

#[test]
fn non_existent_parent_dir() {
    let root = tempdir().unwrap();
    let dir1 = root.path().join("dir1");
    fs::create_dir(&dir1).unwrap();
    let dir2 = root.path().join("a/b/c/dir2");

    fuc_engine::CopyOp::builder()
        .files([(Cow::Owned(dir1), Cow::Borrowed(dir2.as_path()))])
        .force(true)
        .build()
        .run()
        .unwrap();

    assert!(dir2.exists());
}

#[test]
fn one_file() {
    let root = tempdir().unwrap();
    let file1 = root.path().join("file1");
    File::create(&file1).unwrap();
    assert!(file1.exists());
    let file2 = root.path().join("file2");

    fuc_engine::copy_file(&file1, &file2).unwrap();

    assert!(file2.exists());
}

#[test]
fn one_dir() {
    let root = tempdir().unwrap();
    let dir1 = root.path().join("dir1");
    fs::create_dir(&dir1).unwrap();
    assert!(dir1.exists());
    File::create(dir1.join("file")).unwrap();
    let dir2 = root.path().join("dir2");

    fuc_engine::copy_file(&dir1, &dir2).unwrap();

    assert!(dir2.exists());
}
