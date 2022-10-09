use std::time::Duration;

use criterion::{criterion_group, criterion_main, AxisScale, Criterion, PlotConfiguration};

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
