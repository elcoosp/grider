//! This module provides functionality for drawing grids, cells, and grid lines on images.
//! It is feature-gated under the `drawing` feature and requires the `image` and `imageproc` crates.
//!
//! # Examples
//!
//! ```rust
//! use grider::{Grid, GridConfig, drawing::*};
//! use image::open;
//!
//! // Load an image
//! let img = open("tests/large.png").unwrap();
//!
//! // Create a grid from the image
//! let config = GridConfig::default();
//! let grid = Grid::try_from_image_with_config(&img, config).unwrap();
//!
//! // Configure drawing settings
//! let drawing_config = GridDrawingConfig {
//!     padding: 2,
//!     row_color: Rgba([255, 0, 0, 255]), // Red for rows
//!     column_color: Rgba([0, 0, 255, 255]), // Blue for columns
//!     cell_background_color: Rgb([200, 200, 200]), // Light gray for cells
//!     row_color_provider: None, // Use uniform row color
//!     column_color_provider: None, // Use uniform column color
//!     line_thickness: 1,
//! };
//!
//! // Save the image with the grid drawn on it
//! save_image_with_grid(&img, &grid, "output_with_grid.png", &drawing_config).unwrap();
//! ```

use std::fmt;

use crate::*;
#[cfg(feature = "drawing")]
use crate::{Cell, Grid, GridError};
use image::*;
#[cfg(feature = "drawing")]
use image::{DynamicImage, Rgb, Rgba, RgbaImage};
use imageproc::drawing::draw_filled_rect_mut;
#[cfg(feature = "drawing")]
use imageproc::drawing::draw_line_segment_mut;
#[cfg(feature = "drawing")]
use imageproc::rect::Rect;

/// Configuration for drawing grids, cells, and grid lines.
///
/// This struct allows customization of padding, colors, and line thickness.
/// Users can specify uniform colors for rows and columns or provide custom
/// color provider functions for more advanced configurations.
///
/// # Examples
///
/// ```
/// use grider::drawing::GridDrawingConfig;
/// use image::Rgba;
///
/// let config = GridDrawingConfig {
///     padding: 2,
///     row_color: Rgba([255, 0, 0, 255]), // Red for rows
///     column_color: Rgba([0, 0, 255, 255]), // Blue for columns
///     cell_background_color: Rgb([200, 200, 200]), // Light gray for cells
///     row_color_provider: None, // Use uniform row color
///     column_color_provider: None, // Use uniform column color
///     line_thickness: 1,
/// };
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
    /// Thickness of grid lines.
    pub line_thickness: u32,
}
// Manually implement Debug for GridDrawingConfig
impl fmt::Debug for GridDrawingConfig {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
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
        GridDrawingConfig {
            padding: 2,
            row_color: Rgba([255, 0, 0, 255]),    // Red
            column_color: Rgba([0, 0, 255, 255]), // Blue
            cell_background_color: Rgba([200, 200, 200, 255]), // Light gray
            row_color_provider: None,
            column_color_provider: None,
            line_thickness: 2,
        }
    }
}

/// Trait for types that can be drawn on an image.
///
/// This trait is implemented for [`Cell`], [`Grid`], and other types that represent
/// drawable components of a grid.
pub trait Drawable {
    /// Draws the object on the provided image using the given configuration.
    ///
    /// # Arguments
    /// * `image` - The image to draw on.
    /// * `config` - The drawing configuration.
    ///
    /// # Errors
    /// Returns [`GridError`] if drawing fails.
    fn draw(&self, image: &mut RgbaImage, config: &GridDrawingConfig) -> Result<(), GridError>;
}

impl Drawable for Cell<'_> {
    fn draw(&self, image: &mut RgbaImage, config: &GridDrawingConfig) -> Result<(), GridError> {
        let rect = Rect::from(self);

        // Adjust for padding
        let adjusted_left = rect.left() + config.padding as i32;
        let adjusted_top = rect.top() + config.padding as i32;
        let adjusted_width = rect.width() - 2 * config.padding;
        let adjusted_height = rect.height() - 2 * config.padding;

        // Draw cell background if dimensions are positive
        if adjusted_width > 0 && adjusted_height > 0 {
            let cell_rect =
                Rect::at(adjusted_left, adjusted_top).of_size(adjusted_width, adjusted_height);
            draw_filled_rect_mut(image, cell_rect, config.cell_background_color);
        }

        Ok(())
    }
}

impl Drawable for Grid {
    fn draw(&self, image: &mut RgbaImage, config: &GridDrawingConfig) -> Result<(), GridError> {
        // Draw cells with padding
        for (row_index, row) in self.rows.iter().enumerate() {
            for (col_index, column) in self.columns.iter().enumerate() {
                let cell = Cell { row, column };
                cell.draw(image, config)?;
            }
        }

        // Draw horizontal grid lines
        for (row_index, row) in self.rows.iter().enumerate() {
            let y = row.y + row.height;
            let color = if let Some(ref provider) = config.row_color_provider {
                provider(row_index)
            } else {
                config.row_color
            };
            draw_line_segment_mut(
                image,
                (0.0, y as f32),
                (image.width() as f32, y as f32),
                color,
            );
        }

        // Draw vertical grid lines
        for (col_index, column) in self.columns.iter().enumerate() {
            let x = column.x + column.width;
            let color = if let Some(ref provider) = config.column_color_provider {
                provider(col_index)
            } else {
                config.column_color
            };
            draw_line_segment_mut(
                image,
                (x as f32, 0.0),
                (x as f32, image.height() as f32),
                color,
            );
        }

        Ok(())
    }
}
