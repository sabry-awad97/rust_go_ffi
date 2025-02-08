use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use rust_go_ffi::{add_numbers, cleanup, initialize};
use semver::Version;
use std::time::Duration;

fn setup() {
    initialize(Version::new(0, 1, 0)).expect("Failed to initialize FFI");
}

fn teardown() {
    cleanup().expect("Failed to cleanup FFI");
}

fn bench_add_numbers(c: &mut Criterion) {
    setup();

    let mut group = c.benchmark_group("add_numbers");
    group.measurement_time(Duration::from_secs(10));
    group.sample_size(100);

    // Benchmark different input sizes
    for size in [1, 100, 10_000, 1_000_000].iter() {
        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, &size| {
            b.iter(|| add_numbers(black_box(size), black_box(size)).unwrap());
        });
    }

    group.finish();
    teardown();
}

fn bench_initialization(c: &mut Criterion) {
    let mut group = c.benchmark_group("initialization");
    group.measurement_time(Duration::from_secs(5));

    group.bench_function("init_and_cleanup", |b| {
        b.iter(|| {
            initialize(Version::new(0, 1, 0)).unwrap();
            cleanup().unwrap();
        });
    });

    group.finish();
}

criterion_group! {
    name = benches;
    config = Criterion::default()
        .with_plots() // Enable plot generation
        .sample_size(50)
        .measurement_time(Duration::from_secs(30));
    targets = bench_add_numbers, bench_initialization
}
criterion_main!(benches);
