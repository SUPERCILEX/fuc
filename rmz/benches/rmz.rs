use std::{fs, fs::File, io, num::NonZeroUsize, path::Path, time::Duration};

use criterion::{
    criterion_group, criterion_main, measurement::WallTime, AxisScale, BenchmarkGroup, BenchmarkId,
    Criterion, PlotConfiguration, Throughput,
};
use ftzz::generator::{Generator, NumFilesWithRatio};
use tempfile::tempdir;

fn uniform(c: &mut Criterion) {
    let mut group = c.benchmark_group("uniform");
    group.plot_config(PlotConfiguration::default().summary_scale(AxisScale::Logarithmic));

    fn gen_files(dir: &Path, num_files: u64) {
        Generator::builder()
            .root_dir(dir.to_path_buf())
            .num_files_with_ratio(NumFilesWithRatio::from_num_files(
                NonZeroUsize::new(usize::try_from(num_files).unwrap()).unwrap(),
            ))
            .build()
            .generate(&mut io::sink())
            .unwrap();
    }

    for num_files in [10, 1_000, 100_000, 1_000_000] {
        group.sample_size(10);
        group.throughput(Throughput::Elements(num_files));

        add_benches(&mut group, num_files, gen_files);
    }
}

fn dir_heavy(c: &mut Criterion) {
    let mut group = c.benchmark_group("dir_heavy");
    group.plot_config(PlotConfiguration::default().summary_scale(AxisScale::Logarithmic));

    fn gen_files(dir: &Path, num_files: u64) {
        Generator::builder()
            .root_dir(dir.to_path_buf())
            .num_files_with_ratio(
                NumFilesWithRatio::new(
                    NonZeroUsize::new(usize::try_from(num_files).unwrap()).unwrap(),
                    NonZeroUsize::new(1).unwrap(),
                )
                .unwrap(),
            )
            .build()
            .generate(&mut io::sink())
            .unwrap();
    }

    for num_files in [10, 1_000, 100_000, 500_000] {
        group.sample_size(10);
        group.throughput(Throughput::Elements(num_files));

        add_benches(&mut group, num_files, gen_files);
    }
}

fn file_heavy(c: &mut Criterion) {
    let mut group = c.benchmark_group("file_heavy");
    group.plot_config(PlotConfiguration::default().summary_scale(AxisScale::Logarithmic));

    fn gen_files(dir: &Path, num_files: u64) {
        Generator::builder()
            .root_dir(dir.to_path_buf())
            .num_files_with_ratio({
                let num_files = NonZeroUsize::new(usize::try_from(num_files).unwrap()).unwrap();
                NumFilesWithRatio::new(num_files, num_files).unwrap()
            })
            .build()
            .generate(&mut io::sink())
            .unwrap();
    }

    for num_files in [10, 1_000, 100_000] {
        group.sample_size(10);
        group.throughput(Throughput::Elements(num_files));

        add_benches(&mut group, num_files, gen_files);
    }
}

fn files_only(c: &mut Criterion) {
    let mut group = c.benchmark_group("files_only");
    group.plot_config(PlotConfiguration::default().summary_scale(AxisScale::Logarithmic));

    fn gen_files(dir: &Path, num_files: u64) {
        let mut file = dir.to_path_buf();
        for i in 0..num_files {
            file.push(format!("{i}"));
            File::create(&file).unwrap();
            file.pop();
        }
    }

    for num_files in [10, 1_000, 100_000] {
        group.sample_size(10);
        group.throughput(Throughput::Elements(num_files));

        add_benches(&mut group, num_files, gen_files);
    }
}

fn deep_dirs(c: &mut Criterion) {
    let mut group = c.benchmark_group("deep_dirs");
    group.plot_config(PlotConfiguration::default().summary_scale(AxisScale::Logarithmic));

    fn gen_files(dir: &Path, num_files: u64) {
        Generator::builder()
            .root_dir(dir.to_path_buf())
            .num_files_with_ratio(
                NumFilesWithRatio::new(
                    NonZeroUsize::new(usize::try_from(num_files).unwrap()).unwrap(),
                    NonZeroUsize::new(10).unwrap(),
                )
                .unwrap(),
            )
            .max_depth(100)
            .build()
            .generate(&mut io::sink())
            .unwrap();
    }

    for num_files in [10, 1_000, 100_000, 1_000_000] {
        group.sample_size(10);
        group.throughput(Throughput::Elements(num_files));

        add_benches(&mut group, num_files, gen_files);
    }
}

fn add_benches(
    group: &mut BenchmarkGroup<WallTime>,
    num_files: u64,
    gen_files: fn(&Path, num_files: u64),
) {
    group.bench_with_input(
        BenchmarkId::new("fuc_engine::remove_dir_all", num_files),
        &num_files,
        |b, num_files| {
            b.iter_with_setup(
                || {
                    let dir = tempdir().unwrap();
                    gen_files(dir.path(), *num_files);
                    dir
                },
                |dir| {
                    fuc_engine::remove_dir_all(dir.path()).unwrap();
                    dir
                },
            );
        },
    );

    group.bench_with_input(
        BenchmarkId::new("fs::remove_dir_all", num_files),
        &num_files,
        |b, num_files| {
            b.iter_with_setup(
                || {
                    let dir = tempdir().unwrap();
                    gen_files(dir.path(), *num_files);
                    dir
                },
                |dir| {
                    fs::remove_dir_all(dir.path()).unwrap();
                    dir
                },
            );
        },
    );

    group.bench_with_input(
        BenchmarkId::new("og_crappy::remove_dir_all", num_files),
        &num_files,
        |b, num_files| {
            b.iter_with_setup(
                || {
                    let dir = tempdir().unwrap();
                    gen_files(dir.path(), *num_files);
                    dir
                },
                |dir| {
                    og_crappy::remove_dir_all(dir.path()).unwrap();
                    dir
                },
            );
        },
    );
}

/// Implementation of the OG post that started all this:
/// https://github.com/tokio-rs/tokio/issues/4172#issuecomment-945052350
mod og_crappy {
    use std::{
        fs, io,
        num::NonZeroUsize,
        path::{Path, PathBuf},
        thread,
    };

    use tokio::task::JoinHandle;

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

            for child in fs::read_dir(&path)? {
                let child = child?;
                if child.file_type()?.is_dir() {
                    tasks.push(spawn_remove_dir_all_recursive(&child.path()));
                } else {
                    fs::remove_file(&child.path())?;
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
}

criterion_group! {
    name = benches;
    config = Criterion::default().noise_threshold(0.02).warm_up_time(Duration::from_secs(1));
    targets =
    uniform,
    dir_heavy,
    file_heavy,
    files_only,
    deep_dirs,
}
criterion_main!(benches);
