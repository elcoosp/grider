//! This module provides functionality for processing images into grids of rows and columns.
//! It uses the `image` and `imageproc` crates for image manipulation and `insta` for snapshot testing.

/// Debug module for visualizing the grid on the image.
///
/// # Example
/// ```
/// use grider::{*, drawing::*};
/// use image::DynamicImage;
///
/// // Replace with actual image loading
/// let img: DynamicImage = image::open("tests/large.png").unwrap();
/// let config = GridConfig::default();
/// let grid = Grid::try_from_image_with_config(&img, config).unwrap();
///
/// grider::debug::save_image_with_grid(&img, &grid, "output.png", &GridDrawingConfig::default());
/// ```
pub mod debug;
/// This module provides functionality for drawing grids, cells, and grid lines on images.
/// It is feature-gated under the `drawing` feature and requires the `image` and `imageproc` crates.
///
/// The main components of this module are:
/// - [`GridDrawingConfig`]: Configuration for customizing the appearance of grids, cells, and grid lines.
/// - [`Drawable`]: A trait implemented by types that can be drawn on an image, such as [`Cell`] and [`Grid`].
///
/// # Examples
///
/// ```rust
/// use grider::{*, drawing::*};
/// use image::*;
///
/// // Load an image
/// let img = open("tests/large.png").unwrap();
///
/// // Create a grid from the image
/// let config = GridConfig::default();
/// let grid = Grid::try_from_image_with_config(&img, config).unwrap();
///
/// // Configure drawing settings
/// let drawing_config = GridDrawingConfig {
///     padding: 0,
///     row_color: Rgba([255, 0, 0, 255]), // Red for rows
///     column_color: Rgba([0, 0, 255, 255]), // Blue for columns
///     cell_background_color: Rgba([200, 200, 200, 255]), // Light gray for cells
///     row_color_provider: None, // Use uniform row color
///     column_color_provider: None, // Use uniform column color
///     line_thickness: 1,
/// };
///
/// // Save the image with the grid drawn on it
/// debug::save_image_with_grid(&img, &grid, "output_with_grid.png", &drawing_config).unwrap();
/// ```
pub mod drawing;

pub mod grid;
pub mod grid_like;
pub mod grid_subset;

pub use grid::*;
