use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};
use grider::{Grid, GridConfig};
use image::{DynamicImage, GrayImage, Luma};
use std::hint::black_box;

// Helper function to create test images of different patterns
fn create_test_image(width: u32, height: u32, pattern: &str) -> DynamicImage {
    let img = match pattern {
        "checkerboard" => GrayImage::from_fn(width, height, |x, y| {
            if (x + y) % 2 == 0 {
                Luma([255])
            } else {
                Luma([0])
            }
        }),
        "horizontal_stripes" => GrayImage::from_fn(width, height, |_, y| {
            if y % 2 == 0 {
                Luma([255])
            } else {
                Luma([0])
            }
        }),
        "vertical_stripes" => GrayImage::from_fn(width, height, |x, _| {
            if x % 2 == 0 {
                Luma([255])
            } else {
                Luma([0])
            }
        }),
        "sparse" => GrayImage::from_fn(width, height, |x, y| {
            if x % 10 == 0 && y % 10 == 0 {
                Luma([0])
            } else {
                Luma([255])
            }
        }),
        "dense" => GrayImage::from_fn(width, height, |x, y| {
            if x % 3 == 0 || y % 3 == 0 {
                Luma([0])
            } else {
                Luma([255])
            }
        }),
        _ => GrayImage::from_pixel(width, height, Luma([255])), // Default to all white
    };
    DynamicImage::ImageLuma8(img)
}

// Benchmark different image sizes
fn bench_image_sizes(c: &mut Criterion) {
    let mut group = c.benchmark_group("image_sizes");
    let sizes = [
        (100, 100),
        (500, 500),
        (1000, 1000),
        (2000, 2000),
        (4000, 4000),
    ];

    for size in sizes.iter() {
        let (width, height) = *size;
        let img = create_test_image(width, height, "checkerboard");
        let config = GridConfig::default();

        group.bench_with_input(
            BenchmarkId::new("size", format!("{}x{}", width, height)),
            &img,
            |b, img| {
                b.iter(|| {
                    black_box(Grid::try_from_image_with_config(img, config.clone()).unwrap());
                });
            },
        );
    }
    group.finish();
}

// Benchmark different image patterns
fn bench_patterns(c: &mut Criterion) {
    let mut group = c.benchmark_group("patterns");
    let patterns = [
        "checkerboard",
        "horizontal_stripes",
        "vertical_stripes",
        "sparse",
        "dense",
    ];
    let size = (1000, 1000); // Fixed size for pattern comparison

    for pattern in patterns.iter() {
        let img = create_test_image(size.0, size.1, pattern);
        let config = GridConfig::default();

        group.bench_with_input(BenchmarkId::new("pattern", pattern), &img, |b, img| {
            b.iter(|| {
                black_box(Grid::try_from_image_with_config(img, config.clone()).unwrap());
            });
        });
    }
    group.finish();
}

// Benchmark different configurations
fn bench_configs(c: &mut Criterion) {
    let mut group = c.benchmark_group("configurations");
    let img = create_test_image(1000, 1000, "checkerboard");

    let configs = vec![
        ("small_block_sequential", GridConfig::new(4, 0.5, false)),
        ("small_block_parallel", GridConfig::new(4, 0.5, true)),
        ("large_block_sequential", GridConfig::new(24, 0.8, false)),
        ("large_block_parallel", GridConfig::new(24, 0.8, true)),
    ];

    for (name, config) in configs {
        group.bench_with_input(BenchmarkId::new("config", name), &img, |b, img| {
            b.iter(|| {
                black_box(Grid::try_from_image_with_config(img, config.clone()).unwrap());
            });
        });
    }
    group.finish();
}

// Benchmark parallel vs sequential processing
fn bench_parallel_processing(c: &mut Criterion) {
    let mut group = c.benchmark_group("parallel_vs_sequential");
    let sizes = [(500, 500), (1000, 1000), (2000, 2000)];

    for size in sizes.iter() {
        let (width, height) = *size;
        let img = create_test_image(width, height, "dense");

        let parallel_config = GridConfig {
            enable_parallel: true,
            ..GridConfig::default()
        };

        let sequential_config = GridConfig {
            enable_parallel: false,
            ..GridConfig::default()
        };

        // Benchmark parallel processing
        group.bench_with_input(
            BenchmarkId::new("parallel", format!("{}x{}", width, height)),
            &img,
            |b, img| {
                b.iter(|| {
                    black_box(
                        Grid::try_from_image_with_config(img, parallel_config.clone()).unwrap(),
                    );
                });
            },
        );

        // Benchmark sequential processing
        group.bench_with_input(
            BenchmarkId::new("sequential", format!("{}x{}", width, height)),
            &img,
            |b, img| {
                b.iter(|| {
                    black_box(
                        Grid::try_from_image_with_config(img, sequential_config.clone()).unwrap(),
                    );
                });
            },
        );
    }
    group.finish();
}

// Benchmark threshold block sizes
fn bench_threshold_blocks(c: &mut Criterion) {
    let mut group = c.benchmark_group("threshold_blocks");
    let img = create_test_image(1000, 1000, "dense");
    let block_sizes = [4, 8, 12, 16, 24, 32];

    for block_size in block_sizes.iter() {
        let config = GridConfig {
            threshold_block_size: *block_size,
            ..GridConfig::default()
        };

        group.bench_with_input(
            BenchmarkId::new("block_size", block_size),
            &img,
            |b, img| {
                b.iter(|| {
                    black_box(Grid::try_from_image_with_config(img, config.clone()).unwrap());
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
