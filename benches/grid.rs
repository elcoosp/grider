//! Criterion benchmarks for the grid extraction pipeline.

use criterion::{BenchmarkId, Criterion, criterion_group, criterion_main};
use grider::testing::{Pattern, test_image};
use grider::{Grid, GridConfig};
use std::hint::black_box;

/// Benchmarks extraction over a range of image sizes.
fn bench_image_sizes(c: &mut Criterion) {
    let mut group = c.benchmark_group("image_sizes");
    let sizes = [
        (100, 100),
        (500, 500),
        (1000, 1000),
        (2000, 2000),
        (4000, 4000),
    ];

    for (width, height) in sizes {
        let img = test_image(width, height, Pattern::Checkerboard);
        let config = GridConfig::default();

        group.bench_with_input(
            BenchmarkId::new("size", format!("{width}x{height}")),
            &img,
            |b, img| {
                b.iter(|| {
                    black_box(Grid::try_from_image_with_config(img, config).unwrap());
                });
            },
        );
    }
    group.finish();
}

/// Benchmarks extraction over the standard image patterns.
fn bench_patterns(c: &mut Criterion) {
    let mut group = c.benchmark_group("patterns");
    let patterns = [
        Pattern::Checkerboard,
        Pattern::HorizontalStripes,
        Pattern::VerticalStripes,
        Pattern::Sparse,
        Pattern::Dense,
    ];
    let (width, height) = (1000, 1000);

    for pattern in patterns {
        let img = test_image(width, height, pattern);
        let config = GridConfig::default();

        group.bench_with_input(
            BenchmarkId::new("pattern", format!("{pattern:?}")),
            &img,
            |b, img| {
                b.iter(|| {
                    black_box(Grid::try_from_image_with_config(img, config).unwrap());
                });
            },
        );
    }
    group.finish();
}

/// Benchmarks different configuration shapes.
fn bench_configs(c: &mut Criterion) {
    let mut group = c.benchmark_group("configurations");
    let img = test_image(1000, 1000, Pattern::Checkerboard);

    let configs = [
        ("small_block_sequential", GridConfig::new(4, 0.5, false)),
        ("small_block_parallel", GridConfig::new(4, 0.5, true)),
        ("large_block_sequential", GridConfig::new(24, 0.8, false)),
        ("large_block_parallel", GridConfig::new(24, 0.8, true)),
    ];

    for (name, config) in configs {
        group.bench_with_input(BenchmarkId::new("config", name), &img, |b, img| {
            b.iter(|| {
                black_box(Grid::try_from_image_with_config(img, config).unwrap());
            });
        });
    }
    group.finish();
}

/// Benchmarks parallel against sequential processing.
fn bench_parallel_processing(c: &mut Criterion) {
    let mut group = c.benchmark_group("parallel_vs_sequential");
    let sizes = [(500, 500), (1000, 1000), (2000, 2000)];

    for (width, height) in sizes {
        let img = test_image(width, height, Pattern::Dense);

        let parallel_config = GridConfig {
            enable_parallel: true,
            ..GridConfig::default()
        };
        let sequential_config = GridConfig {
            enable_parallel: false,
            ..GridConfig::default()
        };

        group.bench_with_input(
            BenchmarkId::new("parallel", format!("{width}x{height}")),
            &img,
            |b, img| {
                b.iter(|| {
                    black_box(Grid::try_from_image_with_config(img, parallel_config).unwrap());
                });
            },
        );

        group.bench_with_input(
            BenchmarkId::new("sequential", format!("{width}x{height}")),
            &img,
            |b, img| {
                b.iter(|| {
                    black_box(Grid::try_from_image_with_config(img, sequential_config).unwrap());
                });
            },
        );
    }
    group.finish();
}

/// Benchmarks the adaptive threshold block sizes.
fn bench_threshold_blocks(c: &mut Criterion) {
    let mut group = c.benchmark_group("threshold_blocks");
    let img = test_image(1000, 1000, Pattern::Dense);
    let block_sizes = [4, 8, 12, 16, 24, 32];

    for block_size in block_sizes {
        let config = GridConfig {
            threshold_block_size: block_size,
            ..GridConfig::default()
        };

        group.bench_with_input(
            BenchmarkId::new("block_size", block_size),
            &img,
            |b, img| {
                b.iter(|| {
                    black_box(Grid::try_from_image_with_config(img, config).unwrap());
                });
            },
        );
    }
    group.finish();
}

criterion_group! {
    name = benches;
    config = Criterion::default().sample_size(20); // Reduced sample size for faster runs
    targets = bench_image_sizes, bench_patterns, bench_configs,
              bench_parallel_processing, bench_threshold_blocks
}
criterion_main!(benches);
