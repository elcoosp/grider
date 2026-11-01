//! Debug helpers for visualizing the grid on the image.
//!
//! # Examples
//!
//! ```
//! use grider::drawing::GridDrawingConfig;
//! use grider::{Grid, GridConfig};
//! use image::DynamicImage;
//!
//! let img: DynamicImage = image::open("tests/large.png").unwrap();
//! let grid = Grid::try_from_image_with_config(&img, GridConfig::default()).unwrap();
//!
//! grider::debug::save_image_with_grid(
//!     &img,
//!     &grid,
//!     "output.png",
//!     &GridDrawingConfig::default(),
//! )
//! .unwrap();
//! ```

use image::DynamicImage;

use crate::drawing::{Drawable, GridDrawingConfig};
use crate::error::GridError;
use crate::grid::Grid;

/// Saves the image with the grid drawn on it.
///
/// The grid lines, cells, and backgrounds are drawn according to `config`
/// before the image is written to `output_path`.
///
/// # Errors
///
/// Returns [`GridError::ImageConversionError`] if saving fails.
///
/// # Examples
///
/// ```
/// use grider::drawing::GridDrawingConfig;
/// use grider::{Grid, GridConfig};
/// use image::open;
///
/// let img = open("tests/large.png").unwrap();
/// let grid = Grid::try_from_image_with_config(&img, GridConfig::default()).unwrap();
///
/// let drawing_config = GridDrawingConfig::default();
/// grider::debug::save_image_with_grid(&img, &grid, "output_with_grid.png", &drawing_config).unwrap();
/// ```
pub fn save_image_with_grid(
    image: &DynamicImage,
    grid: &Grid,
    output_path: &str,
    config: &GridDrawingConfig,
) -> Result<(), GridError> {
    let mut rgba_img = image.to_rgba8();
    // Drawing is infallible (geometry is clipped to the image bounds); only
    // the file write below can fail.
    grid.draw(&mut rgba_img, config);
    rgba_img
        .save(output_path)
        .map_err(|e| GridError::ImageConversionError(e.to_string()))
}
