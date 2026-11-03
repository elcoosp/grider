//! The [`Grid`] type: rows and columns extracted from an image.

use image::{DynamicImage, GenericImageView, GrayImage};
use tracing::{debug, error, trace};

use crate::config::GridConfig;
use crate::detection::{
    SmallVecLine, calculate_average_line_size, collect_lines, is_column_empty, is_row_empty,
    lines_from_flags, merge_small_lines, process_dimension as process_dimension_pipeline,
    scan_line_flags,
};
use crate::error::GridError;
use crate::lines::{Cell, Column, LineInfo, LineKind, LineTrait, Row};

/// Represents the grid of rows and columns extracted from an image.
///
/// # Examples
///
/// ```
/// use grider::{Grid, GridConfig};
/// use image::open;
///
/// let img = open("tests/large.png").unwrap();
/// let config = GridConfig::default();
/// let grid = Grid::try_from_image_with_config(&img, config).unwrap();
/// ```
#[derive(Debug, PartialEq, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Grid {
    /// The rows of the grid, ordered by ascending `y`.
    pub rows: SmallVecLine<Row>,
    /// The columns of the grid, ordered by ascending `x`.
    pub columns: SmallVecLine<Column>,
}

/// Default tolerance for filtering rows or columns of approximately the same size.
const DEFAULT_TOLERANCE: f32 = 0.1;

impl Grid {
    /// Creates a new `Grid` from an image with custom configuration.
    ///
    /// The image is converted to 8-bit grayscale when needed (already-gray
    /// images are borrowed without copying), binarized with the configured
    /// [`crate::Binarization`] method, and scanned for content and background
    /// runs on both axes.
    ///
    /// # Examples
    ///
    /// ```
    /// use grider::{Grid, GridConfig};
    /// use image::open;
    ///
    /// let img = open("tests/large.png").unwrap();
    /// let config = GridConfig::default();
    /// let grid = Grid::try_from_image_with_config(&img, config).unwrap();
    /// ```
    ///
    /// # Errors
    ///
    /// Returns [`GridError::InvalidDimensions`] when the image is empty and
    /// [`GridError::InvalidConfig`] when the configuration is rejected by
    /// [`GridConfig::validate`].
    pub fn try_from_image_with_config(
        image: &DynamicImage,
        config: GridConfig,
    ) -> Result<Self, GridError> {
        trace!("Processing image with config: {config:?}");
        config.validate()?;
        let (width, height) = image.dimensions();
        if width == 0 || height == 0 {
            error!("Invalid image dimensions: width={width}, height={height}");
            return Err(GridError::InvalidDimensions { width, height });
        }

        if let Some(gray) = image.as_luma8() {
            Ok(Self::extract(gray, &config))
        } else {
            let gray = image.to_luma8();
            Ok(Self::extract(&gray, &config))
        }
    }

    /// Creates a new `Grid` directly from a grayscale image.
    ///
    /// This is the zero-conversion counterpart of
    /// [`Grid::try_from_image_with_config`] for callers that already hold a
    /// [`GrayImage`] (common in computer-vision pipelines): no dynamic-image
    /// wrapper and no grayscale copy are involved.
    ///
    /// # Examples
    ///
    /// ```
    /// use grider::{Grid, GridConfig};
    /// use image::{GrayImage, Luma};
    ///
    /// let gray = GrayImage::from_pixel(50, 50, Luma([255]));
    /// let grid = Grid::try_from_gray_with_config(&gray, GridConfig::default()).unwrap();
    /// assert!(grid.is_empty() || grid.row_count() > 0);
    /// ```
    ///
    /// # Errors
    ///
    /// Returns [`GridError::InvalidDimensions`] when the image is empty and
    /// [`GridError::InvalidConfig`] when the configuration is rejected.
    pub fn try_from_gray_with_config(
        image: &GrayImage,
        config: GridConfig,
    ) -> Result<Self, GridError> {
        trace!("Processing grayscale image with config: {config:?}");
        config.validate()?;
        let (width, height) = image.dimensions();
        if width == 0 || height == 0 {
            error!("Invalid image dimensions: width={width}, height={height}");
            return Err(GridError::InvalidDimensions { width, height });
        }
        Ok(Self::extract(image, &config))
    }

    /// Creates a new `Grid` from an already binarized image.
    ///
    /// Binarization is the most expensive step of the pipeline; callers that
    /// binarize themselves (Otsu, their own Sauvola variant, a scanner SDK…)
    /// skip it entirely. Background must be white (`255`), content black
    /// (`0`) — the same convention as [`crate::binarize`]. Only the merge
    /// and parallelism settings of `config` are used; the threshold
    /// parameters are ignored.
    ///
    /// # Examples
    ///
    /// ```
    /// use grider::{Grid, GridConfig};
    /// use image::{GrayImage, Luma};
    ///
    /// let mut binary = GrayImage::from_pixel(20, 20, Luma([255]));
    /// for x in 0..20 {
    ///     binary.put_pixel(x, 10, Luma([0]));
    /// }
    /// // Keep every run: the default merge ratio would fold the lone 1px
    /// // rule into its background neighbours.
    /// let config = GridConfig::builder().merge_threshold_ratio(0.0).build();
    /// let grid = Grid::try_from_binarized(&binary, config).unwrap();
    /// assert_eq!(grid.count_rows_by_kind(grider::LineKind::Full), 1);
    /// ```
    ///
    /// # Errors
    ///
    /// Returns [`GridError::InvalidDimensions`] when the image is empty and
    /// [`GridError::InvalidConfig`] when the configuration is rejected.
    pub fn try_from_binarized(image: &GrayImage, config: GridConfig) -> Result<Self, GridError> {
        trace!("Processing binarized image with config: {config:?}");
        config.validate()?;
        let (width, height) = image.dimensions();
        if width == 0 || height == 0 {
            error!("Invalid image dimensions: width={width}, height={height}");
            return Err(GridError::InvalidDimensions { width, height });
        }
        let flags = scan_line_flags(image, config.enable_parallel);
        debug!(
            "Binarized scan produced {} row flags and {} column flags",
            flags.rows.len(),
            flags.columns.len()
        );
        Ok(Self {
            rows: lines_from_flags(&flags.rows, config.merge_threshold_ratio),
            columns: lines_from_flags(&flags.columns, config.merge_threshold_ratio),
        })
    }

    /// Runs the full binarize-then-scan pipeline on a validated, non-empty
    /// grayscale image.
    fn extract(gray: &GrayImage, config: &GridConfig) -> Self {
        let binarized = crate::binarize::binarize(gray, config);
        let flags = scan_line_flags(&binarized, config.enable_parallel);
        debug!(
            "Scan produced {} row flags and {} column flags",
            flags.rows.len(),
            flags.columns.len()
        );
        Self {
            rows: lines_from_flags(&flags.rows, config.merge_threshold_ratio),
            columns: lines_from_flags(&flags.columns, config.merge_threshold_ratio),
        }
    }

    /// Returns an iterator over all rows in the grid.
    ///
    /// # Examples
    ///
    /// ```
    /// use grider::{Grid, GridConfig};
    /// use image::open;
    ///
    /// let img = open("tests/large.png").unwrap();
    /// let grid = Grid::try_from_image_with_config(&img, GridConfig::default()).unwrap();
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
    /// # Examples
    ///
    /// ```
    /// use grider::{Grid, GridConfig};
    /// use image::open;
    ///
    /// let img = open("tests/large.png").unwrap();
    /// let grid = Grid::try_from_image_with_config(&img, GridConfig::default()).unwrap();
    ///
    /// for column in grid.columns() {
    ///     println!("Column at x: {}", column.x);
    /// }
    /// ```
    pub fn columns(&self) -> impl Iterator<Item = &Column> {
        self.columns.iter()
    }

    /// Returns the rows as a slice.
    #[must_use]
    pub fn rows_slice(&self) -> &[Row] {
        &self.rows
    }

    /// Returns the columns as a slice.
    #[must_use]
    pub fn columns_slice(&self) -> &[Column] {
        &self.columns
    }

    /// Returns the number of rows in the grid.
    #[must_use]
    pub fn row_count(&self) -> usize {
        self.rows.len()
    }

    /// Returns the number of columns in the grid.
    #[must_use]
    pub fn column_count(&self) -> usize {
        self.columns.len()
    }

    /// Returns `true` when the grid has neither rows nor columns.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.rows.is_empty() && self.columns.is_empty()
    }

    /// Returns an iterator over rows matching `kind`.
    ///
    /// # Examples
    ///
    /// ```
    /// use grider::{Grid, GridConfig, LineKind};
    /// use image::open;
    ///
    /// let img = open("tests/large.png").unwrap();
    /// let grid = Grid::try_from_image_with_config(&img, GridConfig::default()).unwrap();
    ///
    /// let full_rows: Vec<_> = grid.rows_by_kind(LineKind::Full).collect();
    /// ```
    pub fn rows_by_kind(&self, kind: LineKind) -> impl Iterator<Item = &Row> {
        self.rows.iter().filter(move |row| row.kind == kind)
    }

    /// Returns an iterator over columns matching `kind`.
    ///
    /// # Examples
    ///
    /// ```
    /// use grider::{Grid, GridConfig, LineKind};
    /// use image::open;
    ///
    /// let img = open("tests/large.png").unwrap();
    /// let grid = Grid::try_from_image_with_config(&img, GridConfig::default()).unwrap();
    ///
    /// let empty_columns: Vec<_> = grid.columns_by_kind(LineKind::Empty).collect();
    /// ```
    pub fn columns_by_kind(&self, kind: LineKind) -> impl Iterator<Item = &Column> {
        self.columns
            .iter()
            .filter(move |column| column.kind == kind)
    }

    /// Returns an iterator over filtered rows based on the predicate.
    ///
    /// # Examples
    ///
    /// ```
    /// use grider::{Column, Grid, GridConfig, LineKind, Row};
    /// use image::open;
    ///
    /// let img = open("tests/large.png").unwrap();
    /// let grid = Grid::try_from_image_with_config(&img, GridConfig::default()).unwrap();
    ///
    /// let filtered_rows: Vec<&Row> =
    ///     grid.filtered_rows(|row| row.kind == LineKind::Full).collect();
    /// ```
    pub fn filtered_rows(&self, predicate: impl Fn(&&Row) -> bool) -> impl Iterator<Item = &Row> {
        self.rows.iter().filter(predicate)
    }

    /// Returns an iterator over filtered columns based on the predicate.
    ///
    /// # Examples
    ///
    /// ```
    /// use grider::{Column, Grid, GridConfig, LineKind, Row};
    /// use image::open;
    ///
    /// let img = open("tests/large.png").unwrap();
    /// let grid = Grid::try_from_image_with_config(&img, GridConfig::default()).unwrap();
    ///
    /// let filtered_columns: Vec<&Column> =
    ///     grid.filtered_columns(|col| col.kind == LineKind::Full).collect();
    /// ```
    pub fn filtered_columns(
        &self,
        predicate: impl Fn(&&Column) -> bool,
    ) -> impl Iterator<Item = &Column> {
        self.columns.iter().filter(predicate)
    }

    /// Counts the number of rows with the specified kind.
    ///
    /// # Examples
    ///
    /// ```
    /// use grider::{Grid, GridConfig, LineKind};
    /// use image::open;
    ///
    /// let img = open("tests/large.png").unwrap();
    /// let grid = Grid::try_from_image_with_config(&img, GridConfig::default()).unwrap();
    ///
    /// let full_row_count = grid.count_rows_by_kind(LineKind::Full);
    /// ```
    #[must_use]
    pub fn count_rows_by_kind(&self, kind: LineKind) -> usize {
        self.rows.iter().filter(|row| row.kind == kind).count()
    }

    /// Counts the number of columns with the specified kind.
    ///
    /// # Examples
    ///
    /// ```
    /// use grider::{Grid, GridConfig, LineKind};
    /// use image::open;
    ///
    /// let img = open("tests/large.png").unwrap();
    /// let grid = Grid::try_from_image_with_config(&img, GridConfig::default()).unwrap();
    ///
    /// let full_column_count = grid.count_columns_by_kind(LineKind::Full);
    /// ```
    #[must_use]
    pub fn count_columns_by_kind(&self, kind: LineKind) -> usize {
        self.columns.iter().filter(|col| col.kind == kind).count()
    }

    /// Returns the cell at the given row and column *indices*, or `None`.
    ///
    /// This is the index-based counterpart of [`Grid::find_cells`], which works
    /// with coordinates instead.
    ///
    /// # Examples
    ///
    /// ```
    /// use grider::{make_grid, Grid};
    ///
    /// let grid = make_grid!(rows: [(0, 5)], columns: [(0, 5)]);
    /// let cell = grid.cell(0, 0).expect("cell exists");
    /// assert_eq!(cell.rect(), imageproc::rect::Rect::at(0, 0).of_size(5, 5));
    /// assert!(grid.cell(1, 0).is_none());
    /// ```
    #[must_use]
    pub fn cell(&self, row_index: usize, column_index: usize) -> Option<Cell<'_>> {
        Some(Cell {
            row: self.rows.get(row_index)?,
            column: self.columns.get(column_index)?,
        })
    }

    /// Returns an iterator over every cell of the grid (row-major order).
    ///
    /// # Examples
    ///
    /// ```
    /// use grider::{make_grid, Grid};
    ///
    /// let grid = make_grid!(rows: [(0, 5), (5, 5)], columns: [(0, 5), (5, 5)]);
    /// assert_eq!(grid.iter_cells().count(), 4);
    /// ```
    pub fn iter_cells(&self) -> impl Iterator<Item = Cell<'_>> {
        self.rows
            .iter()
            .flat_map(|row| self.columns.iter().map(move |column| Cell { row, column }))
    }

    /// Finds cells based on row and column *coordinates* (`y`/`x` positions).
    ///
    /// Each yielded item is `Ok` when both a row starting at the given `y` and
    /// a column starting at the given `x` exist, `Err` otherwise. Use
    /// [`Grid::cell`] or [`Grid::iter_cells`] for index-based access.
    ///
    /// # Examples
    ///
    /// ```
    /// use grider::{Grid, GridConfig};
    /// use image::open;
    ///
    /// let img = open("tests/large.png").unwrap();
    /// let grid = Grid::try_from_image_with_config(&img, GridConfig::default()).unwrap();
    ///
    /// let cells = grid.find_cells(&[1, 2, 3], &[4, 5, 6]);
    /// for cell in cells {
    ///     match cell {
    ///         Ok(c) => println!("Found cell at row {}, column {}", c.row.y, c.column.x),
    ///         Err(e) => eprintln!("Error finding cell: {e}"),
    ///     }
    /// }
    /// ```
    pub fn find_cells<'a>(
        &'a self,
        row_positions: &'a [u32],
        column_positions: &'a [u32],
    ) -> impl Iterator<Item = Result<Cell<'a>, GridError>> + 'a {
        row_positions.iter().flat_map(move |&row_position| {
            column_positions.iter().map(move |&column_position| {
                let row = self
                    .find_row(row_position)
                    .ok_or(GridError::RowNotFound { y: row_position })?;
                let column = self
                    .find_column(column_position)
                    .ok_or(GridError::ColumnNotFound { x: column_position })?;
                Ok(Cell { row, column })
            })
        })
    }

    /// Finds a row by its `y` coordinate.
    ///
    /// # Examples
    ///
    /// ```
    /// use grider::{Grid, GridConfig};
    /// use image::open;
    ///
    /// let img = open("tests/large.png").unwrap();
    /// let grid = Grid::try_from_image_with_config(&img, GridConfig::default()).unwrap();
    ///
    /// let row = grid.find_row(0);
    /// assert!(row.is_some());
    /// ```
    #[must_use]
    pub fn find_row(&self, y: u32) -> Option<&Row> {
        self.rows.iter().find(|row| row.y == y)
    }

    /// Finds a column by its `x` coordinate.
    ///
    /// # Examples
    ///
    /// ```
    /// use grider::{Grid, GridConfig};
    /// use image::open;
    ///
    /// let img = open("tests/large.png").unwrap();
    /// let grid = Grid::try_from_image_with_config(&img, GridConfig::default()).unwrap();
    ///
    /// let column = grid.find_column(0);
    /// assert!(column.is_some());
    /// ```
    #[must_use]
    pub fn find_column(&self, x: u32) -> Option<&Column> {
        self.columns.iter().find(|col| col.x == x)
    }

    /// Returns the overall extent of the grid as `(width, height)`.
    ///
    /// The width is the largest `column.end()` and the height the largest
    /// `row.end()`; both are `0` when the corresponding axis is empty.
    ///
    /// # Examples
    ///
    /// ```
    /// use grider::{make_grid, Grid};
    ///
    /// let grid = make_grid!(rows: [(0, 5), (5, 10)], columns: [(0, 4)]);
    /// assert_eq!(grid.dimensions(), (4, 15));
    /// ```
    #[must_use]
    pub fn dimensions(&self) -> (u32, u32) {
        let width = self.columns.iter().map(Column::end).max().unwrap_or(0);
        let height = self.rows.iter().map(Row::end).max().unwrap_or(0);
        (width, height)
    }

    /// Keeps only the rows matching `predicate`, in place.
    ///
    /// This is the allocation-free counterpart of the `filter_*` methods.
    ///
    /// # Examples
    ///
    /// ```
    /// use grider::{make_grid, LineKind};
    ///
    /// let mut grid = make_grid!(rows: [(0, 5), (5, 5, LineKind::Full)], columns: [(0, 2)]);
    /// grid.retain_rows(|row| row.kind == LineKind::Full);
    /// assert_eq!(grid.rows.len(), 1);
    /// ```
    pub fn retain_rows<F>(&mut self, mut predicate: F)
    where
        F: FnMut(&Row) -> bool,
    {
        self.rows.retain(|row| predicate(row));
    }

    /// Keeps only the columns matching `predicate`, in place.
    ///
    /// # Examples
    ///
    /// ```
    /// use grider::{make_grid, LineKind};
    ///
    /// let mut grid = make_grid!(rows: [(0, 2)], columns: [(0, 5), (5, 5, LineKind::Full)]);
    /// grid.retain_columns(|column| column.kind == LineKind::Full);
    /// assert_eq!(grid.columns.len(), 1);
    /// ```
    pub fn retain_columns<F>(&mut self, mut predicate: F)
    where
        F: FnMut(&Column) -> bool,
    {
        self.columns.retain(|column| predicate(column));
    }

    /// Returns a copy of the grid keeping only lines of the given kind.
    ///
    /// # Examples
    ///
    /// ```
    /// use grider::{make_grid, LineKind};
    ///
    /// let grid = make_grid!(
    ///     rows: [(0, 5), (5, 5, LineKind::Full)],
    ///     columns: [(0, 5, LineKind::Full)]
    /// );
    /// let full_only = grid.keep_kind(LineKind::Full);
    /// assert_eq!(full_only.rows.len(), 1);
    /// assert_eq!(full_only.columns.len(), 1);
    /// ```
    #[must_use]
    pub fn keep_kind(&self, kind: LineKind) -> Self {
        Self {
            rows: self
                .rows
                .iter()
                .filter(|row| row.kind == kind)
                .copied()
                .collect(),
            columns: self
                .columns
                .iter()
                .filter(|column| column.kind == kind)
                .copied()
                .collect(),
        }
    }

    /// Helper to calculate min and max bounds based on size and tolerance.
    ///
    /// The computation runs in `f64`, which represents every `u32` size
    /// losslessly (an `f32` would lose precision for large images).
    fn calculate_bounds(size: u32, tolerance: f32) -> (f64, f64) {
        let size = f64::from(size);
        let tolerance = f64::from(tolerance);
        (size * (1.0 - tolerance), size * (1.0 + tolerance))
    }

    /// Shared helper filtering lines whose size is within `tolerance` of the
    /// smallest (or biggest) line of the collection.
    ///
    /// When the collection is empty, every line (i.e. none) is kept.
    fn filter_extremum<T, F>(
        lines: &[T],
        get_size: F,
        want_smallest: bool,
        tolerance: f32,
    ) -> SmallVecLine<T>
    where
        T: Clone,
        F: Fn(&T) -> u32,
    {
        let sizes = lines.iter().map(&get_size);
        let extremum = if want_smallest {
            sizes.min()
        } else {
            sizes.max()
        };
        let Some(extremum) = extremum else {
            return lines.iter().cloned().collect();
        };
        let (min_size, max_size) = Self::calculate_bounds(extremum, tolerance);
        lines
            .iter()
            .filter(|line| {
                let line_size = f64::from(get_size(line));
                line_size < min_size || line_size > max_size
            })
            .cloned()
            .collect()
    }

    /// Filters out rows of approximately the same size as the smallest row.
    ///
    /// # Arguments
    /// * `tolerance` - The tolerance for determining if rows are of approximately the same size.
    ///
    /// # Returns
    /// A new `Grid` with rows of approximately the same size as the smallest row removed.
    #[must_use]
    pub fn filter_smallest_rows_with_tolerance(&self, tolerance: f32) -> Self {
        Self {
            rows: Self::filter_extremum(&self.rows, |row| row.height, true, tolerance),
            columns: self.columns.clone(),
        }
    }

    /// Filters out rows of approximately the same size as the biggest row.
    ///
    /// # Arguments
    /// * `tolerance` - The tolerance for determining if rows are of approximately the same size.
    ///
    /// # Returns
    /// A new `Grid` with rows of approximately the same size as the biggest row removed.
    #[must_use]
    pub fn filter_biggest_rows_with_tolerance(&self, tolerance: f32) -> Self {
        Self {
            rows: Self::filter_extremum(&self.rows, |row| row.height, false, tolerance),
            columns: self.columns.clone(),
        }
    }

    /// Filters out columns of approximately the same size as the smallest column.
    ///
    /// # Arguments
    /// * `tolerance` - The tolerance for determining if columns are of approximately the same size.
    ///
    /// # Returns
    /// A new `Grid` with columns of approximately the same size as the smallest column removed.
    #[must_use]
    pub fn filter_smallest_columns_with_tolerance(&self, tolerance: f32) -> Self {
        Self {
            rows: self.rows.clone(),
            columns: Self::filter_extremum(&self.columns, |column| column.width, true, tolerance),
        }
    }

    /// Filters out columns of approximately the same size as the biggest column.
    ///
    /// # Arguments
    /// * `tolerance` - The tolerance for determining if columns are of approximately the same size.
    ///
    /// # Returns
    /// A new `Grid` with columns of approximately the same size as the biggest column removed.
    #[must_use]
    pub fn filter_biggest_columns_with_tolerance(&self, tolerance: f32) -> Self {
        Self {
            rows: self.rows.clone(),
            columns: Self::filter_extremum(&self.columns, |column| column.width, false, tolerance),
        }
    }

    /// Filters out the smallest rows in the grid.
    ///
    /// # Returns
    /// A new `Grid` with the smallest rows removed.
    #[must_use]
    pub fn filter_smallest_rows(&self) -> Self {
        self.filter_smallest_rows_with_tolerance(DEFAULT_TOLERANCE)
    }

    /// Filters out the biggest rows in the grid.
    ///
    /// # Returns
    /// A new `Grid` with the biggest rows removed.
    #[must_use]
    pub fn filter_biggest_rows(&self) -> Self {
        self.filter_biggest_rows_with_tolerance(DEFAULT_TOLERANCE)
    }

    /// Filters out the smallest columns in the grid.
    ///
    /// # Returns
    /// A new `Grid` with the smallest columns removed.
    #[must_use]
    pub fn filter_smallest_columns(&self) -> Self {
        self.filter_smallest_columns_with_tolerance(DEFAULT_TOLERANCE)
    }

    /// Filters out the biggest columns in the grid.
    ///
    /// # Returns
    /// A new `Grid` with the biggest columns removed.
    #[must_use]
    pub fn filter_biggest_columns(&self) -> Self {
        self.filter_biggest_columns_with_tolerance(DEFAULT_TOLERANCE)
    }

    /// Finds the smallest height among all rows in the grid.
    ///
    /// # Returns
    /// The smallest height of the rows, or `None` if there are no rows.
    #[must_use]
    pub fn smallest_row_height(&self) -> Option<u32> {
        self.rows.iter().map(|row| row.height).min()
    }

    /// Finds the biggest height among all rows in the grid.
    ///
    /// # Returns
    /// The biggest height of the rows, or `None` if there are no rows.
    #[must_use]
    pub fn biggest_row_height(&self) -> Option<u32> {
        self.rows.iter().map(|row| row.height).max()
    }

    /// Finds the smallest width among all columns in the grid.
    ///
    /// # Returns
    /// The smallest width of the columns, or `None` if there are no columns.
    #[must_use]
    pub fn smallest_column_width(&self) -> Option<u32> {
        self.columns.iter().map(|col| col.width).min()
    }

    /// Finds the biggest width among all columns in the grid.
    ///
    /// # Returns
    /// The biggest width of the columns, or `None` if there are no columns.
    #[must_use]
    pub fn biggest_column_width(&self) -> Option<u32> {
        self.columns.iter().map(|col| col.width).max()
    }

    /// Collects the initial (unmerged) [`LineInfo`] runs of one axis from an image.
    ///
    /// See [`crate::detection::collect_lines`] for details.
    #[must_use]
    pub fn collect_lines(
        img: &GrayImage,
        primary_dim: u32,
        secondary_dim: u32,
        is_empty_fn: impl Fn(&GrayImage, u32, u32) -> bool,
    ) -> Vec<LineInfo> {
        collect_lines(img, primary_dim, secondary_dim, is_empty_fn)
    }

    /// Processes one dimension (rows or columns) of a binarized image into grid
    /// lines, sequentially.
    ///
    /// See [`crate::detection::process_dimension`] for details.
    ///
    /// # Errors
    ///
    /// Returns [`GridError::InvalidDimensions`] when either dimension is zero.
    pub fn process_dimension<T: LineTrait + Send>(
        img: &GrayImage,
        primary_dim: u32,
        secondary_dim: u32,
        merge_threshold_ratio: f32,
        is_empty_fn: impl Fn(&GrayImage, u32, u32) -> bool + Sync,
    ) -> Result<SmallVecLine<T>, GridError> {
        process_dimension_pipeline(
            img,
            primary_dim,
            secondary_dim,
            merge_threshold_ratio,
            is_empty_fn,
            false,
        )
    }

    /// Merges lines smaller than the threshold.
    ///
    /// See [`crate::detection::merge_small_lines`] for details.
    #[must_use]
    pub fn merge_small_lines(lines: Vec<LineInfo>, threshold: u32) -> SmallVecLine<LineInfo> {
        merge_small_lines(lines, threshold)
    }

    /// Calculates the average size of a set of line runs.
    ///
    /// See [`crate::detection::calculate_average_line_size`] for details.
    #[must_use]
    pub fn calculate_average_line_size(lines: &[LineInfo]) -> u32 {
        calculate_average_line_size(lines)
    }

    /// Checks if a row is empty (all pixels are white).
    ///
    /// See [`crate::detection::is_row_empty`] for details.
    ///
    /// # Panics
    /// Panics if `y` is outside the image.
    #[must_use]
    pub fn is_row_empty(img: &GrayImage, y: u32, width: u32) -> bool {
        is_row_empty(img, y, width)
    }

    /// Checks if a column is empty (all pixels are white).
    ///
    /// See [`crate::detection::is_column_empty`] for details.
    ///
    /// # Panics
    /// Panics if `x` is outside the image.
    #[must_use]
    pub fn is_column_empty(img: &GrayImage, x: u32, height: u32) -> bool {
        is_column_empty(img, x, height)
    }
}

impl std::fmt::Display for Grid {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Grid {{ rows: {}, columns: {} }}",
            self.rows.len(),
            self.columns.len()
        )
    }
}

impl TryFrom<DynamicImage> for Grid {
    type Error = GridError;

    fn try_from(image: DynamicImage) -> Result<Self, Self::Error> {
        TryFrom::try_from(&image)
    }
}

impl TryFrom<&DynamicImage> for Grid {
    type Error = GridError;

    fn try_from(image: &DynamicImage) -> Result<Self, Self::Error> {
        Grid::try_from_image_with_config(image, GridConfig::default())
    }
}

#[cfg(test)]
mod tests {
    use image::{DynamicImage, GrayImage};

    use crate::config::GridConfig;
    use crate::detection::SmallVecLine;
    use crate::error::GridError;
    use crate::lines::{LineInfo, LineKind, Row};
    use crate::{Grid, make_grid};

    #[test]
    fn display_reports_counts() {
        let grid = make_grid!(rows: [(0, 5), (5, 5)], columns: [(0, 3)]);
        assert_eq!(grid.to_string(), "Grid { rows: 2, columns: 1 }");
    }

    #[test]
    fn counts_and_slices_agree() {
        let grid = make_grid!(
            rows: [(0, 5), (5, 5, LineKind::Full), (10, 2)],
            columns: [(0, 3, LineKind::Full)]
        );
        assert_eq!(grid.row_count(), 3);
        assert_eq!(grid.column_count(), 1);
        assert!(!grid.is_empty());
        assert_eq!(grid.rows_slice().len(), 3);
        assert_eq!(grid.columns_slice().len(), 1);
        assert_eq!(grid.rows_slice()[0], grid.rows[0]);
    }

    #[test]
    fn dimensions_reports_extent() {
        let grid = make_grid!(rows: [(0, 5), (5, 10)], columns: [(0, 4)]);
        assert_eq!(grid.dimensions(), (4, 15));
    }

    #[test]
    fn dimensions_of_empty_axes() {
        let grid: Grid = Grid {
            rows: SmallVecLine::new(),
            columns: SmallVecLine::new(),
        };
        assert_eq!(grid.dimensions(), (0, 0));
        assert!(grid.is_empty());
    }

    #[test]
    fn filter_extremum_on_empty_collection_keeps_nothing() {
        let grid: Grid = Grid {
            rows: SmallVecLine::new(),
            columns: SmallVecLine::new(),
        };
        assert!(grid.filter_smallest_rows().rows.is_empty());
        assert!(grid.filter_biggest_rows().rows.is_empty());
        assert!(grid.filter_smallest_columns().columns.is_empty());
        assert!(grid.filter_biggest_columns().columns.is_empty());
        assert_eq!(grid.smallest_row_height(), None);
        assert_eq!(grid.biggest_row_height(), None);
        assert_eq!(grid.smallest_column_width(), None);
        assert_eq!(grid.biggest_column_width(), None);
    }

    #[test]
    fn keep_kind_works_on_both_axes() {
        let grid = make_grid!(
            rows: [(0, 5), (5, 5, LineKind::Full)],
            columns: [(0, 5), (5, 5, LineKind::Full)]
        );
        let empty_only = grid.keep_kind(LineKind::Empty);
        assert_eq!(empty_only.rows.len(), 1);
        assert_eq!(empty_only.columns.len(), 1);
        assert!(empty_only.rows[0].kind.is_empty());
    }

    #[test]
    fn retain_kinds_in_place() {
        let mut grid = make_grid!(
            rows: [(0, 5), (5, 5, LineKind::Full)],
            columns: [(0, 5, LineKind::Full)]
        );
        grid.retain_rows(|row| row.kind.is_full());
        grid.retain_columns(|column| column.kind.is_full());
        assert_eq!(grid.rows.len(), 1);
        assert_eq!(grid.columns.len(), 1);
    }

    #[test]
    fn extremum_helpers_report_extremes() {
        let grid = make_grid!(
            rows: [(0, 2), (2, 8, LineKind::Full), (10, 4)],
            columns: [(0, 1), (1, 7, LineKind::Full), (8, 3)]
        );
        assert_eq!(grid.smallest_row_height(), Some(2));
        assert_eq!(grid.biggest_row_height(), Some(8));
        assert_eq!(grid.smallest_column_width(), Some(1));
        assert_eq!(grid.biggest_column_width(), Some(7));
    }

    #[test]
    fn average_line_size_wrapper_matches_detection() {
        let lines = vec![LineInfo::new(0, 3, LineKind::Empty)];
        assert_eq!(Grid::calculate_average_line_size(&lines), 3);
    }

    /// Builds a small synthetic image with clear row/column structure:
    /// horizontal white bands separated by black lines (and vice versa).
    fn structured_test_image(width: u32, height: u32) -> DynamicImage {
        let mut img = image::GrayImage::new(width, height);
        for y in 0..height {
            for x in 0..width {
                let value = if y % 8 == 0 || x % 8 == 0 { 0 } else { 255 };
                img.put_pixel(x, y, image::Luma([value]));
            }
        }
        DynamicImage::ImageLuma8(img)
    }

    #[test]
    fn try_from_image_extracts_a_synthetic_grid() {
        let img = structured_test_image(33, 33);
        let grid = Grid::try_from_image_with_config(&img, GridConfig::default())
            .expect("structured image yields a grid");
        assert!(!grid.rows.is_empty());
        assert!(!grid.columns.is_empty());

        // Query helpers agree on the extracted grid.
        let full_rows = grid.count_rows_by_kind(LineKind::Full);
        assert_eq!(full_rows, grid.rows_by_kind(LineKind::Full).count());
        assert_eq!(
            grid.count_columns_by_kind(LineKind::Empty),
            grid.columns_by_kind(LineKind::Empty).count()
        );
        let even_rows: Vec<_> = grid.filtered_rows(|row| row.y % 2 == 0).collect();
        assert_eq!(
            even_rows.len(),
            grid.rows.iter().filter(|r| r.y % 2 == 0).count()
        );
        assert_eq!(
            grid.filtered_columns(|column| column.x == 0).count(),
            usize::from(grid.columns[0].x == 0)
        );

        // Coordinate lookups resolve known positions.
        let y = grid.rows[0].y;
        assert_eq!(grid.find_row(y).map(|row| row.y), Some(y));
        let x = grid.columns[0].x;
        assert_eq!(grid.find_column(x).map(|column| column.x), Some(x));

        // Cells resolve through both the coordinate and index based APIs.
        let row = grid.rows[0];
        let column = grid.columns[0];
        let row_positions = [row.y];
        let column_positions = [column.x];
        let cells: Vec<_> = grid.find_cells(&row_positions, &column_positions).collect();
        assert_eq!(cells.len(), 1);
        assert!(cells[0].is_ok());
        assert!(grid.cell(0, 0).is_some());
        assert_eq!(
            grid.iter_cells().count(),
            grid.rows.len() * grid.columns.len()
        );
    }

    #[test]
    fn try_from_trait_impls_match_the_configured_path() {
        let img = structured_test_image(25, 25);
        let reference = Grid::try_from_image_with_config(&img, GridConfig::default()).unwrap();
        assert_eq!(Grid::try_from(&img).unwrap(), reference);
        assert_eq!(Grid::try_from(img.clone()).unwrap(), reference);
    }

    #[test]
    fn gray_entry_point_matches_the_dynamic_path() {
        let dynamic = structured_test_image(25, 25);
        let gray = dynamic.to_luma8();
        assert_eq!(
            Grid::try_from_gray_with_config(&gray, GridConfig::default()).unwrap(),
            Grid::try_from_image_with_config(&dynamic, GridConfig::default()).unwrap()
        );
    }

    #[test]
    fn gray_entry_point_rejects_degenerate_input() {
        let empty = GrayImage::new(0, 4);
        assert_eq!(
            Grid::try_from_gray_with_config(&empty, GridConfig::default()),
            Err(GridError::InvalidDimensions {
                width: 0,
                height: 4
            })
        );
        let config = GridConfig {
            merge_threshold_ratio: 2.0,
            ..GridConfig::default()
        };
        let img = GrayImage::from_pixel(4, 4, image::Luma([255]));
        assert!(Grid::try_from_gray_with_config(&img, config).is_err());
    }

    #[test]
    fn rgb_images_take_the_conversion_path() {
        // Non-gray DynamicImage variants have no borrowable luma buffer and
        // must go through to_luma8.
        let luma = structured_test_image(25, 25);
        let rgb = DynamicImage::ImageRgb8(luma.to_rgb8());
        let expected = Grid::try_from_image_with_config(&luma, GridConfig::default()).unwrap();
        assert_eq!(
            Grid::try_from_image_with_config(&rgb, GridConfig::default()).unwrap(),
            expected
        );
    }

    #[test]
    fn binarized_entry_point_matches_the_full_pipeline() {
        let gray = structured_test_image(25, 25).to_luma8();
        let config = GridConfig::default();
        let binarized = crate::binarize::binarize(&gray, &config);
        assert_eq!(
            Grid::try_from_binarized(&binarized, config).unwrap(),
            Grid::try_from_gray_with_config(&gray, config).unwrap()
        );
        // The merge ratio still applies on the binarized path.
        let no_merge = GridConfig {
            merge_threshold_ratio: 0.0,
            ..GridConfig::default()
        };
        let unmerged = Grid::try_from_binarized(&binarized, no_merge).unwrap();
        assert!(
            unmerged.row_count()
                >= Grid::try_from_binarized(&binarized, config)
                    .unwrap()
                    .row_count()
        );
    }

    #[test]
    fn binarized_entry_point_rejects_degenerate_input() {
        let empty = GrayImage::new(4, 0);
        assert_eq!(
            Grid::try_from_binarized(&empty, GridConfig::default()),
            Err(GridError::InvalidDimensions {
                width: 4,
                height: 0
            })
        );
    }

    #[test]
    fn sauvola_end_to_end_extracts_a_grid() {
        // A synthetic table: four horizontal rules plus vertical rules that
        // only span the middle band, so the top and bottom rows are pure
        // background.
        let mut gray = GrayImage::from_pixel(40, 40, image::Luma([255]));
        for y in [8, 16, 24, 32] {
            for x in 0..40 {
                gray.put_pixel(x, y, image::Luma([0]));
            }
        }
        for x in [8, 16, 24, 32] {
            for y in 12..29 {
                gray.put_pixel(x, y, image::Luma([0]));
            }
        }
        let config = GridConfig::builder()
            .sauvola(0.2)
            .merge_threshold_ratio(0.0)
            .build();
        let grid = Grid::try_from_gray_with_config(&gray, config).unwrap();
        // Runs, not pixel rows: the rules at y=8 and y=32 stay 1px runs while
        // the vertical-rule band y=12..28 forms one 17px Full run.
        let full_rows: Vec<_> = grid.rows_by_kind(LineKind::Full).collect();
        assert_eq!(
            full_rows.len(),
            3,
            "sauvola must see the rule bands: {grid:?}"
        );
        assert_eq!(full_rows[0].y, 8);
        assert_eq!(full_rows[1].y, 12);
        assert_eq!(full_rows[2].y, 32);
        assert_eq!(
            grid.count_rows_by_kind(LineKind::Empty),
            4,
            "sauvola must keep the background bands: {grid:?}"
        );

        // The Sauvola pipeline equals manual binarization + scan.
        let manual = crate::binarize::binarize(&gray, &config);
        assert_eq!(Grid::try_from_binarized(&manual, config).unwrap(), grid);
    }

    #[test]
    fn degenerate_dimensions_log_through_a_subscriber() {
        // The `error!` branches run their lazy formatting only when a
        // dispatcher is installed; exercise them under a scoped subscriber.
        let subscriber = tracing_subscriber::fmt()
            .with_max_level(tracing::Level::TRACE)
            .finish();
        tracing::subscriber::with_default(subscriber, || {
            let flat = GrayImage::new(3, 0);
            assert_eq!(
                Grid::try_from_binarized(&flat, GridConfig::default()),
                Err(GridError::InvalidDimensions {
                    width: 3,
                    height: 0
                })
            );
            let thin = GrayImage::new(0, 3);
            assert_eq!(
                Grid::try_from_gray_with_config(&thin, GridConfig::default()),
                Err(GridError::InvalidDimensions {
                    width: 0,
                    height: 3
                })
            );
            // Successful runs reach the post-guard `debug!` formatting, which
            // only executes under a dispatcher.
            let img = GrayImage::from_pixel(2, 2, image::Luma([255]));
            let _ = Grid::try_from_binarized(&img, GridConfig::default()).unwrap();
            let _ = Grid::try_from_gray_with_config(&img, GridConfig::default()).unwrap();
        });
    }

    #[test]
    fn try_from_rejects_zero_dimensions() {
        let img = DynamicImage::ImageLuma8(image::GrayImage::new(0, 0));
        assert_eq!(
            Grid::try_from_image_with_config(&img, GridConfig::default()),
            Err(GridError::InvalidDimensions {
                width: 0,
                height: 0
            })
        );
    }

    #[test]
    fn try_from_rejects_an_invalid_config() {
        let img = structured_test_image(25, 25);
        let config = GridConfig {
            threshold_block_size: 0,
            ..GridConfig::default()
        };
        assert_eq!(
            Grid::try_from_image_with_config(&img, config),
            Err(GridError::invalid_config(
                "threshold_block_size must be greater than 0"
            ))
        );
    }

    #[test]
    fn find_cells_reports_missing_positions() {
        let grid = make_grid!(rows: [(0, 5)], columns: [(0, 5)]);
        let missing_row_positions = [99_u32];
        let missing_column_positions = [99_u32];
        let first_positions = [0_u32];
        let missing_row = grid
            .find_cells(&missing_row_positions, &first_positions)
            .next()
            .unwrap();
        assert_eq!(missing_row, Err(GridError::RowNotFound { y: 99 }));
        let missing_column = grid
            .find_cells(&first_positions, &missing_column_positions)
            .next()
            .unwrap();
        assert_eq!(missing_column, Err(GridError::ColumnNotFound { x: 99 }));
    }

    #[test]
    fn detection_wrappers_match_their_documented_contracts() {
        // 3x4 image: row 0 black (not empty), rows 1-3 white (empty).
        let mut img = image::GrayImage::from_pixel(3, 4, image::Luma([255]));
        for x in 0..3 {
            img.put_pixel(x, 0, image::Luma([0]));
        }
        assert!(!Grid::is_row_empty(&img, 0, 3));
        assert!(Grid::is_row_empty(&img, 1, 3));
        // Every column crosses the black row 0, so none is empty.
        assert!(!Grid::is_column_empty(&img, 0, 4));
        assert!(!Grid::is_column_empty(&img, 2, 4));

        let lines = Grid::collect_lines(&img, 4, 3, Grid::is_row_empty);
        // One non-empty run (row 0) followed by one empty run (rows 1-3).
        assert_eq!(lines.len(), 2);
        assert_eq!(lines[0].start, 0);
        assert_eq!(lines[0].length, 1);
        assert_eq!(lines[1].start, 1);
        assert_eq!(lines[1].length, 3);

        let processed = Grid::process_dimension::<Row>(
            &img,
            4,
            3,
            GridConfig::default().merge_threshold_ratio,
            Grid::is_row_empty,
        )
        .expect("processing a valid image succeeds");
        // The 1px non-empty run is below the merge threshold relative to the
        // 3px empty run, so both are merged into a single output line.
        assert_eq!(processed.len(), 1);

        assert_eq!(
            Grid::process_dimension::<Row>(&img, 0, 3, 0.8, |_, _, _| false),
            Err(GridError::InvalidDimensions {
                width: 3,
                height: 0
            })
        );

        let merged = Grid::merge_small_lines(lines, 2);
        assert_eq!(merged.len(), 1);
    }
}
