use drawing::*;
use image::DynamicImage;

use super::*;
/// Saves the image with the grid drawn on it.
///
/// This function draws the grid, cells, and grid lines on the provided image
/// and saves the result to the specified path.
///
/// # Arguments
/// * `image` - The input image.
/// * `grid` - The grid to draw.
/// * `output_path` - The path to save the output image.
/// * `config` - The drawing configuration.
///
/// # Errors
/// Returns [`GridError`] if drawing or saving fails.
///
/// # Examples
///
/// ```rust
/// use grider::{*, drawing::*};
/// use image::open;
///
/// let img = open("tests/large.png").unwrap();
/// let config = GridConfig::default();
/// let grid = Grid::try_from_image_with_config(&img, config).unwrap();
///
/// let drawing_config = GridDrawingConfig::default();
/// debug::save_image_with_grid(&img, &grid, "output_with_grid.png", &drawing_config).unwrap();
/// ```
pub fn save_image_with_grid(
    image: &DynamicImage,
    grid: &Grid,
    output_path: &str,
    config: &GridDrawingConfig,
) -> Result<(), GridError> {
    let mut rgba_img = image.to_rgba8();
    #[cfg(feature = "drawing")]
    grid.draw(&mut rgba_img, config)?;
    rgba_img
        .save(output_path)
        .map_err(|e| GridError::ImageConversionError(e.to_string()))
}
