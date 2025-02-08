use criterion::{black_box, criterion_group, criterion_main, Criterion};
use rust_go_ffi::{add_numbers, initialize};
use semver::Version;

pub fn criterion_benchmark(c: &mut Criterion) {
    // Initialize FFI system once before benchmarks
    initialize(Version::new(0, 1, 0)).expect("Failed to initialize FFI");

    c.bench_function("add small numbers", |b| {
        b.iter(|| {
            add_numbers(black_box(5), black_box(3)).unwrap();
        })
    });

    c.bench_function("add large numbers", |b| {
        b.iter(|| {
            add_numbers(black_box(1000000), black_box(2000000)).unwrap();
        })
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
