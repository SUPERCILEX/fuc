#![allow(clippy::multiple_crate_versions)]

use std::{fmt, fs::File, num::NonZeroU64, path::Path, time::Duration};

use criterion::{
    criterion_group, criterion_main, measurement::WallTime, AxisScale, BenchmarkGroup, BenchmarkId,
    Criterion, PlotConfiguration, Throughput,
};
use ftzz::generator::{Generator, NumFilesWithRatio};
use tempfile::tempdir;

// TODO https://github.com/rust-lang/rust/pull/104389
struct Sink;

impl fmt::Write for Sink {
    fn write_str(&mut self, _: &str) -> fmt::Result {
        Ok(())
    }
}

fn uniform(c: &mut Criterion) {
    fn gen_files(dir: &Path, num_files: u64) {
        Generator::builder()
            .root_dir(dir.to_path_buf())
            .num_files_with_ratio(NumFilesWithRatio::from_num_files(
                NonZeroU64::new(num_files).unwrap(),
            ))
            .build()
            .generate(&mut Sink)
            .unwrap();
    }

    let mut group = c.benchmark_group("uniform");
    group.plot_config(PlotConfiguration::default().summary_scale(AxisScale::Logarithmic));

    for num_files in [10, 1_000, 100_000, 1_000_000] {
        group.sample_size(10);
        group.throughput(Throughput::Elements(num_files));

        add_benches(&mut group, num_files, gen_files);
    }
}

fn dir_heavy(c: &mut Criterion) {
    fn gen_files(dir: &Path, num_files: u64) {
        Generator::builder()
            .root_dir(dir.to_path_buf())
            .num_files_with_ratio(
                NumFilesWithRatio::new(
                    NonZeroU64::new(num_files).unwrap(),
                    NonZeroU64::new(1).unwrap(),
                )
                .unwrap(),
            )
            .build()
            .generate(&mut Sink)
            .unwrap();
    }

    let mut group = c.benchmark_group("dir_heavy");
    group.plot_config(PlotConfiguration::default().summary_scale(AxisScale::Logarithmic));

    for num_files in [10, 1_000, 100_000, 500_000] {
        group.sample_size(10);
        group.throughput(Throughput::Elements(num_files));

        add_benches(&mut group, num_files, gen_files);
    }
}

fn file_heavy(c: &mut Criterion) {
    fn gen_files(dir: &Path, num_files: u64) {
        Generator::builder()
            .root_dir(dir.to_path_buf())
            .num_files_with_ratio({
                let num_files = NonZeroU64::new(num_files).unwrap();
                NumFilesWithRatio::new(num_files, num_files).unwrap()
            })
            .build()
            .generate(&mut Sink)
            .unwrap();
    }

    let mut group = c.benchmark_group("file_heavy");
    group.plot_config(PlotConfiguration::default().summary_scale(AxisScale::Logarithmic));

    for num_files in [10, 1_000, 100_000] {
        group.sample_size(10);
        group.throughput(Throughput::Elements(num_files));

        add_benches(&mut group, num_files, gen_files);
    }
}

fn files_only(c: &mut Criterion) {
    fn gen_files(dir: &Path, num_files: u64) {
        let mut file = dir.to_path_buf();
        for i in 0..num_files {
            file.push(format!("{i}"));
            File::create(&file).unwrap();
            file.pop();
        }
    }

    let mut group = c.benchmark_group("files_only");
    group.plot_config(PlotConfiguration::default().summary_scale(AxisScale::Logarithmic));

    for num_files in [10, 1_000, 100_000] {
        group.sample_size(10);
        group.throughput(Throughput::Elements(num_files));

        add_benches(&mut group, num_files, gen_files);
    }
}

fn deep_dirs(c: &mut Criterion) {
    fn gen_files(dir: &Path, num_files: u64) {
        Generator::builder()
            .root_dir(dir.to_path_buf())
            .num_files_with_ratio(
                NumFilesWithRatio::new(
                    NonZeroU64::new(num_files).unwrap(),
                    NonZeroU64::new(10).unwrap(),
                )
                .unwrap(),
            )
            .max_depth(100)
            .build()
            .generate(&mut Sink)
            .unwrap();
    }

    let mut group = c.benchmark_group("deep_dirs");
    group.plot_config(PlotConfiguration::default().summary_scale(AxisScale::Logarithmic));

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
                    assert!(!dir.path().exists());
                    dir
                },
            );
        },
    );

    group.bench_with_input(
        BenchmarkId::new("rayon_rm::remove_dir_all", num_files),
        &num_files,
        |b, num_files| {
            b.iter_with_setup(
                || {
                    let dir = tempdir().unwrap();
                    gen_files(dir.path(), *num_files);
                    dir
                },
                |dir| {
                    rayon_rm::remove_dir_all(dir.path()).unwrap();
                    assert!(!dir.path().exists());
                    dir
                },
            );
        },
    );

    group.bench_with_input(
        BenchmarkId::new("og_crappy_rm::remove_dir_all", num_files),
        &num_files,
        |b, num_files| {
            b.iter_with_setup(
                || {
                    let dir = tempdir().unwrap();
                    gen_files(dir.path(), *num_files);
                    dir
                },
                |dir| {
                    og_crappy_rm::remove_dir_all(dir.path()).unwrap();
                    assert!(!dir.path().exists());
                    dir
                },
            );
        },
    );

    group.bench_with_input(
        BenchmarkId::new("stdlib_rm::remove_dir_all", num_files),
        &num_files,
        |b, num_files| {
            b.iter_with_setup(
                || {
                    let dir = tempdir().unwrap();
                    gen_files(dir.path(), *num_files);
                    dir
                },
                |dir| {
                    stdlib_rm::remove_dir_all(dir.path()).unwrap();
                    assert!(!dir.path().exists());
                    dir
                },
            );
        },
    );
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
