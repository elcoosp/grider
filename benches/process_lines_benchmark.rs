use criterion::{black_box, criterion_group, criterion_main, Criterion};
use grider::{process_lines, LineInfo, Row, SmallVecLine};
use image::{GrayImage, Luma}; // Import from the `grider` module (or your crate name)

/// Benchmark the refactored `process_lines` implementation.
fn benchmark_process_lines_refactored(c: &mut Criterion) {
    let img = GrayImage::from_fn(10000, 10000, |_x, y| {
        if y < 500 {
            Luma([255u8]) // First 500 rows are empty
        } else {
            Luma([0u8]) // Last 500 rows are full
        }
    });

    c.bench_function("process_lines_refactored", |b| {
        b.iter(|| {
            let _rows: SmallVecLine<Row> =
                process_lines(&img, 10000, |y| grider::is_row_empty(&img, y, 10000));
            black_box(_rows);
        });
    });
}

criterion_group!(benches, benchmark_process_lines_refactored);
criterion_main!(benches);
