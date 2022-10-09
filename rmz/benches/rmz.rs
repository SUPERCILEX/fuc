use std::{
    alloc,
    alloc::Layout,
    fs::{copy, File, OpenOptions},
    io::{BufRead, BufReader, Read, Write},
    os::unix::{fs::FileExt, io::AsRawFd},
    path::{Path, PathBuf},
    thread,
    time::Duration,
};

use criterion::{
    criterion_group, criterion_main, measurement::WallTime, AxisScale, BatchSize, BenchmarkGroup,
    BenchmarkId, Criterion, PlotConfiguration, Throughput,
};
use tempfile::{tempdir, TempDir};

fn uniform(c: &mut Criterion) {
    let mut group = c.benchmark_group("uniform");
    group.plot_config(PlotConfiguration::default().summary_scale(AxisScale::Logarithmic));
}

criterion_group! {
    name = benches;
    config = Criterion::default().noise_threshold(0.02).warm_up_time(Duration::from_secs(1));
    targets =
    uniform,
}
criterion_main!(benches);
