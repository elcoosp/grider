//! Drawing grids, cells, and grid lines on images.
//!
//! The main components of this module are:
//! - [`GridDrawingConfig`](crate::drawing::GridDrawingConfig): configuration
//!   for customizing the appearance of grids, cells, and grid lines.
//! - [`Drawable`](crate::drawing::Drawable): a trait implemented by types that
//!   can be drawn on an image, such as [`Cell`], [`Row`], [`Column`] and
//!   [`Grid`].
//!
//! # Examples
//!
//! ```
//! use grider::drawing::GridDrawingConfig;
//! use image::Rgba;
//!
//! let config = GridDrawingConfig {
//!     padding: 0,
//!     row_color: Rgba([255, 0, 0, 255]),
//!     column_color: Rgba([0, 0, 255, 255]),
//!     cell_background_color: Rgba([200, 200, 200, 255]),
//!     row_color_provider: None,
//!     column_color_provider: None,
//!     line_thickness: 1,
//! };
//! ```

use image::{Rgba, RgbaImage};
use imageproc::drawing::draw_filled_rect_mut;
use imageproc::rect::Rect;

use crate::grid::Grid;
use crate::lines::{Cell, Column, Row, clamp_to_i32};

/// Configuration for drawing grids, cells, and grid lines.
///
/// Users can specify uniform colors for rows and columns or provide custom
/// color provider functions for more advanced configurations.
///
/// # Examples
///
/// ```
/// use grider::drawing::GridDrawingConfig;
/// use image::Rgba;
///
/// let config = GridDrawingConfig::builder()
///     .padding(2)
///     .row_color(Rgba([255, 0, 0, 255]))
///     .line_thickness(3)
///     .build();
/// assert_eq!(config.padding, 2);
/// assert_eq!(config.line_thickness, 3);
/// ```
pub struct GridDrawingConfig {
    /// Padding between cells and grid lines.
    pub padding: u32,
    /// Default color for horizontal grid lines (rows).
    pub row_color: Rgba<u8>,
    /// Default color for vertical grid lines (columns).
    pub column_color: Rgba<u8>,
    /// Background color for cells.
    pub cell_background_color: Rgba<u8>,
    /// Optional function to provide custom colors for rows based on their index.
    pub row_color_provider: Option<Box<dyn Fn(usize) -> Rgba<u8>>>,
    /// Optional function to provide custom colors for columns based on their index.
    pub column_color_provider: Option<Box<dyn Fn(usize) -> Rgba<u8>>>,
    /// Thickness of grid lines, in pixels.
    pub line_thickness: u32,
}

/// Manual `Debug` implementation: color providers are rendered as `<function>`.
impl std::fmt::Debug for GridDrawingConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("GridDrawingConfig")
            .field("padding", &self.padding)
            .field("row_color", &self.row_color)
            .field("column_color", &self.column_color)
            .field("cell_background_color", &self.cell_background_color)
            .field("row_color_provider", &"<function>")
            .field("column_color_provider", &"<function>")
            .field("line_thickness", &self.line_thickness)
            .finish()
    }
}

impl Default for GridDrawingConfig {
    fn default() -> Self {
        Self {
            padding: 0,
            row_color: Rgba([255, 0, 0, 255]),    // Red
            column_color: Rgba([0, 0, 255, 255]), // Blue
            cell_background_color: Rgba([200, 200, 200, 255]), // Light gray
            row_color_provider: None,
            column_color_provider: None,
            line_thickness: 2,
        }
    }
}

impl GridDrawingConfig {
    /// Starts a [`GridDrawingConfigBuilder`] pre-populated with default values.
    ///
    /// # Examples
    ///
    /// ```
    /// use grider::drawing::GridDrawingConfig;
    ///
    /// let config = GridDrawingConfig::builder().build();
    /// assert_eq!(config.line_thickness, GridDrawingConfig::default().line_thickness);
    /// ```
    pub fn builder() -> GridDrawingConfigBuilder {
        GridDrawingConfigBuilder::new()
    }

    /// Resolves the color of a row, preferring the custom provider when set.
    #[must_use]
    pub fn row_color_at(&self, index: usize) -> Rgba<u8> {
        match &self.row_color_provider {
            Some(provider) => provider(index),
            None => self.row_color,
        }
    }

    /// Resolves the color of a column, preferring the custom provider when set.
    #[must_use]
    pub fn column_color_at(&self, index: usize) -> Rgba<u8> {
        match &self.column_color_provider {
            Some(provider) => provider(index),
            None => self.column_color,
        }
    }
}

/// Builder for [`GridDrawingConfig`].
///
/// # Examples
///
/// ```
/// use grider::drawing::GridDrawingConfig;
/// use image::Rgba;
///
/// let config = GridDrawingConfig::builder()
///     .padding(1)
///     .cell_background_color(Rgba([0, 0, 0, 255]))
///     .row_color_provider(Box::new(|index| Rgba([u8::try_from(index).unwrap_or(u8::MAX), 0, 0, 255])))
///     .build();
/// assert_eq!(config.padding, 1);
/// ```
#[must_use]
pub struct GridDrawingConfigBuilder {
    config: GridDrawingConfig,
}

impl GridDrawingConfigBuilder {
    /// Starts a builder pre-populated with [`GridDrawingConfig::default`] values.
    pub fn new() -> Self {
        Self {
            config: GridDrawingConfig::default(),
        }
    }

    /// Sets the padding between cells and grid lines.
    pub fn padding(mut self, padding: u32) -> Self {
        self.config.padding = padding;
        self
    }

    /// Sets the default color of horizontal grid lines (rows).
    pub fn row_color(mut self, row_color: Rgba<u8>) -> Self {
        self.config.row_color = row_color;
        self
    }

    /// Sets the default color of vertical grid lines (columns).
    pub fn column_color(mut self, column_color: Rgba<u8>) -> Self {
        self.config.column_color = column_color;
        self
    }

    /// Sets the background color of cells.
    pub fn cell_background_color(mut self, cell_background_color: Rgba<u8>) -> Self {
        self.config.cell_background_color = cell_background_color;
        self
    }

    /// Sets a provider for per-row colors (overrides `row_color`).
    pub fn row_color_provider(mut self, provider: Box<dyn Fn(usize) -> Rgba<u8>>) -> Self {
        self.config.row_color_provider = Some(provider);
        self
    }

    /// Sets a provider for per-column colors (overrides `column_color`).
    pub fn column_color_provider(mut self, provider: Box<dyn Fn(usize) -> Rgba<u8>>) -> Self {
        self.config.column_color_provider = Some(provider);
        self
    }

    /// Sets the thickness of grid lines.
    pub fn line_thickness(mut self, line_thickness: u32) -> Self {
        self.config.line_thickness = line_thickness.max(1);
        self
    }

    /// Builds the [`GridDrawingConfig`].
    #[must_use]
    pub fn build(self) -> GridDrawingConfig {
        self.config
    }
}

impl Default for GridDrawingConfigBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Trait for types that can be drawn on an image.
///
/// Implemented by [`Cell`], [`Row`], [`Column`] and [`Grid`].
///
/// Drawing clips every band and rectangle to the image bounds, so it cannot
/// fail: the method is infallible by construction.
///
/// # Examples
///
/// ```
/// use grider::drawing::{Drawable, GridDrawingConfig};
/// use grider::{Column, LineInfo, LineKind, LineTrait, Row};
/// use image::RgbaImage;
///
/// let mut image = RgbaImage::new(100, 100);
/// let config = GridDrawingConfig::default();
/// let row = Row::new(LineInfo::new(0, 10, LineKind::Full));
/// row.draw(&mut image, &config);
/// ```
pub trait Drawable {
    /// Draws the object on the provided image using the given configuration.
    ///
    /// Drawing is infallible: out-of-bounds geometry is clipped to the image.
    fn draw(&self, image: &mut RgbaImage, config: &GridDrawingConfig);
}

/// Draws a horizontal band of `thickness` pixels centered on `y`, clipped to
/// the image bounds.
fn draw_horizontal_band(image: &mut RgbaImage, y: u32, thickness: u32, color: Rgba<u8>) {
    let half = thickness / 2;
    let top = y.saturating_sub(half);
    let height = thickness.min(image.height().saturating_sub(top));
    if height == 0 || image.width() == 0 {
        return;
    }
    let rect = Rect::at(0, clamp_to_i32(top)).of_size(image.width(), height);
    draw_filled_rect_mut(image, rect, color);
}

/// Draws a vertical band of `thickness` pixels centered on `x`, clipped to the
/// image bounds.
fn draw_vertical_band(image: &mut RgbaImage, x: u32, thickness: u32, color: Rgba<u8>) {
    let half = thickness / 2;
    let left = x.saturating_sub(half);
    let width = thickness.min(image.width().saturating_sub(left));
    if width == 0 || image.height() == 0 {
        return;
    }
    let rect = Rect::at(clamp_to_i32(left), 0).of_size(width, image.height());
    draw_filled_rect_mut(image, rect, color);
}

impl Drawable for Row {
    fn draw(&self, image: &mut RgbaImage, config: &GridDrawingConfig) {
        draw_horizontal_band(image, self.end(), config.line_thickness, config.row_color);
    }
}

impl Drawable for Column {
    fn draw(&self, image: &mut RgbaImage, config: &GridDrawingConfig) {
        draw_vertical_band(
            image,
            self.end(),
            config.line_thickness,
            config.column_color,
        );
    }
}

impl Drawable for Cell<'_> {
    fn draw(&self, image: &mut RgbaImage, config: &GridDrawingConfig) {
        let rect = Rect::from(self);
        let padding = config.padding;

        let adjusted_left = rect.left() + clamp_to_i32(padding);
        let adjusted_top = rect.top() + clamp_to_i32(padding);
        let adjusted_width = rect.width().saturating_sub(padding.saturating_mul(2));
        let adjusted_height = rect.height().saturating_sub(padding.saturating_mul(2));

        if adjusted_width > 0 && adjusted_height > 0 {
            let cell_rect =
                Rect::at(adjusted_left, adjusted_top).of_size(adjusted_width, adjusted_height);
            draw_filled_rect_mut(image, cell_rect, config.cell_background_color);
        }
    }
}

impl Drawable for Grid {
    fn draw(&self, image: &mut RgbaImage, config: &GridDrawingConfig) {
        // Draw cells with padding.
        for row in &self.rows {
            for column in &self.columns {
                Cell { row, column }.draw(image, config);
            }
        }

        // Draw horizontal grid lines.
        for (row_index, row) in self.rows.iter().enumerate() {
            draw_horizontal_band(
                image,
                row.end(),
                config.line_thickness,
                config.row_color_at(row_index),
            );
        }

        // Draw vertical grid lines.
        for (column_index, column) in self.columns.iter().enumerate() {
            draw_vertical_band(
                image,
                column.end(),
                config.line_thickness,
                config.column_color_at(column_index),
            );
        }
    }
}
