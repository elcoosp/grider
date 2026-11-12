<div align="center">
  <img src="https://cdn.jsdelivr.net/gh/elcoosp/grider@main/docs/logo.png" alt="Grider Logo" width="300"/>
  <p>
    <strong>Image Grid Processing for Rust</strong><br/>
    A high-performance library for extracting, analyzing, and manipulating row and column grids from images.
  </p>
  <p>
    <a href="https://crates.io/crates/grider"><img src="https://img.shields.io/crates/v/grider?style=flat-square" alt="Crates.io"/></a>
    <a href="https://docs.rs/grider"><img src="https://img.shields.io/badge/docs.rs-grider-blue?style=flat-square" alt="Documentation"/></a>
    <a href="https://github.com/elcoosp/grider/actions/workflows/ci.yml"><img src="https://img.shields.io/github/actions/workflow/status/elcoosp/grider/ci.yml?branch=main&style=flat-square&label=CI" alt="CI"/></a>
    <a href="https://github.com/elcoosp/grider/blob/main/LICENSE"><img src="https://img.shields.io/crates/l/grider?style=flat-square" alt="License"/></a>
  </p>
</div>

---

## Highlights

- **One-line extraction**: `Grid::try_from_image_with_config(&img, config)` returns every row and
  column band of the image, tagged as `Empty` (background) or `Full` (content).
- **Fast by construction**: binarization runs on a streaming sliding-window integral image
  (1.6–3.1× faster than the previous `imageproc`-based path, bit-for-bit identical output), the
  emptiness scan is a single fused cache-friendly pass, and both steps parallelize over row bands
  with work-size gating so small images never pay thread-pool overhead. See
  [docs/BENCHMARKS.md](https://github.com/elcoosp/grider/blob/main/docs/BENCHMARKS.md).
- **SOTA binarization options**: the adaptive mean threshold (Bradley & Roth) or
  [`Binarization::Sauvola`](https://doi.org/10.1016/S0031-3203(99)00055-2) for degraded documents —
  both `O(width × height)` regardless of the window radius, with `O(width × radius)` extra memory.
- **Ergonomic API**: builders for `GridConfig` and `GridDrawingConfig`, iterators over cells,
  kind-based filters, in-place `retain`, serde support, and visual debug overlays.
- **Quality enforced**: `unsafe` forbidden, missing docs denied, clippy **pedantic** clean with
  *zero* `allow` attributes, and coverage gates (100 % functions, 100 % lines modulo llvm-cov's
  duplicate-instantiation accounting artifacts) enforced in CI.

## Installation

Add `grider` to your `Cargo.toml` dependencies:

```toml
[dependencies]
grider = "0.2"
```

Then, import it into your Rust project:

```rust
use grider::*;
```

## Usage

### Creating a Grid from an Image

```rust
use grider::{Grid, GridConfig};
use image::open;

let img = open("path/to/image.png").expect("Failed to open image");
let config = GridConfig::default();
let grid = Grid::try_from_image_with_config(&img, config).expect("Failed to create grid");
```

### Accessing Rows and Columns

```rust
for row in grid.rows() {
    println!("Row at y: {} ({:?})", row.y, row.kind);
}

for column in grid.columns() {
    println!("Column at x: {}", column.x);
}

// Every intersection of the grid:
for cell in grid.iter_cells() {
    let rect = cell.rect();
    println!("cell {}x{} ({}x{})", rect.left(), rect.top(), rect.width(), rect.height());
}
```

### Custom Configuration

Fine-tune the grid extraction with the builder:

```rust
let config = GridConfig::builder()
    .threshold_block_size(15)  // adaptive threshold radius (min 3)
    .threshold_delta(0)        // offset subtracted from the local mean
    .merge_threshold_ratio(0.9)
    .parallel(false)
    .build();
let grid = Grid::try_from_image_with_config(&img, config).expect("Failed to create grid");
```

Scanning degraded or unevenly lit documents? Switch the binarization to
Sauvola:

```rust
use grider::GridConfig;

let config = GridConfig::builder().sauvola(0.34).build();
```

> [!TIP]
> **Debugging with a visual overlay**
> Overlay the extracted grid on the original image — invaluable while tuning your `GridConfig`.
>
> ```rust
> use grider::drawing::GridDrawingConfig;
> # use grider::{Grid, GridConfig};
> # use image::open;
> # let img = open("tests/large.png").unwrap();
> # let grid = Grid::try_from_image_with_config(&img, GridConfig::default()).unwrap();
> grider::debug::save_image_with_grid(
>     &img,
>     &grid,
>     "output_with_grid.png",
>     &GridDrawingConfig::default(),
> )
> .unwrap();
> ```

### Command line

The binary offers the same extraction pipeline for quick inspection:

```bash
# Extract the grid and save <image>_with_grid.png next to the input
grider tests/large.png

# Custom configuration, JSON output to stdout (no overlay is written)
grider tests/large.png --block-size 15 --delta 2 --merge-ratio 0.9 --no-parallel --json

# Sauvola binarization for degraded documents
grider scan.png --binarization sauvola --sauvola-k 0.34 --json
```

## API Reference

### Core Structs

| Struct | Description |
| :--- | :--- |
| **`Grid`** | The extracted grid: `SmallVecLine<Row>` + `SmallVecLine<Column>` with a rich query API. |
| **`Row`** / **`Column`** | A horizontal/vertical band: start coordinate, size and `LineKind`. |
| **`Cell`** | The intersection of a `Row` and a `Column` (`rect()` gives the covered rectangle). |
| **`GridConfig`** | Extraction settings with `builder()` and `validate()`. |
| **`GridDrawingConfig`** | Overlay rendering settings with `builder()` and color providers. |

### Enums

| Enum | Variants | Description |
| :--- | :--- | :--- |
| **`LineKind`** | `Empty`, `Full` | Content density of a line (with `is_empty()` / `is_full()` helpers). |
| **`GridError`** | `#[non_exhaustive]` | Invalid dimensions/config, missing lines, image conversion failures. |

### Key Functions

| Function | Description |
| :--- | :--- |
| `Grid::try_from_image_with_config` | Creates a grid from an image using a custom configuration. |
| `Grid::try_from_gray_with_config` | Same, directly from a `GrayImage` (no conversion, no copy). |
| `Grid::try_from_binarized` | Creates a grid from an already binarized image (skips binarization). |
| `Grid::iter_cells` / `Grid::cell` | Iterate all cells / index-based cell lookup. |
| `Grid::find_cells` / `Grid::find_row` / `Grid::find_column` | Coordinate-based lookups. |
| `Grid::filtered_rows` / `Grid::filtered_columns` | Iterator over lines matching a predicate. |
| `Grid::rows_by_kind` / `Grid::count_rows_by_kind` | Kind-based access and counting (same for columns). |
| `Grid::filter_smallest/biggest_rows/columns[_with_tolerance]` | Remove structural lines by size. |
| `Grid::retain_rows` / `Grid::retain_columns` / `Grid::keep_kind` | In-place / copy kind filters. |
| `Grid::dimensions` | Overall `(width, height)` extent of the grid. |

### Pipeline functions

The detection internals are public in [`grider::detection`](https://docs.rs/grider):
`is_row_empty`, `is_column_empty`, `run_length_encode`, `collect_lines`,
`merge_small_lines`, `process_dimension`, `scan_line_flags`, `lines_from_flags`
(with `Grid::`-level compatibility wrappers), as are the binarizers in
[`grider::binarize`](https://docs.rs/grider): `adaptive_mean_threshold` (bit-for-bit
identical to `imageproc::contrast::adaptive_threshold`) and `sauvola_threshold`.

### Macros

| Macro | Description |
| :--- | :--- |
| **`make_grid!`** | Constructs a `Grid` from lists of row/column tuples. |
| **`make_line!`** | Quickly creates a `Row` or `Column` from a tuple. |

## Feature flags

| Feature | Default | Description |
| :--- | :--- | :--- |
| `serde` | ✅ | `Serialize`/`Deserialize` for all data types, JSON in the CLI. |
| `drawing` | ✅ | Grid drawing support and the debug overlay helpers. |
| `test-utils` | — | Shared image fixtures for tests and benchmarks (`grider::testing`). |

## Development

```bash
cargo test --features test-utils            # run the test suite
cargo clippy --all-targets --all-features -- -D warnings   # pedantic-clean, zero allows
cargo llvm-cov --all-features --workspace \
  --fail-under-functions 100 --fail-under-lines 99.8 --fail-under-regions 99
cargo bench --features test-utils           # criterion benchmarks (see docs/BENCHMARKS.md)
```

See CONTRIBUTING.md for guidelines and
[docs/BRAINSTORM.md](https://github.com/elcoosp/grider/blob/main/docs/BRAINSTORM.md) for the design notes behind the 0.2 release.

> [!NOTE]
> This project is distributed under the terms of the MIT License. See LICENSE for details.
