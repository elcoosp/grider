//! This module provides functionality for processing images into grids of rows and columns.
//! It uses the `image` and `imageproc` crates for image manipulation and `insta` for snapshot testing.

use image::*;
use imageproc::{contrast::adaptive_threshold, drawing::draw_line_segment_mut};
use smallvec::SmallVec;
use thiserror::Error;

// Determined through benchmarking typical use cases
const DEFAULT_SMALLVEC_SIZE: usize = 32;
const DEFAULT_THRESHOLD_BLOCK_SIZE: u32 = 12;
const DEFAULT_MERGE_THRESHOLD_RATIO: f32 = 0.8;

#[derive(Error, Debug)]
pub enum GridError {
    #[error("Failed to convert image: {0}")]
    ImageConversionError(String),

    #[error("Failed to apply threshold: {0}")]
    ThresholdingError(String),

    #[error("Failed to detect lines: {0}")]
    LineDetectionError(String),

    #[error("Invalid image dimensions: width={width}, height={height}")]
    InvalidDimensions { width: u32, height: u32 },
}

/// A type alias for SmallVec with an optimized stack-allocated buffer size.
pub type SmallVecLine<T> = SmallVec<[T; DEFAULT_SMALLVEC_SIZE]>;

/// Configuration for grid processing.
#[derive(Debug, Clone)]
pub struct GridConfig {
    /// Block size for adaptive thresholding (default: 12)
    pub threshold_block_size: u32,
    /// Ratio for merging small lines (default: 0.8)
    pub merge_threshold_ratio: f32,
    /// Enable parallel processing (default: true)
    pub enable_parallel: bool,
}

impl Default for GridConfig {
    fn default() -> Self {
        Self::new(12, 0.8, true)
    }
}
impl GridConfig {
    pub fn new(
        threshold_block_size: u32,
        merge_threshold_ratio: f32,
        enable_parallel: bool,
    ) -> Self {
        Self {
            threshold_block_size: threshold_block_size.max(3), // Minimum block size
            merge_threshold_ratio,
            enable_parallel,
        }
    }
}
/// Represents the kind of a line (row or column).
#[derive(Debug, PartialEq, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub enum LineKind {
    Empty,
    Full,
}

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct LineInfo {
    pub start: u32,
    pub length: u32,
    pub kind: LineKind,
}

impl LineInfo {
    pub fn new(start: u32, length: u32, kind: LineKind) -> Self {
        Self {
            start,
            length,
            kind,
        }
    }
}

#[derive(Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct Row {
    pub y: u32,
    pub height: u32,
    pub kind: LineKind,
}

#[derive(Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct Column {
    pub x: u32,
    pub width: u32,
    pub kind: LineKind,
}

#[derive(Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct Grid {
    pub rows: SmallVecLine<Row>,
    pub columns: SmallVecLine<Column>,
}
impl Grid {
    /// Creates a new Grid from an image with custom configuration.
    ///
    /// # Arguments
    /// * `image` - The input image to process
    /// * `config` - Configuration parameters for grid processing
    ///
    /// # Returns
    /// A Result containing either the processed Grid or a GridError
    ///
    /// # Errors
    /// Returns GridError if:
    /// * Image dimensions are invalid
    /// * Image conversion fails
    /// * Thresholding fails
    /// * Line detection fails
    ///
    /// # Example
    /// ```
    /// use grider::{Grid, GridConfig};
    ///
    /// let img = image::open("tests/large.png").unwrap();
    /// let config = GridConfig::default();
    /// let grid = Grid::try_from_image_with_config(&img, config).unwrap();
    /// ```
    pub fn try_from_image_with_config(
        image: &DynamicImage,
        config: GridConfig,
    ) -> Result<Self, GridError> {
        // Validate image dimensions
        let (width, height) = image.dimensions();
        if width == 0 || height == 0 {
            return Err(GridError::InvalidDimensions { width, height });
        }

        // Convert to grayscale
        let img = image.to_luma8();

        // Apply adaptive thresholding with configured block size
        let binarized_img = adaptive_threshold(&img, config.threshold_block_size);

        // Process rows and columns based on configuration
        let (rows, columns) = if config.enable_parallel {
            Self::process_lines_parallel(&binarized_img, config.merge_threshold_ratio)?
        } else {
            Self::process_lines_sequential(&binarized_img, config.merge_threshold_ratio)?
        };

        Ok(Grid { rows, columns })
    }

    /// Process image lines in parallel using rayon.
    fn process_lines_parallel(
        img: &GrayImage,
        merge_threshold_ratio: f32,
    ) -> Result<(SmallVecLine<Row>, SmallVecLine<Column>), GridError> {
        let (width, height) = img.dimensions();

        // Process rows and columns in parallel
        let result = rayon::join(
            || {
                Self::process_dimension::<Row>(
                    img,
                    height,
                    width,
                    merge_threshold_ratio,
                    Self::is_row_empty,
                )
            },
            || {
                Self::process_dimension::<Column>(
                    img,
                    width,
                    height,
                    merge_threshold_ratio,
                    Self::is_column_empty,
                )
            },
        );

        // Check results and combine
        match result {
            (Ok(rows), Ok(columns)) => Ok((rows, columns)),
            (Err(e), _) | (_, Err(e)) => Err(e),
        }
    }

    /// Process image lines sequentially.
    fn process_lines_sequential(
        img: &GrayImage,
        merge_threshold_ratio: f32,
    ) -> Result<(SmallVecLine<Row>, SmallVecLine<Column>), GridError> {
        let (width, height) = img.dimensions();

        // Process rows first, then columns
        let rows = Self::process_dimension::<Row>(
            img,
            height,
            width,
            merge_threshold_ratio,
            Self::is_row_empty,
        )?;
        let columns = Self::process_dimension::<Column>(
            img,
            width,
            height,
            merge_threshold_ratio,
            Self::is_column_empty,
        )?;

        Ok((rows, columns))
    }

    /// Generic function to process a dimension (rows or columns).
    pub fn process_dimension<T: LineTrait + Send>(
        img: &GrayImage,
        primary_dim: u32,
        secondary_dim: u32,
        merge_threshold_ratio: f32,
        is_empty_fn: impl Fn(&GrayImage, u32, u32) -> bool + Sync,
    ) -> Result<SmallVecLine<T>, GridError> {
        if primary_dim == 0 || secondary_dim == 0 {
            return Err(GridError::InvalidDimensions {
                width: secondary_dim,
                height: primary_dim,
            });
        }

        // Collect initial lines
        let lines = Self::collect_lines(img, primary_dim, secondary_dim, &is_empty_fn)
            .map_err(|e| GridError::LineDetectionError(e.to_string()))?;

        // Calculate threshold for merging
        let average_size = Self::calculate_average_line_size(&lines);
        let merge_threshold = (average_size as f32 * merge_threshold_ratio) as u32;

        // Merge small lines and convert to final type
        let merged = Self::merge_small_lines(lines, merge_threshold);

        Ok(merged.into_iter().map(T::new).collect())
    }

    /// Collects initial lines without merging.
    fn collect_lines(
        img: &GrayImage,
        primary_dim: u32,
        secondary_dim: u32,
        is_empty_fn: &impl Fn(&GrayImage, u32, u32) -> bool,
    ) -> Result<Vec<LineInfo>, GridError> {
        let mut lines = Vec::new();
        let mut current_start = 0;
        let mut current_kind = if is_empty_fn(img, 0, secondary_dim) {
            LineKind::Empty
        } else {
            LineKind::Full
        };
        let mut current_length = 1;

        for i in 1..primary_dim {
            let new_kind = if is_empty_fn(img, i, secondary_dim) {
                LineKind::Empty
            } else {
                LineKind::Full
            };

            if new_kind == current_kind {
                current_length += 1;
            } else {
                lines.push(LineInfo::new(current_start, current_length, current_kind));
                current_start = i;
                current_kind = new_kind;
                current_length = 1;
            }
        }

        // Add the final line
        lines.push(LineInfo::new(current_start, current_length, current_kind));

        Ok(lines)
    }

    /// Calculate average size of lines for threshold determination.
    fn calculate_average_line_size(lines: &[LineInfo]) -> u32 {
        if lines.is_empty() {
            return 0;
        }
        let total: u32 = lines.iter().map(|l| l.length).sum();
        total / lines.len() as u32
    }

    /// Merges lines smaller than the threshold.
    ///
    /// # Arguments
    /// * `lines` - A vector of [`LineInfo`] representing the lines.
    /// * `threshold` - The threshold for merging lines.
    ///
    /// # Returns
    /// A vector of merged [`LineInfo`].
    pub fn merge_small_lines(lines: Vec<LineInfo>, threshold: u32) -> SmallVecLine<LineInfo> {
        if lines.is_empty() {
            return SmallVecLine::new();
        }
        let mut merged_lines = SmallVecLine::new();
        let mut current_start = lines[0].start;
        let mut current_length = lines[0].length;
        let mut current_kind = lines[0].kind.clone();

        for line in lines.into_iter().skip(1) {
            if current_length < threshold || line.length < threshold {
                // Merge with the previous line if either is smaller than the threshold
                current_length += line.length;
            } else {
                // Push the merged line
                merged_lines.push(LineInfo::new(current_start, current_length, current_kind));
                current_start = line.start;
                current_length = line.length;
                current_kind = line.kind;
            }
        }

        // Push the last merged line
        merged_lines.push(LineInfo::new(current_start, current_length, current_kind));
        merged_lines
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

    /// Collects all lines without grouping.
    ///
    /// # Arguments
    /// * `length` - The length of the lines (height for rows, width for columns).
    /// * `is_empty` - A function to check if a line is empty.
    ///
    /// # Returns
    /// A vector of [`LineInfo`] representing the lines.
    pub fn collect_all_lines(length: u32, is_empty: &impl Fn(u32) -> bool) -> Vec<LineInfo> {
        let mut lines = Vec::new();
        let mut current_start = 0;
        let mut current_kind = if is_empty(0) {
            LineKind::Empty
        } else {
            LineKind::Full
        };
        let mut current_length = 1;

        for i in 1..length {
            let new_kind = if is_empty(i) {
                LineKind::Empty
            } else {
                LineKind::Full
            };

            if new_kind == current_kind {
                current_length += 1;
            } else {
                lines.push(LineInfo::new(current_start, current_length, current_kind));
                current_start = i;
                current_kind = new_kind;
                current_length = 1;
            }
        }

        // Push the last line
        lines.push(LineInfo::new(current_start, current_length, current_kind));
        lines
    }
}
pub trait LineTrait {
    fn new(line: LineInfo) -> Self;
}

impl LineTrait for Row {
    fn new(line: LineInfo) -> Self {
        Row {
            y: line.start,
            height: line.length,
            kind: line.kind,
        }
    }
}

impl LineTrait for Column {
    fn new(line: LineInfo) -> Self {
        Column {
            x: line.start,
            width: line.length,
            kind: line.kind,
        }
    }
}

impl TryFrom<DynamicImage> for Grid {
    type Error = GridError;

    fn try_from(image: DynamicImage) -> Result<Self, Self::Error> {
        // Delegate to the &DynamicImage implementation
        TryFrom::try_from(&image)
    }
}

impl TryFrom<&DynamicImage> for Grid {
    type Error = GridError;
    fn try_from(image: &DynamicImage) -> Result<Self, Self::Error> {
        Grid::try_from_image_with_config(&image, GridConfig::default())
    }
}
#[macro_export]
macro_rules! make_line {
    // For Rows
    (Row, ($y:expr, $height:expr, $kind:expr)) => {
        Row::new(LineInfo::new($y, $height, $kind))
    };
    (Row, ($y:expr, $height:expr)) => {
        Row::new(LineInfo::new($y, $height, LineKind::Empty)) // Default to Empty
    };

    // For Columns
    (Column, ($x:expr, $width:expr, $kind:expr)) => {
        Column::new(LineInfo::new($x, $width, $kind))
    };
    (Column, ($x:expr, $width:expr)) => {
        Column::new(LineInfo::new($x, $width, LineKind::Empty)) // Default to Empty
    };
}

#[macro_export]
macro_rules! make_grid {
    // Match rows and columns as separate lists with tuple syntax
    (rows: [$($row:tt,)*], columns: [$($col:tt,)*]) => {
        Grid {
            rows: SmallVecLine::from_vec(vec![
                $(make_line!(Row, $row)),*
            ]),
            columns: SmallVecLine::from_vec(vec![
                $(make_line!(Column, $col)),*
            ]),
        }
    };
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
    }
}
