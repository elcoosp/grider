use image::{DynamicImage, GenericImageView, GrayImage, ImageBuffer, Luma, Pixel};

// Define the kinds of lines (rows or columns)
#[derive(Debug, PartialEq)]
enum LineKind {
    Empty,
    Full,
}

// Define the Row struct
#[derive(Debug)]
struct Row {
    y: u32,
    height: u32,
    kind: LineKind,
}

// Define the Column struct
#[derive(Debug)]
struct Column {
    x: u32,
    width: u32,
    kind: LineKind,
}

// Define the Grid struct
#[derive(Debug)]
struct Grid {
    rows: Vec<Row>,
    columns: Vec<Column>,
}

// Trait to create lines (rows or columns)
trait LineTrait {
    fn new(start: u32, length: u32, kind: LineKind) -> Self;
}

// Implement the trait for Row
impl LineTrait for Row {
    fn new(start: u32, length: u32, kind: LineKind) -> Self {
        Row {
            y: start,
            height: length,
            kind,
        }
    }
}

// Implement the trait for Column
impl LineTrait for Column {
    fn new(start: u32, length: u32, kind: LineKind) -> Self {
        Column {
            x: start,
            width: length,
            kind,
        }
    }
}

// Function to check if a row is empty
fn is_row_empty(img: &GrayImage, y: u32, width: u32) -> bool {
    for x in 0..width {
        if img.get_pixel(x, y).channels()[0] != 255 {
            return false;
        }
    }
    true
}

// Function to check if a column is empty
fn is_column_empty(img: &GrayImage, x: u32, height: u32) -> bool {
    for y in 0..height {
        if img.get_pixel(x, y).channels()[0] != 255 {
            return false;
        }
    }
    true
}

// Generic function to process lines (rows or columns)
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

// The process_image function
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
fn main() {
    // Replace with the path to your image file
    let image_path = "tests/13.png";

    // Open the image file
    match image::open(image_path) {
        Ok(img) => {
            // Process the image
            let grid = process_image(img);

            // Print the grid (or use it as needed)
            println!("{:?}", grid);
        }
        Err(e) => {
            eprintln!("Failed to open image: {}", e);
        }
    }
}
