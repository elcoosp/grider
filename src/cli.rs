//! Command-line interface for `grider`.
//!
//! The binary target is a three-line wrapper around this module, which keeps
//! the CLI logic testable and fully covered by the test suite.
//!
//! # Examples
//!
//! ```bash
//! # Extract the grid and save `<image>_with_grid.png` next to the input.
//! grider tests/large.png
//!
//! # Custom configuration and JSON output (no overlay is saved).
//! grider tests/large.png --block-size 15 --delta 2 --merge-ratio 0.9 --no-parallel --json
//! ```

use anyhow::{Context, Result};
use clap::Parser;
use std::ffi::{OsStr, OsString};
use std::path::{Path, PathBuf};
use std::time::Instant;
use tracing::debug;
use tracing_subscriber::{EnvFilter, layer::SubscriberExt, util::SubscriberInitExt};

use crate::config::GridConfig;
#[cfg(feature = "drawing")]
use crate::drawing::GridDrawingConfig;
use crate::grid::Grid;
use crate::lines::LineKind;

/// Command-line arguments for the grid processing program.
#[derive(Parser, Debug)]
#[command(version, about = "Extract the row/column grid of an image")]
pub struct Cli {
    /// Path to the input image file.
    pub image_path: PathBuf,
    /// Adaptive threshold block size (minimum 3).
    #[arg(long)]
    pub block_size: Option<u32>,
    /// Offset subtracted from the local mean during adaptive thresholding.
    #[arg(long)]
    pub delta: Option<i32>,
    /// Ratio of the average line size under which lines are merged ([0, 1]).
    #[arg(long)]
    pub merge_ratio: Option<f32>,
    /// Disable parallel processing.
    #[arg(long = "no-parallel")]
    pub no_parallel: bool,
    /// Where to save the overlay image (defaults to `<image>_with_grid.png`).
    #[arg(short, long)]
    pub output: Option<PathBuf>,
    /// Print the grid as JSON instead of saving an overlay image.
    #[arg(long)]
    pub json: bool,
    /// Suppress human-readable summaries.
    #[arg(short, long)]
    pub quiet: bool,
}

/// Entry point of the binary: initializes logging, runs the CLI, maps errors
/// onto the process exit code.
#[must_use]
pub fn main() -> std::process::ExitCode {
    init_tracing();
    match run_from_env() {
        Ok(()) => std::process::ExitCode::SUCCESS,
        Err(err) => {
            eprintln!("error: {err:?}");
            std::process::ExitCode::FAILURE
        }
    }
}

/// Parses the process arguments and runs the CLI.
///
/// # Errors
///
/// Propagates any error from [`run`].
pub fn run_from_env() -> Result<()> {
    let cli = Cli::parse();
    debug!("Parsed CLI arguments: {cli:?}");
    run(&cli)
}

/// Builds the output path for the overlay image: `<image>_with_grid.png`
/// next to the input file.
#[must_use]
pub fn default_output_path(input: &Path) -> PathBuf {
    let mut file_name = OsString::from(input.file_stem().unwrap_or(OsStr::new("output")));
    file_name.push("_with_grid.png");
    input.with_file_name(file_name)
}

/// Prints the grid as pretty JSON when the `serde` feature is enabled.
///
/// Serialization of a [`Grid`] cannot fail (it contains only integers,
/// booleans and plain strings), so the result is unwrapped with a documented
/// invariant instead of threading an impossible error through the CLI.
#[cfg(feature = "serde")]
fn print_json(grid: &Grid) {
    let json = serde_json::to_string_pretty(grid)
        .expect("serializing a Grid cannot fail: it holds no maps, floats or non-string keys");
    println!("{json}");
}

/// Warns that JSON output is unavailable without the `serde` feature.
#[cfg(not(feature = "serde"))]
fn print_json(_grid: &Grid) {
    eprintln!("Skipping JSON output: built without the 'serde' feature");
}

/// Saves the overlay image when the `drawing` feature is enabled.
#[cfg(feature = "drawing")]
fn save_overlay(image: &image::DynamicImage, grid: &Grid, output_path: &Path) -> Result<()> {
    let output_str = output_path.to_string_lossy().to_string();
    crate::debug::save_image_with_grid(image, grid, &output_str, &GridDrawingConfig::default())
        .context("Failed to save overlay image")
}

/// Warns that overlay drawing is unavailable without the `drawing` feature.
#[cfg(not(feature = "drawing"))]
fn save_overlay(_image: &image::DynamicImage, _grid: &Grid, output_path: &Path) {
    eprintln!(
        "Skipping overlay '{}': built without the 'drawing' feature",
        output_path.display()
    );
}

/// Runs the grid extraction described by `cli`.
///
/// # Errors
///
/// Returns an error when the image cannot be opened, the grid cannot be
/// extracted, or the overlay image cannot be written.
pub fn run(cli: &Cli) -> Result<()> {
    let img = image::open(&cli.image_path).context("Failed to open image")?;

    let mut builder = GridConfig::builder();
    if let Some(block_size) = cli.block_size {
        builder = builder.threshold_block_size(block_size);
    }
    if let Some(delta) = cli.delta {
        builder = builder.threshold_delta(delta);
    }
    if let Some(merge_ratio) = cli.merge_ratio {
        builder = builder.merge_threshold_ratio(merge_ratio);
    }
    if cli.no_parallel {
        builder = builder.parallel(false);
    }
    let config = builder.build();

    let started = Instant::now();
    let grid = Grid::try_from_image_with_config(&img, config).context("Failed to extract grid")?;
    let elapsed = started.elapsed();

    if cli.json {
        print_json(&grid);
        return Ok(());
    }

    if !cli.quiet {
        println!("Extracted grid in {elapsed:.2?}");
        println!("{grid}");
        println!(
            "Full rows: {}, empty rows: {}, full columns: {}, empty columns: {}",
            grid.count_rows_by_kind(LineKind::Full),
            grid.count_rows_by_kind(LineKind::Empty),
            grid.count_columns_by_kind(LineKind::Full),
            grid.count_columns_by_kind(LineKind::Empty),
        );
    }

    let output_path = cli
        .output
        .clone()
        .unwrap_or_else(|| default_output_path(&cli.image_path));
    if !cli.quiet {
        println!("Saving overlay to {}", output_path.display());
    }
    #[cfg(feature = "drawing")]
    save_overlay(&img, &grid, &output_path)?;
    #[cfg(not(feature = "drawing"))]
    save_overlay(&img, &grid, &output_path);
    Ok(())
}

/// Initializes the `tracing` subscriber from `RUST_LOG`.
fn init_tracing() {
    tracing_subscriber::registry()
        .with(EnvFilter::from_default_env())
        .init();
}

#[cfg(test)]
mod tests {
    use super::{Cli, default_output_path};
    use clap::CommandFactory;
    use std::path::Path;

    #[test]
    fn cli_parses_successfully() {
        Cli::command().debug_assert();
    }

    #[test]
    fn default_output_path_appends_suffix() {
        let path = default_output_path(Path::new("tests/large.png"));
        assert_eq!(path, Path::new("tests/large_with_grid.png"));
    }

    #[test]
    fn default_output_path_falls_back_to_output_stem() {
        let path = default_output_path(Path::new(""));
        assert_eq!(path, Path::new("output_with_grid.png"));
    }
}
