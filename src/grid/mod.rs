pub mod components;
pub mod config;
pub mod constants;
pub mod lines;
pub mod traits;

pub mod grid_like;
pub mod grid_subset;
pub use components::*;
pub use config::*;
pub use constants::*;
pub use grid_like::*;
pub use grid_subset::*;
pub use lines::*;
pub use traits::*;

use image::*;
use imageproc::contrast::adaptive_threshold;
use imageproc::rect::Rect;
use smallvec::SmallVec;
use thiserror::Error;
use tracing::*;

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

    #[error("Row not found at y={y}")]
    RowNotFound { y: u32 },

    #[error("Column not found at x={x}")]
    ColumnNotFound { x: u32 },
}

/// Represents the grid of rows and columns extracted from an image.
///
/// # Example 1
/// ```
/// use grider::{Grid, GridConfig};
/// use image::open;
///
/// let img = open("tests/large.png").unwrap();
/// let config = GridConfig::default();
/// let grid = Grid::try_from_image_with_config(&img, config).unwrap();
/// ```
/// # Example 2
///
/// ```
/// use grider::{Grid, GridConfig};
/// use image::open;
///
/// let img = open("tests/large.png").unwrap();
/// let config = GridConfig::default();
/// let grid = Grid::try_from_image_with_config(&img, config).unwrap();
///
/// // Automatically filter out the smallest rows
/// let filtered_grid = grid.filter_smallest_rows();
///
/// // Automatically filter out the biggest rows
/// let filtered_grid = grid.filter_biggest_rows();
///
/// // Automatically filter out the smallest columns
/// let filtered_grid = grid.filter_smallest_columns();
///
/// // Automatically filter out the biggest columns
/// let filtered_grid = grid.filter_biggest_columns();
/// ```
#[derive(Debug, PartialEq, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct Grid {
    pub rows: SmallVecLine<Row>,
    pub columns: SmallVecLine<Column>,
}

impl Grid {
    /// Creates a new `Grid` from rows and columns.
    pub fn new(rows: SmallVecLine<Row>, columns: SmallVecLine<Column>) -> Self {
        Self { rows, columns }
    }

    /// Creates a `GridSubset` referencing specific rows and columns.
    pub fn create_subset<'a>(
        &'a self,
        row_indices: &[usize],
        column_indices: &[usize],
    ) -> GridSubset<'a> {
        let rows = row_indices.iter().map(|&i| &self.rows[i]).collect();
        let columns = column_indices.iter().map(|&i| &self.columns[i]).collect();
        GridSubset::new(rows, columns)
    }
    /// Helper function to calculate min and max bounds based on size and tolerance.
    fn calculate_bounds(size: u32, tolerance: f32) -> (f32, f32) {
        let min = size as f32 * (1.0 - tolerance);
        let max = size as f32 * (1.0 + tolerance);
        (min, max)
    }

    pub fn select_lines_by_size<T, F>(
        lines: &[T],
        size: Option<u32>,
        tolerance: f32,
        get_size: F,
    ) -> SmallVecLine<usize>
    where
        F: Fn(&T) -> u32,
    {
        if let Some(size) = size {
            let (min_size, max_size) = Self::calculate_bounds(size, tolerance);
            lines
                .iter()
                .enumerate()
                .filter(|(_, line)| {
                    let line_size = get_size(line) as f32;
                    line_size >= min_size && line_size <= max_size
                })
                .map(|(index, _)| index)
                .collect()
        } else {
            (0..lines.len()).collect()
        }
    }
    pub fn select_smallest_rows(&self) -> GridSubset {
        let height = self.smallest_row_height().unwrap_or(0);
        let selected_indices =
            Self::select_lines_by_size(&self.rows, Some(height), DEFAULT_TOLERANCE, |r| r.height);

        // Collect references to the original grid's rows
        let rows: SmallVecLine<&Row> = selected_indices
            .iter()
            .map(|index| &self.rows[*index])
            .collect();

        GridSubset::new(rows, self.columns.iter().collect())
    }

    pub fn select_biggest_rows(&self) -> GridSubset {
        let height = self.biggest_row_height().unwrap_or(0);
        let selected_indices =
            Self::select_lines_by_size(&self.rows, Some(height), DEFAULT_TOLERANCE, |r| r.height);

        // Collect references to the original grid's rows
        let rows: SmallVecLine<&Row> = selected_indices
            .iter()
            .map(|index| &self.rows[*index])
            .collect();

        GridSubset::new(rows, self.columns.iter().collect())
    }
    pub fn select_smallest_columns(&self) -> GridSubset {
        let width = self.smallest_column_width().unwrap_or(0);
        let selected_indices =
            Self::select_lines_by_size(&self.columns, Some(width), DEFAULT_TOLERANCE, |c| c.width);

        let selected_columns: SmallVecLine<&Column> = selected_indices
            .iter()
            .map(|index| &self.columns[*index])
            .collect();

        GridSubset::new(self.rows.iter().collect(), selected_columns)
    }

    pub fn select_biggest_columns(&self) -> GridSubset {
        let width = self.biggest_column_width().unwrap_or(0);
        let selected_indices =
            Self::select_lines_by_size(&self.columns, Some(width), DEFAULT_TOLERANCE, |c| c.width);

        let selected_columns: SmallVecLine<&Column> = selected_indices
            .iter()
            .map(|index| &self.columns[*index])
            .collect();

        GridSubset::new(self.rows.iter().collect(), selected_columns)
    }
    /// Filters out cells where both the row and column are of kind `LineKind::Full`.
    ///
    /// # Returns
    /// A new `Grid` with the fullest cells removed.
    pub fn select_most_full_cells(&self) -> Grid {
        // Step 1: Calculate the "fullness" score for each cell
        let mut cell_fullness = Vec::new();

        for (row_idx, row) in self.rows.iter().enumerate() {
            for (col_idx, col) in self.columns.iter().enumerate() {
                // Calculate the "fullness" score for the cell
                let row_fullness = if row.kind == LineKind::Full { 1 } else { 0 };
                let col_fullness = if col.kind == LineKind::Full { 1 } else { 0 };
                let cell_score = row_fullness + col_fullness;

                // Store the cell's score and its position
                cell_fullness.push((row_idx, col_idx, cell_score));
            }
        }

        // Step 2: Find the maximum "fullness" score
        let max_score = cell_fullness
            .iter()
            .map(|&(_, _, score)| score)
            .max()
            .unwrap_or(0);

        // Step 3: Identify rows and columns to keep (those part of the most full cells)
        let rows_to_keep: SmallVecLine<Row> = self
            .rows
            .iter()
            .enumerate()
            .filter(|&(row_idx, _)| {
                // Keep rows that are part of the most full cells
                cell_fullness
                    .iter()
                    .any(|&(r_idx, _, score)| r_idx == row_idx && score == max_score)
            })
            .map(|(_, row)| row.clone())
            .collect();

        let columns_to_keep: SmallVecLine<Column> = self
            .columns
            .iter()
            .enumerate()
            .filter(|&(col_idx, _)| {
                // Keep columns that are part of the most full cells
                cell_fullness
                    .iter()
                    .any(|&(_, c_idx, score)| c_idx == col_idx && score == max_score)
            })
            .map(|(_, col)| col.clone())
            .collect();

        // Step 4: Return the filtered grid
        Grid {
            rows: rows_to_keep,
            columns: columns_to_keep,
        }
    }

    /// Finds the smallest height among all rows in the grid.
    ///
    /// # Returns
    /// The smallest height of the rows, or `None` if there are no rows.
    pub fn smallest_row_height(&self) -> Option<u32> {
        self.rows.iter().map(|row| row.height).min()
    }

    /// Finds the biggest height among all rows in the grid.
    ///
    /// # Returns
    /// The biggest height of the rows, or `None` if there are no rows.
    pub fn biggest_row_height(&self) -> Option<u32> {
        self.rows.iter().map(|row| row.height).max()
    }

    /// Finds the smallest width among all columns in the grid.
    ///
    /// # Returns
    /// The smallest width of the columns, or `None` if there are no columns.
    pub fn smallest_column_width(&self) -> Option<u32> {
        self.columns.iter().map(|col| col.width).min()
    }

    /// Finds the biggest width among all columns in the grid.
    ///
    /// # Returns
    /// The biggest width of the columns, or `None` if there are no columns.
    pub fn biggest_column_width(&self) -> Option<u32> {
        self.columns.iter().map(|col| col.width).max()
    }
    /// Creates a new `Grid` from an image with custom configuration.
    ///
    /// # Example
    /// ```
    /// use grider::{Grid, GridConfig};
    /// use image::open;
    ///
    /// let img = open("tests/large.png").unwrap();
    /// let config = GridConfig::default();
    /// let grid = Grid::try_from_image_with_config(&img, config).unwrap();
    /// ```
    pub fn try_from_image_with_config(
        image: &DynamicImage,
        config: GridConfig,
    ) -> Result<Self, GridError> {
        trace!("Processing image with config: {:?}", config);
        // Validate image dimensions
        let (width, height) = image.dimensions();
        if width == 0 || height == 0 {
            error!(
                "Invalid image dimensions: width={}, height={}",
                width, height
            );
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

    /// Returns an iterator over all rows in the grid.
    ///
    /// # Example
    /// ```
    /// use grider::{Grid, GridConfig};
    /// use image::open;
    ///
    /// let img = open("tests/large.png").unwrap();
    /// let config = GridConfig::default();
    /// let grid = Grid::try_from_image_with_config(&img, config).unwrap();
    ///
    /// for row in grid.rows() {
    ///     println!("Row at y: {}", row.y);
    /// }
    /// ```
    pub fn rows(&self) -> impl Iterator<Item = &Row> {
        self.rows.iter()
    }

    /// Returns an iterator over all columns in the grid.
    ///
    /// # Example
    /// ```
    /// use grider::{Grid, GridConfig};
    /// use image::open;
    ///
    /// let img = open("tests/large.png").unwrap();
    /// let config = GridConfig::default();
    /// let grid = Grid::try_from_image_with_config(&img, config).unwrap();
    ///
    /// for column in grid.columns() {
    ///     println!("Column at x: {}", column.x);
    /// }
    /// ```
    pub fn columns(&self) -> impl Iterator<Item = &Column> {
        self.columns.iter()
    }

    /// Returns an iterator over filtered rows based on the predicate.
    ///
    /// # Example
    /// ```
    /// use grider::{Grid, GridConfig, LineKind, Row};
    /// use image::open;
    ///
    /// let img = open("tests/large.png").unwrap();
    /// let config = GridConfig::default();
    /// let grid = Grid::try_from_image_with_config(&img, config).unwrap();
    ///
    /// let filtered_rows: Vec<&Row> = grid.filtered_rows(|row| row.kind == LineKind::Full).collect();
    /// ```
    pub fn filtered_rows<F>(&self, predicate: F) -> impl Iterator<Item = &Row>
    where
        F: Fn(&&Row) -> bool,
    {
        self.rows.iter().filter(predicate)
    }

    /// Returns an iterator over filtered columns based on the predicate.
    ///
    /// # Example
    /// ```
    /// use grider::{Grid, GridConfig, LineKind, Column};
    /// use image::open;
    ///
    /// let img = open("tests/large.png").unwrap();
    /// let config = GridConfig::default();
    /// let grid = Grid::try_from_image_with_config(&img, config).unwrap();
    ///
    /// let filtered_columns: Vec<&Column> = grid.filtered_columns(|col| col.kind == LineKind::Full).collect();
    /// ```
    pub fn filtered_columns<F>(&self, predicate: F) -> impl Iterator<Item = &Column>
    where
        F: Fn(&&Column) -> bool,
    {
        self.columns.iter().filter(predicate)
    }

    /// Counts the number of rows with the specified kind.
    ///
    /// # Example
    /// ```
    /// use grider::{Grid, GridConfig, LineKind};
    /// use image::open;
    ///
    /// let img = open("tests/large.png").unwrap();
    /// let config = GridConfig::default();
    /// let grid = Grid::try_from_image_with_config(&img, config).unwrap();
    ///
    /// let full_row_count = grid.count_rows_by_kind(LineKind::Full);
    /// ```
    pub fn count_rows_by_kind(&self, kind: LineKind) -> usize {
        self.rows.iter().filter(|row| row.kind == kind).count()
    }

    /// Counts the number of columns with the specified kind.
    ///
    /// # Example
    /// ```
    /// use grider::{Grid, GridConfig, LineKind};
    /// use image::open;
    ///
    /// let img = open("tests/large.png").unwrap();
    /// let config = GridConfig::default();
    /// let grid = Grid::try_from_image_with_config(&img, config).unwrap();
    ///
    /// let full_column_count = grid.count_columns_by_kind(LineKind::Full);
    /// ```
    pub fn count_columns_by_kind(&self, kind: LineKind) -> usize {
        self.columns.iter().filter(|col| col.kind == kind).count()
    }

    /// Finds cells based on row and column indices.
    ///
    /// # Example
    /// ```
    /// use grider::{Grid, GridConfig};
    /// use image::open;
    ///
    /// let img = open("tests/large.png").unwrap();
    /// let config = GridConfig::default();
    /// let grid = Grid::try_from_image_with_config(&img, config).unwrap();
    ///
    /// let cells = grid.find_cells(&[1, 2, 3], &[4, 5, 6]);
    /// for cell in cells {
    ///     match cell {
    ///         Ok(c) => println!("Found cell at row {}, column {}", c.row.y, c.column.x),
    ///         Err(e) => eprintln!("Error finding cell: {}", e),
    ///     }
    /// }
    /// ```
    pub fn find_cells<'a>(
        &'a self,
        row_indices: &'a [u32],
        column_indices: &'a [u32],
    ) -> impl Iterator<Item = Result<Cell<'a>, GridError>> + 'a {
        row_indices.iter().flat_map(move |&row_idx| {
            column_indices.iter().map(move |&col_idx| {
                let row = self
                    .find_row(row_idx)
                    .ok_or(GridError::RowNotFound { y: row_idx })?;
                let column = self
                    .find_column(col_idx)
                    .ok_or(GridError::ColumnNotFound { x: col_idx })?;
                Ok(Cell { row, column })
            })
        })
    }

    /// Finds a row by its y-coordinate.
    ///
    /// # Example
    /// ```
    /// use grider::{Grid, GridConfig};
    /// use image::open;
    ///
    /// let img = open("tests/large.png").unwrap();
    /// let config = GridConfig::default();
    /// let grid = Grid::try_from_image_with_config(&img, config).unwrap();
    ///
    /// let row = grid.find_row(0);
    /// assert!(row.is_some());
    /// ```
    pub fn find_row(&self, y: u32) -> Option<&Row> {
        self.rows.iter().find(|row| row.y == y)
    }

    /// Finds a column by its x-coordinate.
    ///
    /// # Example
    /// ```
    /// use grider::{Grid, GridConfig};
    /// use image::open;
    ///
    /// let img = open("tests/large.png").unwrap();
    /// let config = GridConfig::default();
    /// let grid = Grid::try_from_image_with_config(&img, config).unwrap();
    /// println!("{grid:#?}");
    /// let column = grid.find_column(0);
    /// assert!(column.is_some());
    /// ```
    pub fn find_column(&self, x: u32) -> Option<&Column> {
        self.columns.iter().find(|col| col.x == x)
    }

    /// Process image lines in parallel using rayon.
    fn process_lines_parallel(
        img: &GrayImage,
        merge_threshold_ratio: f32,
    ) -> Result<(SmallVecLine<Row>, SmallVecLine<Column>), GridError> {
        trace!("Processing lines in parallel");
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
        trace!("Processing lines sequentially");
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
        debug!(
            "Processing dimension with primary_dim={}, secondary_dim={}",
            primary_dim, secondary_dim
        );
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
    pub fn collect_lines(
        img: &GrayImage,
        primary_dim: u32,
        secondary_dim: u32,
        is_empty_fn: impl Fn(&GrayImage, u32, u32) -> bool,
    ) -> Result<Vec<LineInfo>, GridError> {
        trace!("Collecting lines");
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
        trace!("Calculating average line size");
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
        trace!("Merging small lines with threshold={}", threshold);
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
        trace!("Checking if row y={} is empty", y);
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
        trace!("Checking if column x={} is empty", x);
        (0..height).all(|y| img.get_pixel(x, y).channels()[0] == 255)
    }
}

impl GridLike for Grid {
    /// Returns an iterator over all rows in the grid.
    fn rows_iter(&self) -> impl Iterator<Item = &Row> {
        self.rows.iter()
    }

    /// Returns an iterator over all columns in the grid.
    fn columns_iter(&self) -> impl Iterator<Item = &Column> {
        self.columns.iter()
    }

    type Row = Row;

    type Column = Column;
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
        Grid::try_from_image_with_config(image, GridConfig::default())
    }
}

/// Creates a `Row` or `Column` instance from a tuple of values.
///
/// This macro simplifies the creation of `Row` or `Column` instances by providing
/// a concise syntax. It supports optional specification of the `LineKind` (defaulting
/// to `LineKind::Empty` if not provided).
///
/// # Syntax
///
/// The macro has two forms for each type (`Row` and `Column`):
///
/// 1. **With `LineKind` specified:**
///    ```rust
///    use grider::*;
///    let _ = make_line!(Row, (0, 0, LineKind::Empty));
///    let _ = make_line!(Column, (0, 0, LineKind::Full));
///    ```
///    - `y` / `x`: The starting position of the row/column.
///    - `height` / `width`: The length of the row/column.
///    - `kind`: The `LineKind` (`LineKind::Empty` or `LineKind::Full`).
///
/// 2. **Without `LineKind` (defaults to `LineKind::Empty`):**
///    ```rust
///    use grider::*;
///    let _ = make_line!(Row, (0, 0));
///    let _ = make_line!(Column, (0, 0));
///    ```
///    - `y` / `x`: The starting position of the row/column.
///    - `height` / `width`: The length of the row/column.
///
/// # Examples
///
/// ## Creating a `Row`
///
/// ```rust
/// use grider::*;
///
/// // With LineKind::Full
/// let row = make_line!(Row, (0, 10, LineKind::Full));
/// assert_eq!(row.y, 0);
/// assert_eq!(row.height, 10);
/// assert_eq!(row.kind, LineKind::Full);
///
/// // Without LineKind (defaults to LineKind::Empty)
/// let row = make_line!(Row, (10, 20));
/// assert_eq!(row.y, 10);
/// assert_eq!(row.height, 20);
/// assert_eq!(row.kind, LineKind::Empty);
/// ```
///
/// ## Creating a `Column`
///
/// ```rust
/// use grider::*;
///
/// // With LineKind::Full
/// let column = make_line!(Column, (5, 15, LineKind::Full));
/// assert_eq!(column.x, 5);
/// assert_eq!(column.width, 15);
/// assert_eq!(column.kind, LineKind::Full);
///
/// // Without LineKind (defaults to LineKind::Empty)
/// let column = make_line!(Column, (20, 30));
/// assert_eq!(column.x, 20);
/// assert_eq!(column.width, 30);
/// assert_eq!(column.kind, LineKind::Empty);
/// ```
///
/// # Notes
///
/// - The macro internally uses the `LineInfo` struct to create `Row` or `Column` instances.
/// - If `LineKind` is not provided, it defaults to `LineKind::Empty`.
///
/// # See Also
///
/// - [`LineInfo`](struct.LineInfo.html): The underlying struct used to create rows and columns.
/// - [`Row`](struct.Row.html): Represents a row in the grid.
/// - [`Column`](struct.Column.html): Represents a column in the grid.
#[macro_export]
macro_rules! make_line {
    // For Rows
    (Row, ($y:expr, $height:expr, $kind:expr)) => {
        grid::Row::new(grid::LineInfo::new($y, $height, $kind))
    };
    (Row, ($y:expr, $height:expr)) => {
        grid::Row::new(grid::LineInfo::new($y, $height, grid::LineKind::Empty)) // Default to Empty
    };

    // For Columns
    (Column, ($x:expr, $width:expr, $kind:expr)) => {
        grid::Column::new(grid::LineInfo::new($x, $width, $kind))
    };
    (Column, ($x:expr, $width:expr)) => {
        grid::Column::new(grid::LineInfo::new($x, $width, grid::LineKind::Empty)) // Default to Empty
    };
}

/// Creates a `Grid` instance from a list of rows and columns.
///
/// This macro simplifies the creation of a `Grid` by allowing you to specify rows and columns
/// as lists of tuples. Each tuple represents a `Row` or `Column`, and the macro internally
/// uses the `make_line!` macro to construct the individual rows and columns.
///
/// # Syntax
///
/// The macro takes two lists: one for rows and one for columns. Each list contains tuples
/// that define the properties of the rows or columns.
///
/// ```rust
/// use grider::*;
/// let x = 0;
/// let y = 0;
/// let height = 0;
/// let width = 0;
/// let grid = make_grid!(
///     rows: [
///         (y, height),                // Row with default LineKind::Empty
///         (y, height, LineKind::Full), // Row with explicit LineKind
///     ],
///     columns: [
///         (x, width),                 // Column with default LineKind::Empty
///         (x, width, LineKind::Full), // Column with explicit LineKind
///     ]
/// );
/// ```
///
/// - `y`: The starting y-coordinate of the row.
/// - `height`: The height of the row.
/// - `x`: The starting x-coordinate of the column.
/// - `width`: The width of the column.
/// - `LineKind`: Optional. Specifies the kind of line (`LineKind::Empty` or `LineKind::Full`).
///   If not provided, it defaults to `LineKind::Empty`.
///
/// # Examples
///
/// ## Creating a Grid
///
/// ```rust
/// use grider::{make_grid, LineKind};
///
/// let grid = make_grid!(
///     rows: [
///         (0, 10),                     // Row at y=0, height=10, LineKind::Empty
///         (10, 20, LineKind::Full),    // Row at y=10, height=20, LineKind::Full
///     ],
///     columns: [
///         (0, 5),                      // Column at x=0, width=5, LineKind::Empty
///         (5, 15, LineKind::Full),     // Column at x=5, width=15, LineKind::Full
///     ]
/// );
///
/// assert_eq!(grid.rows.len(), 2);
/// assert_eq!(grid.columns.len(), 2);
/// ```
///
/// ## Notes
///
/// - The macro internally uses the `make_line!` macro to create individual rows and columns.
/// - If `LineKind` is not provided for a row or column, it defaults to `LineKind::Empty`.
/// - The resulting `Grid` contains `SmallVecLine` collections for rows and columns.
///
/// # See Also
///
/// - [`make_line!`]: The macro used internally to create rows and columns.
/// - [`Grid`]: The struct representing the grid of rows and columns.
/// - [`Row`]: Represents a row in the grid.
/// - [`Column`]: Represents a column in the grid.
#[macro_export]
macro_rules! make_grid {
    // Match rows and columns as separate lists with tuple syntax
    (rows: [$($row:tt,)*], columns: [$($col:tt,)*]) => {{

        use grider::*;
        use grider::make_line;
        Grid {
            rows: SmallVecLine::from_vec(vec![
                $(make_line!(Row, $row)),*
                ]),
                columns: SmallVecLine::from_vec(vec![
                    $(make_line!(Column, $col)),*
                    ]),
                }
            }
    };
}
