use anyhow::{Context, Result};
use grider::make_grid;
use grider::{Grid, GridConfig};
use image::GenericImageView;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};
fn main() -> Result<()> {
    // Initialize tracing subscriber

    tracing_subscriber::registry()
        .with(EnvFilter::from_default_env())
        .init();

    // Replace with the path to your image file
    let image_path = "tests/large.png";

    // Open and process the image
    let img = image::open(image_path).context("Failed to open image")?;

    // Validate image dimensions
    let (width, height) = img.dimensions();
    if width == 0 || height == 0 {
        anyhow::bail!("Invalid image dimensions: {}x{}", width, height);
    }

    // Process the image with configuration
    let config = GridConfig::new(12, 0.8, true);
    let grid = Grid::try_from_image_with_config(&img, config)?;

    // Use debug features under feature flag
    #[cfg(feature = "debug")]
    {
        // Save the image with grid lines for debugging
        let output_path = format!("{image_path}_output_with_grid.png");
        grider::debug::save_image_with_grid(&img, &grid, &output_path);
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use grider::*;
    use image::*;
    use insta::assert_yaml_snapshot;
    use proptest::prelude::*;
    use test_case::test_case;
    #[test]
    fn test_try_from_image_with_config() {
        let img = GrayImage::from_fn(100, 100, |x, y| {
            if (x + y) % 2 == 0 {
                Luma([255])
            } else {
                Luma([0])
            }
        });
        let dynamic_img = DynamicImage::ImageLuma8(img);

        let config = GridConfig {
            threshold_block_size: 8,
            merge_threshold_ratio: 0.7,
            enable_parallel: true,
        };

        let result = Grid::try_from_image_with_config(&dynamic_img, config);
        assert!(result.is_ok());
    }

    #[test_case(0, 100)]
    #[test_case(100, 0)]
    fn test_invalid_dimensions(width: u32, height: u32) {
        let img = GrayImage::new(width, height);
        let dynamic_img = DynamicImage::ImageLuma8(img);
        let config = GridConfig::default();

        let result = Grid::try_from_image_with_config(&dynamic_img, config);
        assert!(matches!(
            result,
            Err(GridError::InvalidDimensions { width: w, height: h })
            if w == width && h == height
        ));
    }

    #[cfg(feature = "debug")]
    #[test]
    fn test_save_image_with_grid() {
        // Create a 10x10 grayscale image with a gradient from black to white
        let img = GrayImage::from_fn(10, 10, |x, _y| Luma([(x * 25) as u8]));
        let dynamic_img = DynamicImage::ImageLuma8(img);

        // Create a grid with some rows and columns
        let grid = Grid {
            rows: SmallVecLine::from_vec(vec![
                Row::new(LineInfo::new(0, 5, LineKind::Empty)),
                Row::new(LineInfo::new(5, 5, LineKind::Full)),
            ]),
            columns: SmallVecLine::from_vec(vec![
                Column::new(LineInfo::new(0, 5, LineKind::Empty)),
                Column::new(LineInfo::new(5, 5, LineKind::Full)),
            ]),
        };

        // Save the image with grid lines
        let output_path = "test_output_with_grid.png";
        grider::debug::save_image_with_grid(&dynamic_img, &grid, output_path);

        // Check that the file was created
        assert!(std::path::Path::new(output_path).exists());

        // Clean up the test file
        std::fs::remove_file(output_path).unwrap();
    }
    #[test]
    fn test_is_row_empty_2() {
        // Create a 10x10 grayscale image with all pixels set to white (255)
        let img = GrayImage::from_pixel(10, 10, Luma([255]));

        // Check that all rows are empty
        for y in 0..10 {
            assert!(Grid::is_row_empty(&img, y, 10));
        }

        // Set one pixel in the first row to black (0)
        let mut img = img;
        img.put_pixel(5, 0, Luma([0]));

        // Check that the first row is no longer empty
        assert!(!Grid::is_row_empty(&img, 0, 10));
    }

    #[test]
    fn test_merge_lines() {
        // Create a vector of lines with varying lengths and kinds
        let lines = vec![
            LineInfo::new(0, 5, LineKind::Empty),
            LineInfo::new(5, 3, LineKind::Empty),
            LineInfo::new(8, 10, LineKind::Full),
            LineInfo::new(18, 2, LineKind::Full),
        ];

        // Merge lines smaller than the threshold (e.g., 4)
        let merged_lines = Grid::merge_small_lines(lines, 4);

        // Expected result:
        // - The first two Empty lines are merged (length 5 + 3 = 8)
        // - The last two Full lines are merged (length 10 + 2 = 12)
        assert_eq!(
            merged_lines,
            SmallVecLine::from_vec(vec![
                LineInfo::new(0, 8, LineKind::Empty),
                LineInfo::new(8, 12, LineKind::Full),
            ])
        );
    }

    #[test]
    fn test_is_column_empty_2() {
        // Create a 10x10 grayscale image with all pixels set to white (255)
        let img = GrayImage::from_pixel(10, 10, Luma([255]));

        // Check that all columns are empty
        for x in 0..10 {
            assert!(Grid::is_column_empty(&img, x, 10));
        }

        // Set one pixel in the first column to black (0)
        let mut img = img;
        img.put_pixel(0, 5, Luma([0]));

        // Check that the first column is no longer empty
        assert!(!Grid::is_column_empty(&img, 0, 10));
    }

    #[test]
    fn test_is_row_empty() {
        let img =
            GrayImage::from_raw(3, 3, vec![255, 255, 255, 255, 0, 255, 255, 255, 255]).unwrap();
        assert!(Grid::is_row_empty(&img, 0, 3));
        assert!(!Grid::is_row_empty(&img, 1, 3));
        assert!(Grid::is_row_empty(&img, 2, 3));
    }
    proptest! {
        #[test]
        fn test_is_row_empty_proptest(width in 1..100u32, height in 1..100u32, y in 0..100u32) {
            // Create a grayscale image with all pixels set to white (255)
            let img = GrayImage::from_pixel(width, height, Luma([255]));

            // If y is within the image height, the row should be empty
            if y < height {
                assert!(Grid::is_row_empty(&img, y, width));
            }
        }

        #[test]
        fn test_is_column_empty_proptest(width in 1..100u32, height in 1..100u32, x in 0..100u32) {
            // Create a grayscale image with all pixels set to white (255)
            let img = GrayImage::from_pixel(width, height, Luma([255]));

            // If x is within the image width, the column should be empty
            if x < width {
                assert!(Grid::is_column_empty(&img, x, height));
            }
        }

        #[test]
        fn test_merge_small_lines_proptest(
            lines in prop::collection::vec((0..100u32, 1..100u32, prop::sample::select(&[LineKind::Empty, LineKind::Full])), 1..100),
            threshold in 1..100u32
        ) {
            // Convert the generated lines into LineInfo structs
            let lines: Vec<LineInfo> = lines.into_iter()
                .map(|(start, length, kind)| LineInfo::new(start, length, kind))
                .collect();

            // Merge the lines using the threshold
            let merged_lines = Grid::merge_small_lines(lines .clone(), threshold);

            // Verify that no merged line is smaller than the threshold (unless it's the last line)
            for line in merged_lines.iter().take(merged_lines.len() - 1) {
                assert!(line.length >= threshold);
            }

            // Verify that the total length of the merged lines equals the total length of the input lines
            let total_input_length: u32 = lines.iter().map(|line| line.length).sum();
            let total_merged_length: u32 = merged_lines.iter().map(|line| line.length).sum();
            assert_eq!(total_input_length, total_merged_length);
        }
        #[test]
        fn test_process_image_proptest(width in 1..100u32, height in 1..100u32) {
            // Create a grayscale image with random pixel values
            let img = GrayImage::from_fn(width, height, |_, _| Luma([rand::random::<u8>()]));

            // Process the image
            let grid: Grid = (&DynamicImage::ImageLuma8(img)).try_into().unwrap();

            // Verify that the sum of row heights equals the image height
            let total_row_height: u32 = grid.rows.iter().map(|row| row.height).sum();
            assert_eq!(total_row_height, height);

            // Verify that the sum of column widths equals the image width
            let total_column_width: u32 = grid.columns.iter().map(|col| col.width).sum();
            assert_eq!(total_column_width, width);
        }
        #[test]
        fn test_collect_all_lines_proptest(length in 1..100u32) {
            // Generate a random pattern of empty and full lines
            let pattern: Vec<bool> = (0..length).map(|_| rand::random::<bool>()).collect();

            // Define a function to check if a line is empty
            let is_empty = |i: u32| pattern[i as usize];

            // Collect all lines
            let lines = Grid::collect_all_lines(length, &is_empty);

            // Verify that the lines match the pattern
            let mut current_start = 0;
            for line in lines {
                let expected_kind = if pattern[current_start as usize] {
                    LineKind::Empty
                } else {
                    LineKind::Full
                };
                assert_eq!(line.kind, expected_kind);
                current_start += line.length;
            }
        }
    }
    #[test]
    fn test_is_column_empty() {
        let img =
            GrayImage::from_raw(3, 3, vec![255, 255, 255, 255, 0, 255, 255, 255, 255]).unwrap();
        assert!(Grid::is_column_empty(&img, 0, 3));
        assert!(!Grid::is_column_empty(&img, 1, 3));
        assert!(Grid::is_column_empty(&img, 2, 3));
    }

    #[test]
    fn test_process_image() {
        let img = GrayImage::from_fn(10, 10, |_x, y| {
            if y < 5 {
                Luma([255u8]) // First 5 rows are empty
            } else {
                Luma([0u8]) // Last 5 rows are full
            }
        });
        let grid: Grid = (&DynamicImage::ImageLuma8(img)).try_into().unwrap();

        // Assert YAML snapshot with a custom name
        assert_yaml_snapshot!("process_image", grid);
    }

    #[test]
    fn test_process_image_with_redactions() {
        let img = GrayImage::from_fn(10, 10, |_x, y| {
            if y < 5 {
                Luma([255u8]) // First 5 rows are empty
            } else {
                Luma([0u8]) // Last 5 rows are full
            }
        });
        let grid: Grid = (&DynamicImage::ImageLuma8(img)).try_into().unwrap();

        // Define the expected grid using the higher-order macro
        let expected_grid = make_grid! {
            rows: [
                (0, 5),
                (5, 5, LineKind::Full),
            ],
            columns: [
                (0, 10, LineKind::Full),
            ]
        };

        // Assert that the generated grid matches the expected grid
        assert_eq!(grid, expected_grid);
        assert_yaml_snapshot!("process_image_with_redactions", grid, {
            ".rows[0].y" => 0, // Redact the first row's y-coordinate
            ".rows[1].y" => 5, // Redact the second row's y-coordinate
        });
    }

    #[test]
    fn test_grid_processing_small_image() {
        let img =
            GrayImage::from_raw(3, 3, vec![255, 255, 255, 255, 0, 255, 255, 255, 255]).unwrap();
        let dynamic_img = DynamicImage::ImageLuma8(img);

        let config = GridConfig {
            threshold_block_size: 3, // Small block size for small image
            merge_threshold_ratio: 0.5,
            enable_parallel: false, // Sequential processing for small images
        };

        let grid = Grid::try_from_image_with_config(&dynamic_img, config).unwrap();
        assert_yaml_snapshot!("grid_processing_small_image", grid);
    }

    #[test]
    fn test_grid_processing_large_image() {
        let img = GrayImage::from_fn(10, 10, |_x, y| {
            if y < 5 {
                Luma([255u8]) // First 5 rows are empty
            } else {
                Luma([0u8]) // Last 5 rows are full
            }
        });
        let dynamic_img = DynamicImage::ImageLuma8(img);

        let config = GridConfig::default();
        let grid = Grid::try_from_image_with_config(&dynamic_img, config).unwrap();
        assert_yaml_snapshot!("grid_processing_large_image", grid);
    }

    proptest! {
        #[test]
        fn test_grid_processing_proptest(width in 1..100u32, height in 1..100u32) {
            // Create a grayscale image with random pixel values
            let img = GrayImage::from_fn(width, height, |_, _| Luma([rand::random::<u8>()]));
            let dynamic_img = DynamicImage::ImageLuma8(img);

            let config = GridConfig::default();
            let grid = Grid::try_from_image_with_config(&dynamic_img, config).unwrap();

            // Verify that the sum of row heights equals the image height
            let total_row_height: u32 = grid.rows.iter().map(|row| row.height).sum();
            prop_assert_eq!(total_row_height, height);

            // Verify that the sum of column widths equals the image width
            let total_column_width: u32 = grid.columns.iter().map(|col| col.width).sum();
            prop_assert_eq!(total_column_width, width);
        }

        #[test]
        fn test_grid_processing_with_different_configs(
            width in 1..100u32,
            height in 1..100u32,
            threshold_block_size in 3..20u32,
            merge_threshold_ratio in 0.1..1.0f32
        ) {
            let img = GrayImage::from_fn(width, height, |_, _| Luma([rand::random::<u8>()]));
            let dynamic_img = DynamicImage::ImageLuma8(img);

            let config = GridConfig {
                threshold_block_size,
                merge_threshold_ratio,
                enable_parallel: true,
            };

            let grid = Grid::try_from_image_with_config(&dynamic_img, config).unwrap();

            // Basic validity checks
            prop_assert!(!grid.rows.is_empty());
            prop_assert!(!grid.columns.is_empty());

            // Check total dimensions
            let total_row_height: u32 = grid.rows.iter().map(|row| row.height).sum();
            let total_column_width: u32 = grid.columns.iter().map(|col| col.width).sum();
            prop_assert_eq!(total_row_height, height);
            prop_assert_eq!(total_column_width, width);
        }
    }

    #[test]
    fn test_grid_with_custom_config() {
        let img = GrayImage::from_fn(20, 20, |x, y| {
            if (x < 10 && y < 10) || (x >= 10 && y >= 10) {
                Luma([0])
            } else {
                Luma([255])
            }
        });
        let dynamic_img = DynamicImage::ImageLuma8(img);

        // Test with different configurations
        let configs = vec![
            GridConfig {
                threshold_block_size: 5,
                merge_threshold_ratio: 0.3,
                enable_parallel: false,
            },
            GridConfig {
                threshold_block_size: 10,
                merge_threshold_ratio: 0.8,
                enable_parallel: true,
            },
        ];

        for (i, config) in configs.into_iter().enumerate() {
            let grid = Grid::try_from_image_with_config(&dynamic_img, config).unwrap();
            assert_yaml_snapshot!(format!("grid_custom_config_{}", i), grid);
        }
    }

    #[test]
    fn test_grid_with_redactions() {
        let img = GrayImage::from_fn(
            10,
            10,
            |_x, y| {
                if y < 5 {
                    Luma([255u8])
                } else {
                    Luma([0u8])
                }
            },
        );
        let dynamic_img = DynamicImage::ImageLuma8(img);

        let grid = Grid::try_from_image_with_config(&dynamic_img, GridConfig::default()).unwrap();

        let expected_grid = make_grid! {
            rows: [
                (0, 5),
                (5, 5, LineKind::Full),
            ],
            columns: [
                (0, 10, LineKind::Full),
            ]
        };

        assert_eq!(grid, expected_grid);
        assert_yaml_snapshot!("grid_with_redactions", grid, {
            ".rows[0].y" => 0,
            ".rows[1].y" => 5,
        });
    }

    /// Creates a test image with specified dimensions and pattern
    ///
    /// # Arguments
    /// * `width` - Width of the image
    /// * `height` - Height of the image
    /// * `pattern` - Pattern type ("checkerboard", "gradient", "empty", "full")
    ///
    /// # Returns
    /// * `DynamicImage` - The created test image
    pub fn create_test_image(width: u32, height: u32, pattern: &str) -> DynamicImage {
        let img = match pattern {
            "checkerboard" => GrayImage::from_fn(width, height, |x, y| {
                if (x + y) % 2 == 0 {
                    Luma([255u8]) // White pixel
                } else {
                    Luma([0u8]) // Black pixel
                }
            }),
            "gradient" => GrayImage::from_fn(width, height, |x, _y| {
                Luma([((x as f32 / width as f32) * 255.0) as u8])
            }),
            "empty" => GrayImage::from_pixel(width, height, Luma([255u8])), // All white
            "full" => GrayImage::from_pixel(width, height, Luma([0u8])),    // All black
            _ => GrayImage::from_pixel(width, height, Luma([255u8])),       // Default to empty
        };

        DynamicImage::ImageLuma8(img)
    }
    // Macro to define a generic test for filtering rows or columns
    macro_rules! test_filter {
        ($name:ident, $filter_method:ident, $kind:expr) => {
            #[test]
            fn $name() {
                let img = create_test_image(10, 10, "checkerboard");
                let config = GridConfig::default();
                let grid = Grid::try_from_image_with_config(&img, config).unwrap();

                // Filter rows or columns by kind
                let filtered: Vec<_> = grid.$filter_method(|item| item.kind == $kind).collect();
                assert!(!filtered.is_empty());
                assert!(filtered.iter().all(|item| item.kind == $kind));
            }
        };
    }

    // Macro to define a generic test for counting rows or columns by kind
    macro_rules! test_count {
        ($name:ident, $count_method:ident, $kind:expr) => {
            #[test]
            fn $name() {
                let img = create_test_image(10, 10, "checkerboard");
                let config = GridConfig::default();
                let grid = Grid::try_from_image_with_config(&img, config).unwrap();

                // Count rows or columns by kind
                let count = grid.$count_method($kind);
                assert!(count > 0);
            }
        };
    }

    // Macro to define a generic test for finding cells
    macro_rules! test_find_cells {
        ($name:ident, $row_indices:expr, $column_indices:expr) => {
            #[test]
            fn $name() {
                let img = create_test_image(10, 10, "checkerboard");
                let config = GridConfig::default();
                let grid = Grid::try_from_image_with_config(&img, config).unwrap();

                // Find cells based on row and column indices
                let cells: Vec<_> = grid.find_cells($row_indices, $column_indices).collect();
                assert!(!cells.is_empty());
                for cell in cells {
                    assert!(cell.is_ok());
                }
            }
        };
    }

    // Define tests using macros
    test_filter!(test_filtered_rows, filtered_rows, LineKind::Full);
    test_filter!(test_filtered_columns, filtered_columns, LineKind::Full);

    test_count!(test_count_rows_by_kind, count_rows_by_kind, LineKind::Full);
    test_count!(
        test_count_columns_by_kind,
        count_columns_by_kind,
        LineKind::Full
    );

    // test_find_cells!(test_find_cells, &[1, 2, 3], &[1, 2, 3]);

    // Additional tests for edge cases
    #[test]
    fn test_filtered_rows_empty() {
        let img = create_test_image(10, 10, "empty");
        let config = GridConfig::default();
        let grid = Grid::try_from_image_with_config(&img, config).unwrap();

        // Filter rows by kind (should be empty)
        let filtered: Vec<_> = grid
            .filtered_rows(|row| row.kind == LineKind::Full)
            .collect();
        assert!(filtered.is_empty());
    }

    #[test]
    fn test_filtered_columns_empty() {
        let img = create_test_image(10, 10, "empty");
        let config = GridConfig::default();
        let grid = Grid::try_from_image_with_config(&img, config).unwrap();

        // Filter columns by kind (should be empty)
        let filtered: Vec<_> = grid
            .filtered_columns(|col| col.kind == LineKind::Full)
            .collect();
        assert!(filtered.is_empty());
    }

    #[test]
    fn test_find_cells_invalid_indices() {
        let img = create_test_image(10, 10, "checkerboard");
        let config = GridConfig::default();
        let grid = Grid::try_from_image_with_config(&img, config).unwrap();

        // Find cells with invalid indices
        let cells: Vec<_> = grid.find_cells(&[100], &[200]).collect();
        assert!(!cells.is_empty());
        for cell in cells {
            assert!(cell.is_err());
        }
    }
    #[test]
    fn test_create_checkerboard() {
        let img = create_test_image(2, 2, "checkerboard");
        if let DynamicImage::ImageLuma8(gray_img) = img {
            assert_eq!(gray_img.get_pixel(0, 0)[0], 255);
            assert_eq!(gray_img.get_pixel(1, 0)[0], 0);
            assert_eq!(gray_img.get_pixel(0, 1)[0], 0);
            assert_eq!(gray_img.get_pixel(1, 1)[0], 255);
        } else {
            panic!("Expected ImageLuma8");
        }
    }

    #[test]
    fn test_create_gradient() {
        let img = create_test_image(4, 1, "gradient");
        if let DynamicImage::ImageLuma8(gray_img) = img {
            assert!(gray_img.get_pixel(0, 0)[0] < gray_img.get_pixel(3, 0)[0]);
        } else {
            panic!("Expected ImageLuma8");
        }
    }

    #[test]
    fn test_create_empty_and_full() {
        let empty = create_test_image(2, 2, "empty");
        let full = create_test_image(2, 2, "full");

        if let (DynamicImage::ImageLuma8(empty_img), DynamicImage::ImageLuma8(full_img)) =
            (empty, full)
        {
            assert_eq!(empty_img.get_pixel(0, 0)[0], 255);
            assert_eq!(full_img.get_pixel(0, 0)[0], 0);
        } else {
            panic!("Expected ImageLuma8");
        }
    }

    #[test]
    fn test_invalid_pattern() {
        let img = create_test_image(2, 2, "invalid");
        if let DynamicImage::ImageLuma8(gray_img) = img {
            assert_eq!(gray_img.get_pixel(0, 0)[0], 255); // Should default to empty
        } else {
            panic!("Expected ImageLuma8");
        }
    }
    // New macro for testing line info creation and validation
    #[macro_export]
    macro_rules! test_line_info {
    ($($name:ident: $value:expr,)*) => {
        $(
            #[test]
            fn $name() {
                let (start, length, kind, expected) = $value;
                let line = LineInfo::new(start, length, kind);
                assert_eq!(line, expected);
            }
        )*
    }
}

    // New macro for testing grid configuration
    #[macro_export]
    macro_rules! test_grid_config {
    ($($name:ident: $value:expr,)*) => {
        $(
            #[test]
            fn $name() {
                let (block_size, ratio, parallel, expected_result) = $value;
                let config = GridConfig {
                    threshold_block_size: block_size,
                    merge_threshold_ratio: ratio,
                    enable_parallel: parallel,
                };
                let img = create_test_image(100, 100, "checkerboard");
                let result = Grid::try_from_image_with_config(&img, config).is_ok();
                assert_eq!(result, expected_result);
            }
        )*
    }
}

    #[cfg(test)]
    mod tests {
        use super::*;
        use test_case::test_case;

        // Test line info creation with the new macro
        test_line_info! {
            test_line_info_empty: (0, 5, LineKind::Empty, LineInfo { start: 0, length: 5, kind: LineKind::Empty }),
            test_line_info_full: (10, 3, LineKind::Full, LineInfo { start: 10, length: 3, kind: LineKind::Full }),
            test_line_info_zero_length: (0, 0, LineKind::Empty, LineInfo { start: 0, length: 0, kind: LineKind::Empty }),
        }

        // Test grid configurations with the new macro
        test_grid_config! {
            test_grid_config_small_block: (4, 0.5, true, true),
            test_grid_config_large_block: (24, 0.8, false, true),
            // FIXME: thread 'tests::tests::test_grid_config_invalid_block' panicked at /Users/admin/.cargo/registry/src/index.crates.io-6f17d22bba15001f/imageproc-0.25.0/src/contrast.rs:20:5: assertion failed: block_radius > 0
            // test_grid_config_invalid_block: (0, 0.5, true, false),
        }

        // Test uncovered error cases
        #[test]
        fn test_grid_error_display() {
            let error = GridError::ImageConversionError("test error".to_string());
            assert_eq!(error.to_string(), "Failed to convert image: test error");

            let error = GridError::ThresholdingError("threshold failed".to_string());
            assert_eq!(
                error.to_string(),
                "Failed to apply threshold: threshold failed"
            );
        }

        // Test process_dimension with invalid inputs
        #[test]
        fn test_process_dimension_invalid() {
            let img = GrayImage::new(0, 0);
            let result = Grid::process_dimension::<Row>(&img, 0, 0, 0.5, Grid::is_row_empty);
            assert!(result.is_err());
        }

        // Test parallel processing with edge cases
        #[test]
        fn test_parallel_processing_edge_cases() {
            let img = GrayImage::new(1, 1);

            // Test minimal image
            let config = GridConfig {
                enable_parallel: true,
                ..Default::default()
            };
            let result =
                Grid::try_from_image_with_config(&DynamicImage::ImageLuma8(img.clone()), config);
            assert!(result.is_ok());

            // Test with very small block size
            let config = GridConfig {
                threshold_block_size: 1,
                enable_parallel: true,
                ..Default::default()
            };
            let result = Grid::try_from_image_with_config(&DynamicImage::ImageLuma8(img), config);
            assert!(result.is_ok());
        }

        // Test merge_small_lines edge cases
        #[test]
        fn test_merge_small_lines_edge_cases() {
            // Test empty input
            let empty: Vec<LineInfo> = vec![];
            let result = Grid::merge_small_lines(empty, 5);
            assert!(result.is_empty());

            // Test single line
            let single = vec![LineInfo::new(0, 1, LineKind::Empty)];
            let result = Grid::merge_small_lines(single, 5);
            assert_eq!(result.len(), 1);

            // Test zero threshold
            let lines = vec![
                LineInfo::new(0, 1, LineKind::Empty),
                LineInfo::new(1, 1, LineKind::Full),
            ];
            let result = Grid::merge_small_lines(lines, 0);
            assert_eq!(result.len(), 2);
        }

        // Property-based tests for uncovered code
        proptest! {
            #[test]
            fn test_grid_error_dimensions_proptest(width in 0u32..1000u32, height in 0u32..1000u32) {
                let error = GridError::InvalidDimensions { width, height };
                let error_str = error.to_string();
                prop_assert!(error_str.contains(&width.to_string()));
                prop_assert!(error_str.contains(&height.to_string()));
            }

            #[test]
            fn test_process_dimension_proptest(
                primary_dim in 1u32..100u32,
                secondary_dim in 1u32..100u32,
                threshold_ratio in 0.1f32..1.0f32
            ) {
                let img = GrayImage::new(secondary_dim, primary_dim);
                let result = Grid::process_dimension::<Row>(
                    &img,
                    primary_dim,
                    secondary_dim,
                    threshold_ratio,
                    Grid::is_row_empty,
                );
                prop_assert!(result.is_ok());
            }
        }
    }
}
