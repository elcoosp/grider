//! Tests for the detection pipeline ([`grider::detection`]) and its `Grid`
//! compatibility wrappers.

use grider::detection::{
    calculate_average_line_size, collect_lines, is_column_empty, is_row_empty, merge_small_lines,
    process_dimension, run_length_encode,
};
use grider::lines::{LineInfo, LineKind, Row};
use grider::{Grid, GridConfig, MIN_THRESHOLD_BLOCK_SIZE};
use image::GrayImage;

fn image_from(width: u32, height: u32, rows: &[&[u8]]) -> GrayImage {
    let flat: Vec<u8> = rows.iter().flat_map(|row| row.iter().copied()).collect();
    GrayImage::from_raw(width, height, flat).expect("valid raw buffer")
}

#[test]
fn is_row_empty_checks_the_requested_row() {
    let img = image_from(3, 3, &[&[255, 255, 255], &[255, 0, 255], &[255, 255, 255]]);
    assert!(is_row_empty(&img, 0, 3));
    assert!(!is_row_empty(&img, 1, 3));
    assert!(is_row_empty(&img, 2, 3));
}

#[test]
fn is_row_empty_honors_partial_width() {
    let img = image_from(4, 1, &[&[255, 255, 0, 0]]);
    assert!(is_row_empty(&img, 0, 2));
    assert!(!is_row_empty(&img, 0, 4));
}

#[test]
fn is_column_empty_checks_the_requested_column() {
    let img = image_from(3, 3, &[&[255, 0, 255], &[255, 0, 255], &[255, 0, 255]]);
    assert!(is_column_empty(&img, 0, 3));
    assert!(!is_column_empty(&img, 1, 3));
    assert!(is_column_empty(&img, 2, 3));
}

#[test]
fn grid_wrappers_delegate_detection() {
    let img = image_from(2, 2, &[&[255, 255], &[0, 0]]);
    assert_eq!(Grid::is_row_empty(&img, 0, 2), is_row_empty(&img, 0, 2));
    assert_eq!(
        Grid::is_column_empty(&img, 0, 2),
        is_column_empty(&img, 0, 2)
    );

    let lines = Grid::collect_lines(&img, 2, 2, Grid::is_row_empty);
    assert_eq!(lines.len(), 2);

    let merged = Grid::merge_small_lines(lines.clone(), 1);
    assert_eq!(merged.len(), lines.len());

    assert_eq!(Grid::calculate_average_line_size(&lines), 1);
}

#[test]
fn grid_process_dimension_wrapper_is_sequential() {
    let img = image_from(
        4,
        6,
        &[
            &[255, 255, 255, 255],
            &[0, 0, 0, 0],
            &[0, 0, 0, 0],
            &[255, 255, 255, 255],
            &[0, 0, 0, 0],
            &[255, 255, 255, 255],
        ],
    );
    let rows = Grid::process_dimension::<Row>(&img, 6, 4, 0.8, Grid::is_row_empty)
        .expect("valid dimensions");
    let expected =
        process_dimension::<Row>(&img, 6, 4, 0.8, is_row_empty, false).expect("valid dimensions");
    assert_eq!(rows, expected);
}

#[test]
fn run_length_encode_partitions_flags() {
    let flags = [false, false, true, false, true, true];
    let runs = run_length_encode(&flags);
    assert_eq!(
        runs,
        vec![
            LineInfo::new(0, 2, LineKind::Full),
            LineInfo::new(2, 1, LineKind::Empty),
            LineInfo::new(3, 1, LineKind::Full),
            LineInfo::new(4, 2, LineKind::Empty),
        ]
    );
    assert_eq!(run_length_encode(&[]), Vec::<LineInfo>::new());
}

#[test]
fn collect_lines_uses_the_custom_predicate() {
    let img = image_from(2, 4, &[&[255, 255], &[0, 0], &[255, 255], &[0, 0]]);
    // A custom "empty" definition: a row is empty when its first pixel is bright.
    let lines = collect_lines(&img, 4, 2, |img, y, _width| img.get_pixel(0, y)[0] > 128);
    assert_eq!(
        lines,
        vec![
            LineInfo::new(0, 1, LineKind::Empty),
            LineInfo::new(1, 1, LineKind::Full),
            LineInfo::new(2, 1, LineKind::Empty),
            LineInfo::new(3, 1, LineKind::Full),
        ]
    );
}

#[test]
fn average_line_size_handles_edge_cases() {
    assert_eq!(calculate_average_line_size(&[]), 0);
    assert_eq!(
        calculate_average_line_size(&[LineInfo::new(0, 7, LineKind::Full)]),
        7
    );
    assert_eq!(
        calculate_average_line_size(&[
            LineInfo::new(0, 1, LineKind::Empty),
            LineInfo::new(1, 2, LineKind::Full),
        ]),
        1
    );
}

#[test]
fn merge_small_lines_merges_short_runs() {
    let lines = vec![
        LineInfo::new(0, 5, LineKind::Empty),
        LineInfo::new(5, 3, LineKind::Empty),
        LineInfo::new(8, 10, LineKind::Full),
        LineInfo::new(18, 2, LineKind::Full),
    ];
    let merged = merge_small_lines(lines, 4);
    assert_eq!(
        merged.as_slice(),
        &[
            LineInfo::new(0, 8, LineKind::Empty),
            LineInfo::new(8, 12, LineKind::Full),
        ]
    );
}

#[test]
fn merge_small_lines_edge_cases() {
    assert!(merge_small_lines(vec![], 5).is_empty());

    let single = vec![LineInfo::new(0, 1, LineKind::Empty)];
    assert_eq!(merge_small_lines(single, 5).len(), 1);

    let lines = vec![
        LineInfo::new(0, 1, LineKind::Empty),
        LineInfo::new(1, 1, LineKind::Full),
    ];
    assert_eq!(merge_small_lines(lines, 0).len(), 2);
}

#[test]
fn process_dimension_rejects_zero_dimensions() {
    let img = GrayImage::new(0, 0);
    let result = process_dimension::<Row>(&img, 0, 0, 0.5, is_row_empty, false);
    assert!(result.is_err());
}

#[test]
fn process_dimension_parallel_matches_sequential() {
    let img = image_from(
        6,
        12,
        &[
            &[255, 255, 255, 255, 255, 255],
            &[0, 0, 0, 0, 0, 0],
            &[0, 0, 0, 0, 0, 0],
            &[0, 0, 0, 0, 0, 0],
            &[255, 255, 255, 255, 255, 255],
            &[0, 0, 0, 0, 0, 0],
            &[0, 0, 0, 0, 0, 0],
            &[255, 255, 255, 255, 255, 255],
            &[255, 255, 255, 255, 255, 255],
            &[0, 0, 0, 0, 0, 0],
            &[0, 0, 0, 0, 0, 0],
            &[0, 0, 0, 0, 0, 0],
        ],
    );
    for ratio in [0.1, 0.5, 0.8] {
        let sequential =
            process_dimension::<Row>(&img, 12, 6, ratio, is_row_empty, false).expect("valid dims");
        let parallel =
            process_dimension::<Row>(&img, 12, 6, ratio, is_row_empty, true).expect("valid dims");
        assert_eq!(sequential, parallel, "ratio {ratio}");
    }
}

#[test]
fn merged_lines_cover_the_whole_dimension() {
    let img = image_from(
        5,
        20,
        &[
            &[255; 5], &[0; 5], &[255; 5], &[0; 5], &[255; 5], &[0; 5], &[255; 5], &[0; 5],
            &[255; 5], &[0; 5], &[255; 5], &[0; 5], &[255; 5], &[0; 5], &[255; 5], &[0; 5],
            &[255; 5], &[0; 5], &[255; 5], &[0; 5],
        ],
    );
    let rows = process_dimension::<Row>(&img, 20, 5, 0.8, is_row_empty, false).expect("valid dims");
    let total: u32 = rows.iter().map(Row::end).next_back().unwrap_or(0);
    assert_eq!(total, 20);
    let total_height: u32 = rows.iter().map(|row| row.height).sum();
    assert_eq!(total_height, 20);
}

#[test]
fn small_block_sizes_are_clamped_by_the_config() {
    assert_eq!(
        GridConfig::new(1, 0.8, true).threshold_block_size,
        MIN_THRESHOLD_BLOCK_SIZE
    );
    let img = image_from(2, 2, &[&[255, 255], &[255, 255]]);
    let config = GridConfig {
        threshold_block_size: 2,
        ..GridConfig::default()
    };
    assert!(
        Grid::try_from_image_with_config(&image::DynamicImage::ImageLuma8(img), config).is_ok()
    );
}

#[test]
fn threshold_delta_shifts_detection() {
    // A faint grey line is treated as background with a permissive delta...
    let img = image_from(1, 3, &[&[255], &[250], &[255]]);
    let dynamic = image::DynamicImage::ImageLuma8(img);

    let permissive = GridConfig {
        threshold_block_size: 3,
        threshold_delta: 30,
        merge_threshold_ratio: 0.8,
        enable_parallel: false,
        binarization: grider::Binarization::AdaptiveMean,
    };
    let grid = Grid::try_from_image_with_config(&dynamic, permissive).unwrap();
    assert!(
        grid.rows.iter().all(|row| row.kind == LineKind::Empty),
        "permissive delta must smooth the faint line away: {grid:?}"
    );

    // ...but shows up as content with a strict (negative) delta.
    let strict = GridConfig {
        threshold_block_size: 3,
        threshold_delta: -30,
        merge_threshold_ratio: 0.8,
        enable_parallel: false,
        binarization: grider::Binarization::AdaptiveMean,
    };
    let grid = Grid::try_from_image_with_config(&dynamic, strict).unwrap();
    assert!(
        grid.rows.iter().any(|row| row.kind == LineKind::Full),
        "strict delta must reveal the faint line: {grid:?}"
    );
}
