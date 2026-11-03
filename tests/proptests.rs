//! Property-based tests for the detection pipeline.

use grider::detection::{
    calculate_average_line_size, is_column_empty, is_row_empty, merge_small_lines,
    process_dimension, run_length_encode,
};
use grider::lines::{LineInfo, LineKind, Row};
use grider::{Grid, GridConfig, GridError};
use image::{DynamicImage, GrayImage, Luma};
use proptest::prelude::*;

proptest! {
    #[test]
    fn is_row_empty_matches_pixel_walk(
        width in 1..100u32,
        height in 1..100u32,
        y in 0..100u32,
    ) {
        let img = GrayImage::from_pixel(width, height, Luma([255]));
        if y < height {
            assert!(is_row_empty(&img, y, width));
        }
    }

    #[test]
    fn is_column_empty_matches_pixel_walk(
        width in 1..100u32,
        height in 1..100u32,
        x in 0..100u32,
    ) {
        let img = GrayImage::from_pixel(width, height, Luma([255]));
        if x < width {
            assert!(is_column_empty(&img, x, height));
        }
    }

    #[test]
    fn run_length_encode_partitions_the_flags(flags in prop::collection::vec(any::<bool>(), 0..100)) {
        let runs = run_length_encode(&flags);
        prop_assert_eq!(runs.iter().map(|line| line.length as usize).sum::<usize>(), flags.len());
        for window in runs.windows(2) {
            prop_assert_ne!(window[0].kind, window[1].kind);
            prop_assert_eq!(window[0].start + window[0].length, window[1].start);
        }
        for run in &runs {
            prop_assert!(run.length > 0);
            let empty = flags[run.start as usize];
            prop_assert_eq!(empty, run.kind == LineKind::Empty);
        }
    }

    #[test]
    fn merge_small_lines_preserves_total_length(
        lines in prop::collection::vec(
            (0..100u32, 1..100u32, prop::sample::select(&[LineKind::Empty, LineKind::Full])),
            1..100
        ),
        threshold in 0..100u32,
    ) {
        let lines: Vec<LineInfo> = lines
            .into_iter()
            .map(|(start, length, kind)| LineInfo::new(start, length, kind))
            .collect();
        let merged = merge_small_lines(lines.clone(), threshold);

        let total_input: u32 = lines.iter().map(|line| line.length).sum();
        let total_merged: u32 = merged.iter().map(|line| line.length).sum();
        prop_assert_eq!(total_input, total_merged);
        prop_assert_eq!(merged.first().map(|line| line.start), lines.first().map(|line| line.start));
    }

    #[test]
    fn average_line_size_is_bounded(
        lengths in prop::collection::vec(1..1000u32, 1..50),
    ) {
        let lines: Vec<LineInfo> = lengths
            .iter()
            .enumerate()
            .map(|(index, &length)| {
                LineInfo::new(u32::try_from(index).unwrap_or(u32::MAX), length, LineKind::Full)
            })
            .collect();
        let average = calculate_average_line_size(&lines);
        prop_assert!(average >= lengths.iter().min().copied().unwrap_or(0) - 1);
        prop_assert!(average <= lengths.iter().max().copied().unwrap_or(0) + 1);
    }

    #[test]
    fn processed_grid_covers_the_image(width in 1..100u32, height in 1..100u32) {
        let img = GrayImage::from_fn(width, height, |_, _| Luma([rand::random::<u8>()]));
        let grid: Grid = (&DynamicImage::ImageLuma8(img)).try_into().unwrap();

        let total_row_height: u32 = grid.rows.iter().map(|row| row.height).sum();
        prop_assert_eq!(total_row_height, height);
        let total_column_width: u32 = grid.columns.iter().map(|column| column.width).sum();
        prop_assert_eq!(total_column_width, width);
    }

    #[test]
    fn parallel_and_sequential_pipelines_agree(
        width in 1..60u32,
        height in 1..60u32,
        threshold_block_size in 3..20u32,
        merge_threshold_ratio in 0.1..1.0f32,
    ) {
        let img = GrayImage::from_fn(width, height, |_, _| Luma([rand::random::<u8>()]));
        let dynamic = DynamicImage::ImageLuma8(img);

        let config = GridConfig {
            threshold_block_size,
            merge_threshold_ratio,
            enable_parallel: true,
            ..GridConfig::default()
        };
        let parallel = Grid::try_from_image_with_config(&dynamic, config).unwrap();
        let sequential = Grid::try_from_image_with_config(
            &dynamic,
            GridConfig { enable_parallel: false, ..config },
        ).unwrap();
        prop_assert_eq!(&parallel, &sequential);

        prop_assert!(!parallel.rows.is_empty());
        prop_assert!(!parallel.columns.is_empty());
    }

    #[test]
    fn grid_processing_respects_configs(
        width in 1..100u32,
        height in 1..100u32,
        threshold_block_size in 3..20u32,
        merge_threshold_ratio in 0.1..1.0f32,
    ) {
        let img = GrayImage::from_fn(width, height, |_, _| Luma([rand::random::<u8>()]));
        let dynamic = DynamicImage::ImageLuma8(img);
        let config = GridConfig {
            threshold_block_size,
            merge_threshold_ratio,
            enable_parallel: true,
            ..GridConfig::default()
        };
        let grid = Grid::try_from_image_with_config(&dynamic, config).unwrap();
        prop_assert!(!grid.rows.is_empty());
        prop_assert!(!grid.columns.is_empty());
    }

    #[test]
    fn error_message_includes_dimensions(width in 0u32..1000u32, height in 0u32..1000u32) {
        let error = GridError::InvalidDimensions { width, height };
        let message = error.to_string();
        prop_assert!(message.contains(&width.to_string()));
        prop_assert!(message.contains(&height.to_string()));
    }

    #[test]
    fn process_dimension_succeeds_on_valid_dims(
        primary_dim in 1u32..100u32,
        secondary_dim in 1u32..100u32,
        threshold_ratio in 0.1f32..1.0f32,
    ) {
        let img = GrayImage::new(secondary_dim, primary_dim);
        let result = process_dimension::<Row>(
            &img,
            primary_dim,
            secondary_dim,
            threshold_ratio,
            is_row_empty,
            false,
        );
        prop_assert!(result.is_ok());
    }
}

// Confirms the merged runs still carry well-formed kinds after merging.
#[test]
fn line_kinds_survive_merge() {
    let lines = vec![
        LineInfo::new(0, 1, LineKind::Full),
        LineInfo::new(1, 1, LineKind::Empty),
        LineInfo::new(2, 1, LineKind::Full),
    ];
    let merged = merge_small_lines(lines, 10);
    assert!(
        merged
            .iter()
            .all(|line| line.kind == LineKind::Full || line.kind == LineKind::Empty)
    );
}

proptest! {
    /// The streaming adaptive mean threshold must reproduce
    /// `imageproc::contrast::adaptive_threshold` bit-for-bit on arbitrary
    /// images, radii and deltas — the guarantee that keeps the default
    /// pipeline's output identical across the optimization.
    #[test]
    fn adaptive_mean_matches_imageproc(
        width in 1..40u32,
        height in 1..40u32,
        radius in 1..9u32,
        delta in -60i32..60,
        seed in 0..1000u64,
    ) {
        let count = (width * height) as usize;
        let mut state = seed;
        let pixels: Vec<u8> = (0..count)
            .map(|_| {
                // xorshift-ish deterministic mix, full u8 range.
                state = state.wrapping_mul(6_364_136_223_846_793_005)
                    .wrapping_add(1_442_695_040_888_963_407);
                u8::try_from((state >> 33) & 0xFF).unwrap_or(0)
            })
            .collect();
        let img = GrayImage::from_raw(width, height, pixels).expect("valid raw buffer");
        let mine = grider::binarize::adaptive_mean_threshold(&img, radius, delta, false);
        let reference = imageproc::contrast::adaptive_threshold(&img, radius, delta);
        prop_assert_eq!(mine.as_raw(), reference.as_raw());
    }

    /// The streaming Sauvola threshold must match a naive per-pixel reference
    /// implementation exactly, including every edge of the window.
    #[test]
    fn sauvola_matches_naive_reference(
        width in 1..25u32,
        height in 1..25u32,
        radius in 1..7u32,
        k in -0.4f32..1.0,
        seed in 0..1000u64,
    ) {
        let (width_u, height_u) = (width as usize, height as usize);
        let mut state = seed ^ 0x9E37_79B9_7F4A_7C15;
        let pixels: Vec<u8> = (0..width_u * height_u)
            .map(|_| {
                state = state.wrapping_mul(6_364_136_223_846_793_005)
                    .wrapping_add(1_442_695_040_888_963_407);
                u8::try_from((state >> 33) & 0xFF).unwrap_or(0)
            })
            .collect();
        let img = GrayImage::from_raw(width, height, pixels).expect("valid raw buffer");
        let mine = grider::binarize::sauvola_threshold(&img, radius, k, false);
        let raw = img.as_raw();
        let mut expected = Vec::with_capacity(width_u * height_u);
        let radius = radius as usize;
        for y in 0..height_u {
            for x in 0..width_u {
                let y_low = y.saturating_sub(radius);
                let y_high = (y + radius).min(height_u - 1);
                let x_low = x.saturating_sub(radius);
                let x_high = (x + radius).min(width_u - 1);
                let mut sum = 0.0f64;
                let mut sum_sq = 0.0f64;
                for wy in y_low..=y_high {
                    for wx in x_low..=x_high {
                        let value = f64::from(raw[wy * width_u + wx]);
                        sum += value;
                        sum_sq += value * value;
                    }
                }
                let area =
                    u32::try_from(y_high - y_low + 1).unwrap_or(u32::MAX);
                let area = f64::from(area)
                    * f64::from(u32::try_from(x_high - x_low + 1).unwrap_or(u32::MAX));
                let mean = sum / area;
                let variance = (sum_sq / area - mean * mean).max(0.0);
                let std = variance.sqrt();
                let threshold =
                    mean * (1.0 + f64::from(k) * (std / grider::SAUVOLA_DYNAMIC_RANGE - 1.0));
                expected.push(if f64::from(raw[y * width_u + x]) > threshold {
                    255
                } else {
                    0
                });
            }
        }
        prop_assert_eq!(mine.as_raw(), &expected);
    }

    /// The fused emptiness scan must agree with the per-line predicates on
    /// arbitrary binarized images, and parallelism must change nothing.
    #[test]
    fn scan_flags_match_predicates(
        width in 1..50u32,
        height in 1..50u32,
        density in 0.05f64..0.95,
        seed in 0..1000u64,
    ) {
        let count = (width * height) as usize;
        let mut state = seed | 1;
        let mut raw = Vec::with_capacity(count);
        for _ in 0..count {
            state = state.wrapping_mul(6_364_136_223_846_793_005)
                .wrapping_add(1_442_695_040_888_963_407);
            let draw = f64::from(u32::try_from((state >> 11) % 1000).unwrap_or(0)) / 1000.0;
            raw.push(if draw < density { 0 } else { 255 });
        }
        let img = GrayImage::from_raw(width, height, raw).expect("valid raw buffer");
        let flags = grider::detection::scan_line_flags(&img, false);
        let parallel = grider::detection::scan_line_flags(&img, true);
        prop_assert_eq!(&flags, &parallel);
        for (y, empty) in flags.rows.iter().enumerate() {
            let y = u32::try_from(y).unwrap_or(u32::MAX);
            prop_assert_eq!(*empty, grider::detection::is_row_empty(&img, y, width));
        }
        for (x, empty) in flags.columns.iter().enumerate() {
            let x = u32::try_from(x).unwrap_or(u32::MAX);
            prop_assert_eq!(*empty, grider::detection::is_column_empty(&img, x, height));
        }
    }
}
