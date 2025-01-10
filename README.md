# Grider: Image Grid Processing for Rust

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

## Installation

Add `grider` to your `Cargo.toml`:

```toml
[dependencies]
grider = "0.1"
```

And include it in your Rust project:

```rust
use grider::*;
```

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
    println!("Row at y: {}", row.y);
}

for column in grid.columns() {
    println!("Column at x: {}", column.x);
}
```

### Custom Configuration

```rust
let config = GridConfig::new(
    15,          // Threshold block size
    0.9,         // Merge threshold ratio
    false,       // Disable parallel processing
);
let grid = Grid::try_from_image_with_config(&img, config).unwrap();
```

### Debugging with Visual Grid

```rust
grider::debug::save_image_with_grid(&img, &grid, "output_with_grid.png");
```

## API Documentation

### Structs

- **Grid**: Represents the grid of rows and columns extracted from an image.
  - **Rows**: List of rows in the grid.
  - **Columns**: List of columns in the grid.

- **Row**: Represents a row in the grid.
  - **y**: Y-coordinate of the row.
  - **height**: Height of the row.
  - **kind**: Type of the row (Empty or Full).

- **Column**: Represents a column in the grid.
  - **x**: X-coordinate of the column.
  - **width**: Width of the column.
  - **kind**: Type of the column (Empty or Full).

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

### Macros

- **make_grid!**: Creates a grid from rows and columns with a convenient macro.
- **make_line!**: Creates a row or column from a tuple of parameters.

## Examples

- **Basic Grid Extraction**: Demonstrates how to extract a grid from an image and print row and column information.
- **Custom Configuration**: Shows how to adjust grid processing parameters for different image types.
- **Debugging with Visual Grid**: Illustrates how to visualize the grid on the original image for debugging.

## Contributing

Contributions are welcome! Please read the [CONTRIBUTING.md](CONTRIBUTING.md) file for guidelines on how to contribute to this project.

## License

Grider is licensed under the MIT License. See the [LICENSE](LICENSE) file for details.
