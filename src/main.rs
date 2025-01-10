use anyhow::{Context, Result};
use grider::{debug::save_image_with_grid, process_image};

fn main() -> Result<()> {
    // Replace with the path to your image file
    let image_path = "tests/13.png";

    // Open the image file
    let img = image::open(image_path).context("Failed to open image")?;

    // Process the image
    let grid = process_image(img.clone());

    // Save the image with grid lines for debugging
    save_image_with_grid(&img, &grid, "output_with_grid.png");

    Ok(())
}

/// Unit tests for the grid generation logic.
#[cfg(test)]
mod tests {
    use grider::*;
    use image::*;
    use insta::assert_yaml_snapshot;
    use pretty_assertions::assert_eq;
    use proptest::proptest;
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
        debug::save_image_with_grid(&dynamic_img, &grid, output_path);

        // Check that the file was created
        assert!(std::path::Path::new(output_path).exists());

        // Clean up the test file
        std::fs::remove_file(output_path).unwrap();
    }
    #[test]
    fn test_adaptive_threshold() {
        // Create a 10x10 grayscale image with a gradient from black to white
        let img = GrayImage::from_fn(10, 10, |x, _y| Luma([(x * 25) as u8]));

        // Process the image
        let grid = process_image(DynamicImage::ImageLuma8(img));

        // Check that the grid has been generated correctly
        // For a gradient image, we expect some rows and columns to be marked as Full
        assert!(!grid.rows.is_empty());
        assert!(!grid.columns.is_empty());
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
    fn test_process_lines() {
        // Create a 10x10 grayscale image with alternating empty and full rows
        let mut img = GrayImage::new(10, 10);
        for y in 0..10 {
            let pixel_value = if y % 2 == 0 { 255 } else { 0 }; // Even rows are empty, odd rows are full
            for x in 0..10 {
                img.put_pixel(x, y, Luma([pixel_value]));
            }
        }

        // Process rows
        let rows: SmallVecLine<Row> = process_lines(&img, 10, |y| is_row_empty(&img, y, 10));

        // Expected result:
        // - Even rows are empty (LineKind::Empty)
        // - Odd rows are full (LineKind::Full)
        assert_eq!(
            rows,
            SmallVecLine::from_vec(vec![
                Row::new(LineInfo::new(0, 1, LineKind::Empty)),
                Row::new(LineInfo::new(1, 1, LineKind::Full)),
                Row::new(LineInfo::new(2, 1, LineKind::Empty)),
                Row::new(LineInfo::new(3, 1, LineKind::Full)),
                Row::new(LineInfo::new(4, 1, LineKind::Empty)),
                Row::new(LineInfo::new(5, 1, LineKind::Full)),
                Row::new(LineInfo::new(6, 1, LineKind::Empty)),
                Row::new(LineInfo::new(7, 1, LineKind::Full)),
                Row::new(LineInfo::new(8, 1, LineKind::Empty)),
                Row::new(LineInfo::new(9, 1, LineKind::Full)),
            ])
        );
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
        fn test_is_row_empty_prop(width in 1..100u32, height in 1..100u32, y in 0..100u32) {
            let img = GrayImage::from_pixel(width, height, Luma([255]));
            if y < height {
                assert!(is_row_empty(&img, y, width));
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
    fn test_process_lines_small_image() {
        let img =
            GrayImage::from_raw(3, 3, vec![255, 255, 255, 255, 0, 255, 255, 255, 255]).unwrap();
        let rows: SmallVecLine<Row> = process_lines(&img, 3, |y| is_row_empty(&img, y, 3));

        // Assert YAML snapshot with a custom name
        assert_yaml_snapshot!("process_lines_small_image", rows);
    }

    #[test]
    fn test_process_lines_large_image() {
        let img = GrayImage::from_fn(10, 10, |_x, y| {
            if y < 5 {
                Luma([255u8]) // First 5 rows are empty
            } else {
                Luma([0u8]) // Last 5 rows are full
            }
        });
        let rows: SmallVecLine<Row> = process_lines(&img, 10, |y| is_row_empty(&img, y, 10));

        // Assert YAML snapshot with a custom name
        assert_yaml_snapshot!("process_lines_large_image", rows);
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
        let grid = process_image(DynamicImage::ImageLuma8(img));

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
        let grid = process_image(DynamicImage::ImageLuma8(img));

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
    fn test_process_lines_inline_snapshot() {
        let img =
            GrayImage::from_raw(3, 3, vec![255, 255, 255, 255, 0, 255, 255, 255, 255]).unwrap();
        let rows: SmallVecLine<Row> = process_lines(&img, 3, |y| is_row_empty(&img, y, 3));

        // Assert inline YAML snapshot
        assert_yaml_snapshot!(rows, @r###"
        - y: 0
          height: 1
          kind: Empty
        - y: 1
          height: 1
          kind: Full
        - y: 2
          height: 1
          kind: Empty
        "###);
    }
}
