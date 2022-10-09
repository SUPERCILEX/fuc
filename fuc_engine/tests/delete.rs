use std::{fs, fs::File, io, num::NonZeroUsize};

use ftzz::generator::{Generator, NumFilesWithRatio};
use tempfile::tempdir;

use fuc_engine::{remove_dir_all, RemoveOp};

#[test]
fn one_file() {
    let dir = tempdir().unwrap();
    let file = dir.path().join("file");
    File::create(&file).unwrap();
    assert!(file.exists());

    remove_dir_all(&file).unwrap();

    assert!(!file.exists());
}

#[test]
fn one_dir() {
    let dir = tempdir().unwrap();
    let file = dir.path().join("file");
    fs::create_dir(&file).unwrap();
    assert!(file.exists());

    remove_dir_all(&file).unwrap();

    assert!(!file.exists());
}

#[test]
fn dependency() {
    let dir = tempdir().unwrap();
    let a = dir.path().join("a");
    let b = a.join("b");
    fs::create_dir(&a).unwrap();
    fs::create_dir(&b).unwrap();
    assert!(a.exists());
    assert!(b.exists());

    RemoveOp::builder()
        .files([a.as_ref(), b.as_ref()])
        .build()
        .run()
        .unwrap();

    assert!(!a.exists());
    assert!(!b.exists());
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

    remove_dir_all(dir.path()).unwrap();

    assert!(!dir.path().exists());
}
