use image::*;
use imageproc::drawing::draw_line_segment_mut;

/// Represents the kind of a line (row or column).
///
/// A line can be either [`LineKind::Empty`] (fully white) or [`LineKind::Full`] (contains non-white pixels).
#[derive(Debug, PartialEq)]
enum LineKind {
    Empty,
    Full,
}

/// Represents a row in the grid.
///
/// A row is defined by its starting y-coordinate (`y`), height (`height`), and [`LineKind`].
#[derive(Debug)]
struct Row {
    y: u32,
    height: u32,
    kind: LineKind,
}

/// Represents a column in the grid.
///
/// A column is defined by its starting x-coordinate (`x`), width (`width`), and [`LineKind`].
#[derive(Debug)]
struct Column {
    x: u32,
    width: u32,
    kind: LineKind,
}

/// Represents the grid composed of [`Row`]s and [`Column`]s.
///
/// The grid is generated by processing an image and grouping consecutive rows and columns
/// based on their [`LineKind`].
#[derive(Debug)]
struct Grid {
    rows: Vec<Row>,
    columns: Vec<Column>,
}

/// Trait to create lines (rows or columns).
///
/// This trait is implemented for both [`Row`] and [`Column`] to allow generic processing of lines.
trait LineTrait {
    fn new(start: u32, length: u32, kind: LineKind) -> Self;
}

/// Implement the trait for [`Row`].
impl LineTrait for Row {
    fn new(start: u32, length: u32, kind: LineKind) -> Self {
        Row {
            y: start,
            height: length,
            kind,
        }
    }
}

/// Implement the trait for [`Column`].
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
fn is_row_empty(img: &GrayImage, y: u32, width: u32) -> bool {
    for x in 0..width {
        if img.get_pixel(x, y).channels()[0] != 255 {
            return false;
        }
    }
    true
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
fn is_column_empty(img: &GrayImage, x: u32, height: u32) -> bool {
    for y in 0..height {
        if img.get_pixel(x, y).channels()[0] != 255 {
            return false;
        }
    }
    true
}

/// Processes lines (rows or columns) and groups them by their [`LineKind`].
///
/// This function is generic over [`LineTrait`] and can process both [`Row`]s and [`Column`]s.
///
/// # Arguments
/// * `img` - The grayscale image to process.
/// * `length` - The length of the lines (height for rows, width for columns).
/// * `is_empty` - A function to check if a line is empty.
///
/// # Returns
/// A vector of lines grouped by their [`LineKind`].
fn process_lines<T>(img: &GrayImage, length: u32, is_empty: impl Fn(u32) -> bool) -> Vec<T>
where
    T: LineTrait,
{
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
            lines.push(T::new(current_start, current_length, current_kind));
            current_start = i;
            current_kind = new_kind;
            current_length = 1;
        }
    }
    // Push the last line
    lines.push(T::new(current_start, current_length, current_kind));
    lines
}

/// Processes the image and generates the [`Grid`].
///
/// This function converts the image to grayscale, binarizes it, and processes it to generate
/// the [`Grid`] of [`Row`]s and [`Column`]s.
///
/// # Arguments
/// * `image` - The input image to process.
///
/// # Returns
/// A [`Grid`] representing the rows and columns of the image.
fn process_image(image: DynamicImage) -> Grid {
    // Convert the image to grayscale
    let img = image.into_luma8();
    let (width, height) = img.dimensions();

    // Binarize the image using a threshold of 128
    let binarized_img = ImageBuffer::from_vec(
        width,
        height,
        img.into_iter()
            .map(|p| if p >= &128 { 255u8 } else { 0u8 })
            .collect(),
    )
    .unwrap();

    // Process rows
    let rows = process_lines(&binarized_img, height, |y| {
        is_row_empty(&binarized_img, y, width)
    });

    // Process columns
    let columns = process_lines(&binarized_img, width, |x| {
        is_column_empty(&binarized_img, x, height)
    });

    // Create the Grid
    Grid { rows, columns }
}

/// Debug module for visualizing the grid on the image.
mod debug {
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
        println!("Image with grid lines saved to {}", output_path);
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
            debug::save_image_with_grid(&img, &grid, "output_with_grid.png");
        }
        Err(e) => {
            eprintln!("Failed to open image: {}", e);
        }
    }
}
