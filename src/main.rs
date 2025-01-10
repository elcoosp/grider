use anyhow::{Context, Result};
use grider::{Grid, GridConfig};
use image::GenericImageView;

fn main() -> Result<()> {
    // Replace with the path to your image file
    let image_path = "tests/large.png";

    // Open and process the image
    let img = image::open(image_path).context("Failed to open image")?;

    // Validate image dimensions
    let (width, height) = img.dimensions();
    if width == 0 || height == 0 {
        anyhow::bail!("Invalid image dimensions: {}x{}", width, height);
    }
    // Use debug features under feature flag
    #[cfg(feature = "debug")]
    {
        // Load configuration from environment or config file
        let config = GridConfig::default();
        // Process the image with configuration
        let grid = Grid::try_from_image_with_config(&img, config)?;

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
    #[ignore]
    #[test]
    fn test_adaptive_threshold_behavior() {
        // Create a 10x10 grayscale image with a checkerboard pattern
        let img = GrayImage::from_fn(10, 10, |x, y| {
            if (x + y) % 2 == 0 {
                Luma([255u8]) // White pixel
            } else {
                Luma([0u8]) // Black pixel
            }
        });

        // Convert the image to a DynamicImage for processing
        let dynamic_img = DynamicImage::ImageLuma8(img);

        // Process the image
        let grid: Grid = (&dynamic_img).try_into().unwrap();

        // Debugging: Print the grid
        println!("Grid Rows: {:?}", grid.rows);
        println!("Grid Columns: {:?}", grid.columns);

        // Verify that the grid reflects the expected behavior after adaptive thresholding
        // In a checkerboard pattern, we expect alternating rows and columns to be marked as Full or Empty
        assert_eq!(grid.rows.len(), 2); // Expect 2 rows: one Full, one Empty
        assert_eq!(grid.columns.len(), 2); // Expect 2 columns: one Full, one Empty

        // Check the first row
        assert_eq!(grid.rows[0].kind, LineKind::Full); // First row should be Full (contains black pixels)
        assert_eq!(grid.rows[0].height, 5); // Height of the first row group

        // Check the second row
        assert_eq!(grid.rows[1].kind, LineKind::Empty); // Second row should be Empty (all white pixels)
        assert_eq!(grid.rows[1].height, 5); // Height of the second row group

        // Check the first column
        assert_eq!(grid.columns[0].kind, LineKind::Full); // First column should be Full (contains black pixels)
        assert_eq!(grid.columns[0].width, 5); // Width of the first column group

        // Check the second column
        assert_eq!(grid.columns[1].kind, LineKind::Empty); // Second column should be Empty (all white pixels)
        assert_eq!(grid.columns[1].width, 5); // Width of the second column group
    }
    #[test]
    fn test_is_row_empty_2() {
        // Create a 10x10 grayscale image with all pixels set to white (255)
        let img = GrayImage::from_pixel(10, 10, Luma([255]));

        // Check that all rows are empty
        for y in 0..10 {
            assert!(is_row_empty(&img, y, 10));
        }

        // Set one pixel in the first row to black (0)
        let mut img = img;
        img.put_pixel(5, 0, Luma([0]));

        // Check that the first row is no longer empty
        assert!(!is_row_empty(&img, 0, 10));
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
        let merged_lines = merge_small_lines(lines, 4);

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
            assert!(is_column_empty(&img, x, 10));
        }

        // Set one pixel in the first column to black (0)
        let mut img = img;
        img.put_pixel(0, 5, Luma([0]));

        // Check that the first column is no longer empty
        assert!(!is_column_empty(&img, 0, 10));
    }

    #[test]
    fn test_is_row_empty() {
        let img =
            GrayImage::from_raw(3, 3, vec![255, 255, 255, 255, 0, 255, 255, 255, 255]).unwrap();
        assert!(is_row_empty(&img, 0, 3));
        assert!(!is_row_empty(&img, 1, 3));
        assert!(is_row_empty(&img, 2, 3));
    }
    proptest! {
        #[test]
        fn test_is_row_empty_proptest(width in 1..100u32, height in 1..100u32, y in 0..100u32) {
            // Create a grayscale image with all pixels set to white (255)
            let img = GrayImage::from_pixel(width, height, Luma([255]));

            // If y is within the image height, the row should be empty
            if y < height {
                assert!(is_row_empty(&img, y, width));
            }
        }

        #[test]
        fn test_is_column_empty_proptest(width in 1..100u32, height in 1..100u32, x in 0..100u32) {
            // Create a grayscale image with all pixels set to white (255)
            let img = GrayImage::from_pixel(width, height, Luma([255]));

            // If x is within the image width, the column should be empty
            if x < width {
                assert!(is_column_empty(&img, x, height));
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
            let merged_lines = merge_small_lines(lines .clone(), threshold);

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
            let lines = collect_all_lines(length, &is_empty);

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
        assert!(is_column_empty(&img, 0, 3));
        assert!(!is_column_empty(&img, 1, 3));
        assert!(is_column_empty(&img, 2, 3));
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
}
