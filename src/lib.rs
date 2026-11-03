//! High-performance extraction of row and column grids from images.
//!
//! `grider` reads an image, binarizes it with an adaptive threshold, detects
//! the runs of background/content scan lines on both axes and returns them as
//! a [`Grid`] of [`Row`]s and [`Column`]s — the skeleton of tables, spreadsheets
//! and document layouts.
//!
//! # Quick start
//!
//! ```
//! use grider::{Grid, GridConfig};
//! use image::open;
//!
//! let img = open("tests/large.png").unwrap();
//! let config = GridConfig::default();
//! let grid = Grid::try_from_image_with_config(&img, config).unwrap();
//!
//! for row in grid.rows() {
//!     println!("Row at y: {}", row.y);
//! }
//! ```
//!
//! # Configuration
//!
//! Extraction is driven by [`GridConfig`], either through its constructor or
//! its builder:
//!
//! ```
//! use grider::GridConfig;
//!
//! let config = GridConfig::builder()
//!     .threshold_block_size(15)
//!     .threshold_delta(0)
//!     .merge_threshold_ratio(0.9)
//!     .parallel(false)
//!     .build();
//! ```
//!
//! The binarization step is configurable too — the default adaptive mean
//! threshold is bit-for-bit identical to `imageproc`, while
//! [`Binarization::Sauvola`] targets degraded document images:
//!
//! ```
//! use grider::GridConfig;
//!
//! let config = GridConfig::builder().sauvola(0.34).build();
//! ```
//!
//! # Filtering and querying
//!
//! ```
//! # use grider::{Grid, GridConfig};
//! # use image::open;
//! # let img = open("tests/large.png").unwrap();
//! let grid = Grid::try_from_image_with_config(&img, GridConfig::default()).unwrap();
//!
//! // Filter out structural lines (headers/footers) by size…
//! let grid = grid.filter_biggest_rows();
//! // …or keep only the content lines.
//! let full = grid.keep_kind(grider::LineKind::Full);
//!
//! // Visit every intersection of the remaining lines.
//! for cell in full.iter_cells() {
//!     let rect = cell.rect();
//!     println!("cell at {}x{}", rect.left(), rect.top());
//! }
//! ```
//!
//! # Debugging with a visual overlay
//!
//! > Tune your `GridConfig` by inspecting the overlay image produced by
//! > [`debug::save_image_with_grid`] — see its documentation for a full example.
//!
//! # Command line
//!
//! The crate ships a binary with the same powers as the library:
//!
//! ```bash
//! grider tests/large.png --block-size 15 --json
//! ```
//!
//! # Feature flags
//!
//! | Feature | Default | Description |
//! |---------|---------|-------------|
//! | `serde` | yes | `Serialize`/`Deserialize` for all data types + JSON in the CLI. |
//! | `drawing` | yes | Grid drawing support and the [`debug`] overlay helpers. |
//! | `test-utils` | no | Shared image fixtures for tests and benchmarks. |
//!
//! # Performance
//!
//! The pipeline is `O(width × height)` regardless of the threshold window
//! radius: binarization runs on a streaming sliding-window integral image
//! (Bradley & Roth, 2005; memory `O(width × radius)`), the emptiness scan is
//! a single cache-friendly pass computing row and column flags together, and
//! both steps parallelize over row bands. See `docs/BENCHMARKS.md` for
//! measured numbers.
//!
//! # Error handling
//!
//! All fallible operations return [`GridError`], a `#[non_exhaustive]` error
//! enum covering invalid dimensions, invalid configuration, missing lines and
//! image conversion failures.
//!
//! Lint policy lives in `Cargo.toml` (`[lints]`): `forbid(unsafe_code)`,
//! `deny(missing_docs)` and pedantic clippy, with no `allow` attributes
//! anywhere in the codebase.

/// Binarization algorithms (integral-image adaptive mean and Sauvola).
pub mod binarize;
/// Command-line interface of the `grider` binary.
pub mod cli;
/// Configuration for grid processing.
pub mod config;
/// Debug helpers for visualizing the grid on the image.
#[cfg(feature = "drawing")]
pub mod debug;
/// The image-to-lines detection pipeline.
pub mod detection;
/// Drawing grids, cells, and grid lines on images.
#[cfg(feature = "drawing")]
pub mod drawing;
/// Error types produced by grid extraction and drawing.
pub mod error;
/// The [`Grid`] type: rows and columns extracted from an image.
pub mod grid;
/// Line primitives: kinds, raw line info, rows, columns and cells.
pub mod lines;
/// Convenience macros for building grids and lines.
pub mod macros;
/// Shared fixtures for tests and benchmarks.
#[cfg(feature = "test-utils")]
pub mod testing;

pub use crate::config::{
    Binarization, DEFAULT_MERGE_THRESHOLD_RATIO, DEFAULT_THRESHOLD_BLOCK_SIZE,
    DEFAULT_THRESHOLD_DELTA, GridConfig, GridConfigBuilder, MIN_THRESHOLD_BLOCK_SIZE,
    SAUVOLA_DEFAULT_K, SAUVOLA_DYNAMIC_RANGE,
};
pub use crate::detection::{LineFlags, SmallVecLine, lines_from_flags, scan_line_flags};
pub use crate::error::GridError;
pub use crate::grid::Grid;
pub use crate::lines::{Cell, Column, LineInfo, LineKind, LineTrait, Row, clamp_to_i32};
