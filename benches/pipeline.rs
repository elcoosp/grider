//! Criterion benchmarks for the optimized pipeline stages: binarization
//! algorithms, the fused emptiness scan, and the different entry points.

use criterion::{BenchmarkId, Criterion, criterion_group, criterion_main};
use grider::testing::{Pattern, test_image};
use grider::{Grid, GridConfig, scan_line_flags};
use image::GrayImage;
use std::hint::black_box;

/// Benchmarks the binarization algorithms against each other and against the
/// upstream `imageproc` implementation the adaptive mean replaces.
fn bench_binarization(c: &mut Criterion) {
    let mut group = c.benchmark_group("binarization");
    for (width, height) in [(1000_u32, 1000_u32), (4000_u32, 4000_u32)] {
        let img = test_image(width, height, Pattern::Checkerboard);
        let gray = img.to_luma8();

        group.bench_with_input(
            BenchmarkId::new("imageproc_adaptive_mean", format!("{width}x{height}")),
            &gray,
            |b, gray| {
                b.iter(|| {
                    black_box(imageproc::contrast::adaptive_threshold(gray, 12, 0));
                });
            },
        );
        group.bench_with_input(
            BenchmarkId::new("grider_adaptive_mean", format!("{width}x{height}")),
            &gray,
            |b, gray| {
                b.iter(|| {
                    black_box(grider::binarize::adaptive_mean_threshold(
                        gray, 12, 0, false,
                    ));
                });
            },
        );
        group.bench_with_input(
            BenchmarkId::new("grider_adaptive_mean_parallel", format!("{width}x{height}")),
            &gray,
            |b, gray| {
                b.iter(|| {
                    black_box(grider::binarize::adaptive_mean_threshold(gray, 12, 0, true));
                });
            },
        );
        group.bench_with_input(
            BenchmarkId::new("grider_sauvola", format!("{width}x{height}")),
            &gray,
            |b, gray| {
                b.iter(|| {
                    black_box(grider::binarize::sauvola_threshold(gray, 12, 0.2, false));
                });
            },
        );
    }
    group.finish();
}

/// Benchmarks the fused emptiness scan that replaced the row + strided
/// column double pass.
fn bench_scan(c: &mut Criterion) {
    let mut group = c.benchmark_group("scan_flags");
    for (width, height) in [(1000_u32, 1000_u32), (4000_u32, 4000_u32)] {
        let img = test_image(width, height, Pattern::Dense);
        let binarized: GrayImage = {
            let gray = img.to_luma8();
            grider::binarize::adaptive_mean_threshold(&gray, 12, 0, false)
        };

        group.bench_with_input(
            BenchmarkId::new("sequential", format!("{width}x{height}")),
            &binarized,
            |b, binarized| {
                b.iter(|| black_box(scan_line_flags(binarized, false)));
            },
        );
        group.bench_with_input(
            BenchmarkId::new("parallel", format!("{width}x{height}")),
            &binarized,
            |b, binarized| {
                b.iter(|| black_box(scan_line_flags(binarized, true)));
            },
        );
    }
    group.finish();
}

/// Benchmarks the extraction entry points: `DynamicImage` (with and without a
/// grayscale conversion), raw `GrayImage`, and pre-binarized input.
fn bench_entry_points(c: &mut Criterion) {
    let mut group = c.benchmark_group("entry_points");
    let (width, height) = (1000_u32, 1000_u32);
    let luma = test_image(width, height, Pattern::Dense);
    let rgb = image::DynamicImage::ImageRgb8(luma.to_rgb8());
    let gray = luma.to_luma8();
    let binarized = grider::binarize::binarize(&gray, &GridConfig::default());
    let config = GridConfig::default();

    group.bench_function("dynamic_luma8_zero_copy", |b| {
        b.iter(|| black_box(Grid::try_from_image_with_config(&luma, config).unwrap()));
    });
    group.bench_function("dynamic_rgb_converted", |b| {
        b.iter(|| black_box(Grid::try_from_image_with_config(&rgb, config).unwrap()));
    });
    group.bench_function("gray_direct", |b| {
        b.iter(|| black_box(Grid::try_from_gray_with_config(&gray, config).unwrap()));
    });
    group.bench_function("binarized_skip_threshold", |b| {
        b.iter(|| black_box(Grid::try_from_binarized(&binarized, config).unwrap()));
    });
    group.finish();
}

criterion_group! {
    name = pipeline_benches;
    config = Criterion::default().sample_size(20);
    targets = bench_binarization, bench_scan, bench_entry_points
}
criterion_main!(pipeline_benches);
