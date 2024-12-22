use std::{borrow::Cow, fs, fs::File};

use rstest::rstest;
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
        .unwrap_err();

    assert!(!to.exists());
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

#[rstest]
#[cfg(unix)]
fn dereference_symbolic_link_to_regular_file(#[values(false, true)] follow_symlinks: bool) {
    let root = tempdir().unwrap();
    let from = root.path().join("from");
    File::create(from).unwrap();
    let link = root.path().join("link");
    std::os::unix::fs::symlink("from", &link).unwrap();
    let to = root.path().join("to");

    fuc_engine::CopyOp::builder()
        .files([(Cow::Owned(link), Cow::Borrowed(to.as_path()))])
        .follow_symlinks(follow_symlinks)
        .build()
        .run()
        .unwrap();

    if follow_symlinks {
        assert!(to.symlink_metadata().unwrap().is_file());
    } else {
        assert!(to.symlink_metadata().unwrap().is_symlink());
    }
}

#[rstest]
#[cfg(unix)]
fn dereference_symbolic_link_to_regular_file_in_dir(#[values(false, true)] follow_symlinks: bool) {
    let root = tempdir().unwrap();
    let from = root.path().join("from");
    fs::create_dir(&from).unwrap();
    File::create(from.join("file")).unwrap();
    std::os::unix::fs::symlink("file", from.join("link")).unwrap();
    let to = root.path().join("to");

    fuc_engine::CopyOp::builder()
        .files([(Cow::Owned(from), Cow::Borrowed(to.as_path()))])
        .follow_symlinks(follow_symlinks)
        .build()
        .run()
        .unwrap();

    assert!(to.join("file").symlink_metadata().unwrap().is_file());
    if follow_symlinks {
        assert!(to.join("link").symlink_metadata().unwrap().is_file());
    } else {
        assert!(to.join("link").symlink_metadata().unwrap().is_symlink());
    }
}

#[rstest]
#[cfg(unix)]
fn dereference_symbolic_link_to_dir_in_dir(#[values(false, true)] follow_symlinks: bool) {
    let root = tempdir().unwrap();
    let from = root.path().join("from");
    fs::create_dir(&from).unwrap();
    fs::create_dir(from.join("subdir")).unwrap();
    File::create(from.join("subdir/file")).unwrap();
    std::os::unix::fs::symlink("subdir", from.join("subdirlink")).unwrap();
    let to = root.path().join("to");

    fuc_engine::CopyOp::builder()
        .files([(Cow::Owned(from), Cow::Borrowed(to.as_path()))])
        .follow_symlinks(follow_symlinks)
        .build()
        .run()
        .unwrap();

    assert!(to.join("subdir").symlink_metadata().unwrap().is_dir());
    assert!(to.join("subdir/file").symlink_metadata().unwrap().is_file());
    if follow_symlinks {
        assert!(to.join("subdirlink").symlink_metadata().unwrap().is_dir());
        assert!(
            to.join("subdirlink/file")
                .symlink_metadata()
                .unwrap()
                .is_file()
        );
    } else {
        assert!(
            to.join("subdirlink")
                .symlink_metadata()
                .unwrap()
                .is_symlink()
        );
    }
}
