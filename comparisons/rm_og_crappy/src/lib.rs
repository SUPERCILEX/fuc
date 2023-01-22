#![allow(clippy::pedantic)]

use std::{
    fs, io,
    num::NonZeroUsize,
    path::{Path, PathBuf},
    thread,
};

use tokio::task::JoinHandle;

/// Implementation of the OG post that started all this:
/// <https://github.com/tokio-rs/tokio/issues/4172#issuecomment-945052350>
pub fn remove_dir_all<P: AsRef<Path>>(path: P) -> Result<(), io::Error> {
    let parallelism =
        thread::available_parallelism().unwrap_or(unsafe { NonZeroUsize::new_unchecked(1) });
    let runtime = tokio::runtime::Builder::new_current_thread()
        .max_blocking_threads(parallelism.get())
        .build()?;

    runtime.block_on(fast_remove_dir_all(path.as_ref()))
}

async fn fast_remove_dir_all(path: &Path) -> io::Result<()> {
    let path = path.to_path_buf();
    let path = tokio::task::spawn_blocking(|| -> io::Result<Option<PathBuf>> {
        let filetype = fs::symlink_metadata(&path)?.file_type();
        if filetype.is_symlink() {
            fs::remove_file(&path)?;
            Ok(None)
        } else {
            Ok(Some(path))
        }
    })
    .await??;

    match path {
        None => Ok(()),
        Some(path) => remove_dir_all_recursive(path).await,
    }
}

async fn remove_dir_all_recursive(path: PathBuf) -> io::Result<()> {
    let path_copy = path.clone();
    let tasks = tokio::task::spawn_blocking(move || -> io::Result<_> {
        let mut tasks = Vec::new();

        for child in fs::read_dir(path)? {
            let child = child?;
            if child.file_type()?.is_dir() {
                tasks.push(spawn_remove_dir_all_recursive(&child.path()));
            } else {
                fs::remove_file(child.path())?;
            }
        }

        Ok(tasks)
    })
    .await??;

    for result in futures::future::join_all(tasks).await {
        result??;
    }

    tokio::task::spawn_blocking(|| fs::remove_dir(path_copy)).await??;

    Ok(())
}

fn spawn_remove_dir_all_recursive(path: &Path) -> JoinHandle<io::Result<()>> {
    tokio::task::spawn(remove_dir_all_recursive(path.to_path_buf()))
}
