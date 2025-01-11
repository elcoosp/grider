# Grider: Image Grid Processing for Rust 🖼️

## ⚠️ **Beta Software Warning** ⚠️

**Grider is currently in beta.**  
This means the library is under active development, and **breaking changes may occur without following semantic versioning**.  
Use it at your own risk, and expect potential API changes in future releases.  
If you rely on Grider in production, pin the version explicitly in your `Cargo.toml`.

---

## Badges

[![CI](https://github.com/elcoosp/grider/actions/workflows/ci.yml/badge.svg)](https://github.com/elcoosp/grider/actions/workflows/ci.yml)  
[![Crates.io](https://img.shields.io/crates/v/grider)](https://crates.io/crates/grider)  
[![Docs](https://docs.rs/grider/badge.svg)](https://docs.rs/grider)  
[![License](https://img.shields.io/crates/l/grider)](https://github.com/elcoosp/grider/blob/main/LICENSE)

## Table of Contents

1. [Installation](#installation)
2. [Usage](#usage)
   - [Creating a Grid from an Image](#creating-a-grid-from-an-image)
   - [Accessing Rows and Columns](#accessing-rows-and-columns)
   - [Filtering Rows and Columns](#filtering-rows-and-columns)
   - [Custom Configuration](#custom-configuration)
   - [Debugging with Visual Grid](#debugging-with-visual-grid)
3. [API Documentation](#api-documentation)
   - [Structs](#structs)
   - [Enums](#enums)
   - [Functions](#functions)
   - [Macros](#macros)
4. [Examples](#examples)
5. [Contributing](#contributing)
6. [License](#license)

---

## Installation

Add `grider` to your `Cargo.toml`:

```toml
[dependencies]
grider = "0.1"  # Pin the version to avoid unexpected breaking changes
```

And include it in your Rust project:

```rust
use grider::*;
```

---

## Usage

### Creating a Grid from an Image

```rust
use grider::{Grid, GridConfig};
use image::open;

let img = open("path/to/image.png").unwrap();
let config = GridConfig::default();
let grid = Grid::try_from_image_with_config(&img, config).unwrap();
```

### Accessing Rows and Columns

```rust
for row in grid.rows() {
    println!("Row at y: {}, height: {}, kind: {:?}", row.y, row.height, row.kind);
}

for column in grid.columns() {
    println!("Column at x: {}, width: {}, kind: {:?}", column.x, column.width, column.kind);
}
```

### Filtering Rows and Columns

You can filter rows and columns based on their size or kind:

```rust
// Filter out the smallest rows
let filtered_grid = grid.filter_smallest_rows();

// Filter out the biggest columns
let filtered_grid = grid.filter_biggest_columns();

// Filter rows by kind (e.g., only full rows)
let full_rows: Vec<_> = grid.filtered_rows(|row| row.kind == LineKind::Full).collect();

// Filter columns by kind (e.g., only empty columns)
let empty_columns: Vec<_> = grid.filtered_columns(|col| col.kind == LineKind::Empty).collect();
```

### Custom Configuration

You can customize the grid processing by adjusting the `GridConfig`:

```rust
let config = GridConfig::new(
    15,          // Threshold block size for adaptive thresholding
    0.9,         // Merge threshold ratio for merging small lines
    false,       // Disable parallel processing
);
let grid = Grid::try_from_image_with_config(&img, config).unwrap();
```

### Debugging with Visual Grid

You can visualize the grid on the original image for debugging purposes:

```rust
use grider::debug::save_image_with_grid;
use grider::drawing::GridDrawingConfig;

let drawing_config = GridDrawingConfig {
    padding: 0,
    row_color: Rgba([255, 0, 0, 255]), // Red for rows
    column_color: Rgba([0, 0, 255, 255]), // Blue for columns
    cell_background_color: Rgba([200, 200, 200, 255]), // Light gray for cells
    line_thickness: 1,
};

save_image_with_grid(&img, &grid, "output_with_grid.png", &drawing_config).unwrap();
```

---

## API Documentation

### Structs

- **Grid**: Represents the grid of rows and columns extracted from an image.
  - **Rows**: List of rows in the grid.
  - **Columns**: List of columns in the grid.

- **Row**: Represents a row in the grid.
  - **y**: Y-coordinate of the row.
  - **height**: Height of the row.
  - **kind**: Type of the row (`LineKind::Empty` or `LineKind::Full`).

- **Column**: Represents a column in the grid.
  - **x**: X-coordinate of the column.
  - **width**: Width of the column.
  - **kind**: Type of the column (`LineKind::Empty` or `LineKind::Full`).

- **GridConfig**: Configuration for grid processing.
  - **threshold_block_size**: Block size for adaptive thresholding.
  - **merge_threshold_ratio**: Ratio for merging small lines.
  - **enable_parallel**: Enable parallel processing.

- **GridDrawingConfig**: Configuration for drawing grids, cells, and grid lines on images.

### Enums

- **LineKind**: Represents the type of a line (row or column).
  - **Empty**: Indicates an empty line.
  - **Full**: Indicates a full line.

### Functions

- **Grid::try_from_image_with_config**: Creates a grid from an image with custom configuration.
- **Grid::rows**: Returns an iterator over all rows in the grid.
- **Grid::columns**: Returns an iterator over all columns in the grid.
- **Grid::filtered_rows**: Returns an iterator over filtered rows based on a predicate.
- **Grid::filtered_columns**: Returns an iterator over filtered columns based on a predicate.
- **Grid::count_rows_by_kind**: Counts the number of rows with the specified kind.
- **Grid::count_columns_by_kind**: Counts the number of columns with the specified kind.
- **Grid::find_cells**: Finds cells based on row and column indices.
- **Grid::filter_smallest_rows**: Filters out the smallest rows in the grid.
- **Grid::filter_biggest_rows**: Filters out the biggest rows in the grid.
- **Grid::filter_smallest_columns**: Filters out the smallest columns in the grid.
- **Grid::filter_biggest_columns**: Filters out the biggest columns in the grid.

### Macros

- **make_grid!**: Creates a grid from rows and columns with a convenient macro.
- **make_line!**: Creates a row or column from a tuple of parameters.

---

## Examples

- **Basic Grid Extraction**: Demonstrates how to extract a grid from an image and print row and column information.
- **Custom Configuration**: Shows how to adjust grid processing parameters for different image types.
- **Filtering Rows and Columns**: Illustrates how to filter rows and columns based on size or kind.
- **Debugging with Visual Grid**: Illustrates how to visualize the grid on the original image for debugging.

---

## Contributing

Contributions are welcome! Please read the [CONTRIBUTING.md](CONTRIBUTING.md) file for guidelines on how to contribute to this project.

---

## License

Grider is licensed under the MIT License. See the [LICENSE](LICENSE) file for details.

---
