use grider::process_image;

/// Unit tests for the grid generation logic.
#[cfg(test)]
mod tests {
    use grider::*;
    use image::*;
    use insta::assert_yaml_snapshot;

    /// Tests the `is_row_empty` function.
    #[test]
    fn test_is_row_empty() {
        let img =
            GrayImage::from_raw(3, 3, vec![255, 255, 255, 255, 0, 255, 255, 255, 255]).unwrap();
        assert!(is_row_empty(&img, 0, 3));
        assert!(!is_row_empty(&img, 1, 3));
        assert!(is_row_empty(&img, 2, 3));
    }

    /// Tests the `is_column_empty` function.
    #[test]
    fn test_is_column_empty() {
        let img =
            GrayImage::from_raw(3, 3, vec![255, 255, 255, 255, 0, 255, 255, 255, 255]).unwrap();
        assert!(is_column_empty(&img, 0, 3));
        assert!(!is_column_empty(&img, 1, 3));
        assert!(is_column_empty(&img, 2, 3));
    }

    /// Tests the `process_lines` function with a small image.
    #[test]
    fn test_process_lines_small_image() {
        let img =
            GrayImage::from_raw(3, 3, vec![255, 255, 255, 255, 0, 255, 255, 255, 255]).unwrap();
        let rows: Vec<Row> = process_lines(&img, 3, |y| is_row_empty(&img, y, 3));

        // Assert YAML snapshot with a custom name
        assert_yaml_snapshot!("process_lines_small_image", rows);
    }

    /// Tests the `process_lines` function with a larger image.
    #[test]
    fn test_process_lines_large_image() {
        let img = GrayImage::from_fn(10, 10, |_x, y| {
            if y < 5 {
                Luma([255u8]) // First 5 rows are empty
            } else {
                Luma([0u8]) // Last 5 rows are full
            }
        });
        let rows: Vec<Row> = process_lines(&img, 10, |y| is_row_empty(&img, y, 10));

        // Assert YAML snapshot with a custom name
        assert_yaml_snapshot!("process_lines_large_image", rows);
    }

    /// Tests the `process_image` function.
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

    /// Tests the `process_image` function with redactions.
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

        // Assert YAML snapshot with redactions
        assert_yaml_snapshot!("process_image_with_redactions", grid, {
            ".rows[0].y" => 0, // Redact the first row's y-coordinate
            ".rows[1].y" => 5, // Redact the second row's y-coordinate
        });
    }

    /// Tests the `process_lines` function with inline snapshots.
    #[test]
    fn test_process_lines_inline_snapshot() {
        let img =
            GrayImage::from_raw(3, 3, vec![255, 255, 255, 255, 0, 255, 255, 255, 255]).unwrap();
        let rows: Vec<Row> = process_lines(&img, 3, |y| is_row_empty(&img, y, 3));

        // Assert inline YAML snapshot
        assert_yaml_snapshot!(rows, @r"
        - y: 0
          height: 1
          kind: Empty
        - y: 1
          height: 1
          kind: Full
        - y: 2
          height: 1
          kind: Empty
        ");
    }
}

fn main() {
    // Replace with the path to your image file
    let image_path = "tests/13.png";

    // Open the image file
    match image::open(image_path) {
        Ok(img) => {
            // Process the image
            let grid = process_image(img.clone());

            // Print the grid (or use it as needed)
            println!("{:?}", grid);

            // Save the image with grid lines for debugging
            grider::debug::save_image_with_grid(&img, &grid, "output_with_grid.png");
        }
        Err(e) => {
            eprintln!("Failed to open image: {}", e);
        }
    }
}
