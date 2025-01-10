use criterion::{black_box, criterion_group, criterion_main, Criterion};
use grider::{process_lines, Row};
use image::{GrayImage, Luma}; // Import from the `grider` module (or your crate name)

/// Original implementation of `process_lines` for benchmarking.
fn process_lines_original<T>(img: &GrayImage, length: u32, is_empty: impl Fn(u32) -> bool) -> Vec<T>
where
    T: grider::LineTrait,
{
    let mut lines = Vec::new();
    let mut current_start = 0;
    let mut current_kind = if is_empty(0) {
        grider::LineKind::Empty
    } else {
        grider::LineKind::Full
    };
    let mut current_length = 1;

    // First pass: Collect all lines without grouping
    let mut all_lines = Vec::new();
    for i in 1..length {
        let new_kind = if is_empty(i) {
            grider::LineKind::Empty
        } else {
            grider::LineKind::Full
        };
        if new_kind == current_kind {
            current_length += 1;
        } else {
            all_lines.push((current_start, current_length, current_kind.clone()));
            current_start = i;
            current_kind = new_kind;
            current_length = 1;
        }
    }
    // Push the last line
    all_lines.push((current_start, current_length, current_kind));

    // Calculate the average size of all lines
    let total_size: u32 = all_lines.iter().map(|&(_, length, _)| length).sum();
    let average_size = if all_lines.is_empty() {
        0
    } else {
        total_size / all_lines.len() as u32
    };

    // Use 80% of the average size as the threshold to merge more lines
    let threshold = (average_size * 8) / 10;

    // Second pass: Merge lines smaller than the threshold
    let mut merged_lines = Vec::new();
    let mut current_merged_start = all_lines[0].0;
    let mut current_merged_length = all_lines[0].1;
    let mut current_merged_kind = all_lines[0].2.clone();

    for (start, length, kind) in &all_lines[1..] {
        if current_merged_length < threshold || *length < threshold {
            // Merge with the previous line if either is smaller than the threshold
            current_merged_length += length;
        } else {
            // Push the merged line
            merged_lines.push((
                current_merged_start,
                current_merged_length,
                current_merged_kind.clone(),
            ));
            current_merged_start = *start;
            current_merged_length = *length;
            current_merged_kind = kind.clone();
        }
    }
    // Push the last merged line
    merged_lines.push((
        current_merged_start,
        current_merged_length,
        current_merged_kind,
    ));

    // Convert merged lines to the appropriate type
    for (start, length, kind) in merged_lines {
        lines.push(T::new(start, length, kind));
    }

    lines
}

/// Benchmark the original `process_lines` implementation.
fn benchmark_process_lines_original(c: &mut Criterion) {
    let img = GrayImage::from_fn(1000, 1000, |_x, y| {
        if y < 500 {
            Luma([255u8]) // First 500 rows are empty
        } else {
            Luma([0u8]) // Last 500 rows are full
        }
    });

    c.bench_function("process_lines_original", |b| {
        b.iter(|| {
            let _rows: Vec<Row> =
                process_lines_original(&img, 1000, |y| grider::is_row_empty(&img, y, 1000));
            black_box(_rows);
        });
    });
}

/// Benchmark the refactored `process_lines` implementation.
fn benchmark_process_lines_refactored(c: &mut Criterion) {
    let img = GrayImage::from_fn(1000, 1000, |_x, y| {
        if y < 500 {
            Luma([255u8]) // First 500 rows are empty
        } else {
            Luma([0u8]) // Last 500 rows are full
        }
    });

    c.bench_function("process_lines_refactored", |b| {
        b.iter(|| {
            let _rows: Vec<Row> =
                process_lines(&img, 1000, |y| grider::is_row_empty(&img, y, 1000));
            black_box(_rows);
        });
    });
}

criterion_group!(
    benches,
    benchmark_process_lines_original,
    benchmark_process_lines_refactored
);
criterion_main!(benches);
