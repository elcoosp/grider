//! This module provides functionality for processing images into grids of rows and columns.
//! It uses the `image` and `imageproc` crates for image manipulation and `insta` for snapshot testing.
//! The `serde` feature is used to enable serialization for snapshot testing.
use image::*;
use imageproc::{contrast::adaptive_threshold, drawing::draw_line_segment_mut};
use rayon::prelude::*;

/// Represents the kind of a line (row or column).
///
/// A line can be either [`LineKind::Empty`] (fully white) or [`LineKind::Full`] (contains non-white pixels).
#[derive(Debug, PartialEq, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub enum LineKind {
    Empty,
    Full,
}

/// Represents a row in the grid.
///
/// A row is defined by its starting y-coordinate (`y`), height (`height`), and [`LineKind`].
#[derive(Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct Row {
    y: u32,
    height: u32,
    kind: LineKind,
}

/// Represents a column in the grid.
///
/// A column is defined by its starting x-coordinate (`x`), width (`width`), and [`LineKind`].
#[derive(Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct Column {
    x: u32,
    width: u32,
    kind: LineKind,
}

/// Represents the grid composed of [`Row`]s and [`Column`]s.
///
/// The grid is generated by processing an image and grouping consecutive rows and columns
/// based on their [`LineKind`].
#[derive(Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct Grid {
    rows: Vec<Row>,
    columns: Vec<Column>,
}

/// Trait to create lines (rows or columns).
///
/// This trait is implemented for both [`Row`] and [`Column`] to allow generic processing of lines.
pub trait LineTrait {
    fn new(start: u32, length: u32, kind: LineKind) -> Self;
}

impl LineTrait for Row {
    fn new(start: u32, length: u32, kind: LineKind) -> Self {
        Row {
            y: start,
            height: length,
            kind,
        }
    }
}

impl LineTrait for Column {
    fn new(start: u32, length: u32, kind: LineKind) -> Self {
        Column {
            x: start,
            width: length,
            kind,
        }
    }
}

/// Checks if a row is empty (all pixels are white).
///
/// # Arguments
/// * `img` - The grayscale image to check.
/// * `y` - The y-coordinate of the row.
/// * `width` - The width of the image.
///
/// # Returns
/// `true` if the row is empty (fully white), otherwise `false`.
pub fn is_row_empty(img: &GrayImage, y: u32, width: u32) -> bool {
    (0..width).all(|x| img.get_pixel(x, y).channels()[0] == 255)
}

/// Checks if a column is empty (all pixels are white).
///
/// # Arguments
/// * `img` - The grayscale image to check.
/// * `x` - The x-coordinate of the column.
/// * `height` - The height of the image.
///
/// # Returns
/// `true` if the column is empty (fully white), otherwise `false`.
pub fn is_column_empty(img: &GrayImage, x: u32, height: u32) -> bool {
    (0..height).all(|y| img.get_pixel(x, y).channels()[0] == 255)
}

/// Processes lines (rows or columns) and groups them by their [`LineKind`].
///
/// This function merges adjacent lines of the same kind and uses a dynamic threshold
/// to ignore lines smaller than a fraction of the average size.
///
/// # Arguments
/// * `img` - The grayscale image to process.
/// * `length` - The length of the lines (height for rows, width for columns).
/// * `is_empty` - A function to check if a line is empty.
///
/// # Returns
/// A vector of lines grouped by their [`LineKind`].
pub fn process_lines<T>(
    img: &GrayImage,
    length: u32,
    is_empty: impl Fn(u32) -> bool + Sync,
) -> Vec<T>
where
    T: LineTrait + Send,
{
    // Step 1: Collect all lines without grouping (in parallel)
    let all_lines: Vec<(u32, u32, LineKind)> = (0..length)
        .into_par_iter()
        .map(|i| {
            let kind = if is_empty(i) {
                LineKind::Empty
            } else {
                LineKind::Full
            };
            (i, 1, kind)
        })
        .collect();

    // Step 2: Calculate the average size of all lines
    let average_size = calculate_average_line_size(&all_lines);
    // println!("Average size: {}", average_size);

    // Step 3: Merge lines smaller than the threshold
    let threshold = (average_size * 8) / 10; // 80% of the average size
    let merged_lines = merge_lines(all_lines, threshold);

    // Step 4: Convert merged lines into the appropriate type
    merged_lines
        .into_par_iter() // Parallelize the conversion
        .map(|(start, length, kind)| T::new(start, length, kind))
        .collect()
}

/// Collects all lines without grouping.
///
/// This function is replaced by the parallelized version in `process_lines`.
fn collect_lines(length: u32, is_empty: &impl Fn(u32) -> bool) -> Vec<(u32, u32, LineKind)> {
    (0..length)
        .map(|i| {
            let kind = if is_empty(i) {
                LineKind::Empty
            } else {
                LineKind::Full
            };
            (i, 1, kind)
        })
        .collect()
}

/// Calculates the average size of all lines.
///
/// # Arguments
/// * `lines` - A vector of tuples representing the lines: (start, length, kind).
///
/// # Returns
/// The average size of the lines.
fn calculate_average_line_size(lines: &[(u32, u32, LineKind)]) -> u32 {
    let total_size: u32 = lines.par_iter().map(|&(_, length, _)| length).sum();
    if lines.is_empty() {
        0
    } else {
        total_size / lines.len() as u32
    }
}

/// Merges lines smaller than the threshold.
///
/// # Arguments
/// * `lines` - A vector of tuples representing the lines: (start, length, kind).
/// * `threshold` - The threshold for merging lines.
///
/// # Returns
/// A vector of merged lines: (start, length, kind).
fn merge_lines(lines: Vec<(u32, u32, LineKind)>, threshold: u32) -> Vec<(u32, u32, LineKind)> {
    let mut merged_lines = Vec::with_capacity(lines.len()); // Pre-allocate memory
    let mut current_start = lines[0].0;
    let mut current_length = lines[0].1;
    let mut current_kind = lines[0].2.clone();

    for (start, length, kind) in lines.into_iter().skip(1) {
        if current_length < threshold || length < threshold {
            // Merge with the previous line if either is smaller than the threshold
            current_length += length;
        } else {
            // Push the merged line
            merged_lines.push((current_start, current_length, current_kind));
            current_start = start;
            current_length = length;
            current_kind = kind;
        }
    }

    // Push the last merged line
    merged_lines.push((current_start, current_length, current_kind));
    merged_lines
}
/// Processes the image and generates the [`Grid`].
///
/// This function converts the image to grayscale, applies adaptive thresholding,
/// and processes it to generate the [`Grid`] of [`Row`]s and [`Column`]s.
///
/// # Arguments
/// * `image` - The input image to process.
///
/// # Returns
/// A [`Grid`] representing the rows and columns of the image.
pub fn process_image(image: DynamicImage) -> Grid {
    // Convert the image to grayscale
    let img = image.to_luma8();

    // Apply adaptive thresholding
    let binarized_img = adaptive_threshold(&img, 12); // Adjust the radius as needed

    // Process rows and columns in parallel
    let (width, height) = binarized_img.dimensions();
    let rows: Vec<Row> = process_lines(&binarized_img, height, |y| {
        is_row_empty(&binarized_img, y, width)
    });

    let columns: Vec<Column> = process_lines(&binarized_img, width, |x| {
        is_column_empty(&binarized_img, x, height)
    });

    // Create the Grid
    Grid { rows, columns }
}

/// Debug module for visualizing the grid on the image.
pub mod debug {
    use super::*;

    /// Saves the image with grid lines for debugging.
    ///
    /// This function draws horizontal lines for [`Row`]s and vertical lines for [`Column`]s
    /// on the image and saves it to the specified path.
    ///
    /// # Arguments
    /// * `image` - The input image.
    /// * `grid` - The [`Grid`] to visualize.
    /// * `output_path` - The path to save the output image.
    pub fn save_image_with_grid(image: &DynamicImage, grid: &Grid, output_path: &str) {
        // Convert the image to RGBA for drawing
        let mut rgba_img = image.to_rgba8();
        let (w, h) = (rgba_img.width() as f32, rgba_img.height() as f32);

        // Draw horizontal lines for rows
        for row in &grid.rows {
            let y = row.y + row.height;
            draw_line_segment_mut(
                &mut rgba_img,
                (0.0, y as f32),
                (w, y as f32),
                Rgba([255, 0, 0, 255]), // Red color for rows
            );
        }

        // Draw vertical lines for columns
        for column in &grid.columns {
            let x = column.x + column.width;
            draw_line_segment_mut(
                &mut rgba_img,
                (x as f32, 0.0),
                (x as f32, h),
                Rgba([0, 0, 255, 255]), // Blue color for columns
            );
        }

        // Save the image with grid lines
        rgba_img.save(output_path).unwrap();
        // println!("Image with grid lines saved to {}", output_path);
    }
}
