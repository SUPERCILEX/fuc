use std::{borrow::Cow, fs, fs::File};

use tempfile::tempdir;

#[test]
fn pre_existing_file_no_force() {
    let root = tempdir().unwrap();
    let from = root.path().join("from");
    File::create(&from).unwrap();
    let to = root.path().join("to");
    File::create(&to).unwrap();

    fuc_engine::CopyOp::builder()
        .files([(Cow::Owned(from), Cow::Owned(to))])
        .force(false)
        .build()
        .run()
        .unwrap_err();
}

#[test]
fn pre_existing_file_force() {
    let root = tempdir().unwrap();
    let from = root.path().join("from");
    File::create(&from).unwrap();
    let to = root.path().join("to");
    File::create(&to).unwrap();

    fuc_engine::CopyOp::builder()
        .files([(Cow::Owned(from), Cow::Owned(to))])
        .force(true)
        .build()
        .run()
        .unwrap();
}

#[test]
fn pre_existing_dir_force() {
    let root = tempdir().unwrap();
    let from = root.path().join("from");
    fs::create_dir(&from).unwrap();
    File::create(from.join("a")).unwrap();
    File::create(from.join("b")).unwrap();
    let to = root.path().join("to");
    fs::create_dir(&to).unwrap();
    File::create(to.join("b")).unwrap();
    File::create(to.join("c")).unwrap();

    fuc_engine::CopyOp::builder()
        .files([(Cow::Owned(from), Cow::Borrowed(to.as_path()))])
        .force(true)
        .build()
        .run()
        .unwrap();

    assert!(to.join("a").exists());
    assert!(to.join("b").exists());
    assert!(to.join("c").exists());
}

#[test]
#[cfg(unix)]
fn self_nested() {
    let root = tempdir().unwrap();
    let from = root.path().join("from");
    fs::create_dir(&from).unwrap();
    let to = root.path().join("from/to");

    fuc_engine::CopyOp::builder()
        .files([(Cow::Owned(from), Cow::Borrowed(to.as_path()))])
        .force(true)
        .build()
        .run()
        .unwrap();

    assert!(to.exists());
}

#[test]
fn non_existent_parent_dir() {
    let root = tempdir().unwrap();
    let from = root.path().join("from");
    fs::create_dir(&from).unwrap();
    let to = root.path().join("a/b/c/to");

    fuc_engine::CopyOp::builder()
        .files([(Cow::Owned(from), Cow::Borrowed(to.as_path()))])
        .force(true)
        .build()
        .run()
        .unwrap();

    assert!(to.exists());
}

#[test]
fn one_file() {
    let root = tempdir().unwrap();
    let from = root.path().join("from");
    File::create(&from).unwrap();
    let to = root.path().join("to");

    fuc_engine::copy_file(&from, &to).unwrap();

    assert!(to.exists());
}

#[test]
fn one_dir() {
    let root = tempdir().unwrap();
    let from = root.path().join("from");
    fs::create_dir(&from).unwrap();
    File::create(from.join("file")).unwrap();
    let to = root.path().join("to");

    fuc_engine::copy_file(&from, &to).unwrap();

    assert!(to.exists());
}

#[test]
#[cfg(unix)]
fn symbolic_link_copy_dir() {
    let root = tempdir().unwrap();
    let from = root.path().join("dir");
    fs::create_dir(&from).unwrap();
    std::os::unix::fs::symlink(".", from.join("file")).unwrap();
    let to = root.path().join("to");

    fuc_engine::copy_file(&from, &to).unwrap();

    assert!(to.exists());
}

#[test]
#[cfg(unix)]
fn symbolic_link_copy_link() {
    let root = tempdir().unwrap();
    let from = root.path().join("from");
    std::os::unix::fs::symlink(".", &from).unwrap();
    let to = root.path().join("to");

    fuc_engine::copy_file(&from, &to).unwrap();

    assert!(to.exists());
}
