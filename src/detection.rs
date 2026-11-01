//! The image-to-lines detection pipeline.
//!
//! The pipeline is a straight line of steps:
//!
//! 1. **scan** every scan line of one axis and record whether it is empty
//!    (parallel across lines when enabled),
//! 2. **run-length encode** the flags into [`LineInfo`] runs,
//! 3. **merge** runs that are smaller than a ratio of the average run size,
//! 4. project the runs into [`Row`]s or [`Column`]s via [`LineTrait`].
//!
//! Steps 2–4 are pure functions over plain data, which keeps them trivial to
//! test and reason about; only the scan step touches the image.

use image::GrayImage;
use rayon::iter::{IntoParallelIterator, ParallelIterator};
use smallvec::SmallVec;
use tracing::debug;

use crate::config::MIN_THRESHOLD_BLOCK_SIZE;
use crate::error::GridError;
use crate::lines::{LineInfo, LineKind, LineTrait};

/// Checks whether a row is empty (all its pixels are white).
///
/// The scan is a zero-copy walk over the image's raw buffer, so it performs a
/// single bounds check per row instead of one per pixel.
///
/// # Arguments
/// * `img` - The grayscale image to check.
/// * `y` - The y-coordinate of the row.
/// * `width` - The number of leading pixels of the row to check.
///
/// # Panics
/// Panics if `y` is outside the image, mirroring `get_pixel` semantics.
///
/// # Examples
///
/// ```
/// use grider::detection::is_row_empty;
/// use image::{GrayImage, Luma};
///
/// let img = GrayImage::from_raw(3, 2, vec![255, 255, 255, 255, 0, 255]).unwrap();
/// assert!(is_row_empty(&img, 0, 3));
/// assert!(!is_row_empty(&img, 1, 3));
/// ```
#[must_use]
pub fn is_row_empty(img: &GrayImage, y: u32, width: u32) -> bool {
    let stride = img.width() as usize;
    let start = y as usize * stride;
    img.as_raw()[start..start + width as usize]
        .iter()
        .all(|&pixel| pixel == u8::MAX)
}

/// Checks whether a column is empty (all its pixels are white).
///
/// The scan walks the raw buffer with a row-major stride and exits on the
/// first non-background pixel.
///
/// # Arguments
/// * `img` - The grayscale image to check.
/// * `x` - The x-coordinate of the column.
/// * `height` - The number of leading pixels of the column to check.
///
/// # Panics
/// Panics if `x` is outside the image, mirroring `get_pixel` semantics.
///
/// # Examples
///
/// ```
/// use grider::detection::is_column_empty;
/// use image::{GrayImage, Luma};
///
/// let img = GrayImage::from_raw(3, 2, vec![255, 255, 0, 255, 255, 0]).unwrap();
/// assert!(is_column_empty(&img, 0, 2));
/// assert!(!is_column_empty(&img, 2, 2));
/// ```
#[must_use]
pub fn is_column_empty(img: &GrayImage, x: u32, height: u32) -> bool {
    let stride = img.width() as usize;
    let raw = img.as_raw();
    (0..height as usize).all(|y| raw[y * stride + x as usize] == u8::MAX)
}

/// Converts a sequence of per-scan-line emptiness flags into [`LineInfo`] runs.
///
/// This is the pure run-length encoding step of the pipeline: consecutive
/// flags of equal value become a single run. A `true` flag means the scan
/// line is background and maps to [`LineKind::Empty`], a `false` flag means
/// content and maps to [`LineKind::Full`].
///
/// # Examples
///
/// ```
/// use grider::detection::run_length_encode;
/// use grider::{LineInfo, LineKind};
///
/// let runs = run_length_encode(&[false, false, true, false]);
/// assert_eq!(runs, vec![
///     LineInfo::new(0, 2, LineKind::Full),
///     LineInfo::new(2, 1, LineKind::Empty),
///     LineInfo::new(3, 1, LineKind::Full),
/// ]);
/// ```
#[must_use]
pub fn run_length_encode(flags: &[bool]) -> Vec<LineInfo> {
    let mut runs: Vec<LineInfo> = Vec::new();
    for (position, &flag) in flags.iter().enumerate() {
        let kind = if flag {
            LineKind::Empty
        } else {
            LineKind::Full
        };
        match runs.last_mut() {
            Some(last) if last.kind == kind => last.length += 1,
            _ => runs.push(LineInfo::new(
                u32::try_from(position).unwrap_or(u32::MAX),
                1,
                kind,
            )),
        }
    }
    runs
}

/// Collects the initial (unmerged) [`LineInfo`] runs of one axis from an image.
///
/// `is_empty_fn` is called once per scan line with `(image, line_index,
/// secondary_dimension)`, allowing custom definitions of "empty" (e.g. a
/// grey-level tolerance). Prefer the built-in [`is_row_empty`] /
/// [`is_column_empty`] predicates for standard behaviour.
#[must_use]
pub fn collect_lines(
    img: &GrayImage,
    primary_dim: u32,
    secondary_dim: u32,
    is_empty_fn: impl Fn(&GrayImage, u32, u32) -> bool,
) -> Vec<LineInfo> {
    let flags: Vec<bool> = (0..primary_dim)
        .map(|line| is_empty_fn(img, line, secondary_dim))
        .collect();
    run_length_encode(&flags)
}

/// Computes the average [`LineInfo::length`] of a set of runs (0 when empty).
#[must_use]
pub fn calculate_average_line_size(lines: &[LineInfo]) -> u32 {
    let Some((first, rest)) = lines.split_first() else {
        return 0;
    };
    let total: u32 = rest
        .iter()
        .fold(first.length, |acc, line| acc + line.length);
    let average = total as usize / lines.len();
    u32::try_from(average).unwrap_or(u32::MAX)
}

/// Merges runs that are smaller than `threshold` into their neighbour.
///
/// Runs are merged left-to-right: a run shorter than the threshold is glued to
/// the run currently being built. The merged runs always cover the same total
/// extent as the input.
///
/// # Examples
///
/// ```
/// use grider::detection::merge_small_lines;
/// use grider::{LineInfo, LineKind};
///
/// let lines = vec![
///     LineInfo::new(0, 5, LineKind::Empty),
///     LineInfo::new(5, 3, LineKind::Empty),
///     LineInfo::new(8, 10, LineKind::Full),
/// ];
/// let merged = merge_small_lines(lines, 4);
/// assert_eq!(merged.len(), 2);
/// ```
#[must_use]
pub fn merge_small_lines(lines: Vec<LineInfo>, threshold: u32) -> SmallVecLine<LineInfo> {
    let mut merged_lines = SmallVecLine::new();
    let mut lines = lines.into_iter();
    let Some(first) = lines.next() else {
        return merged_lines;
    };
    let mut current_start = first.start;
    let mut current_length = first.length;
    let mut current_kind = first.kind;

    for line in lines {
        if current_length < threshold || line.length < threshold {
            // Merge with the current run if either is smaller than the threshold.
            current_length += line.length;
        } else {
            merged_lines.push(LineInfo::new(current_start, current_length, current_kind));
            current_start = line.start;
            current_length = line.length;
            current_kind = line.kind;
        }
    }

    merged_lines.push(LineInfo::new(current_start, current_length, current_kind));
    merged_lines
}

/// Processes one dimension (rows or columns) of a binarized image into grid lines.
///
/// # Arguments
/// * `img` - The binarized grayscale image.
/// * `primary_dim` - Number of scan lines on the processed axis.
/// * `secondary_dim` - Length of each scan line on the other axis.
/// * `merge_threshold_ratio` - Ratio of the average run size under which runs merge.
/// * `is_empty_fn` - Predicate deciding whether a scan line is background.
/// * `parallel` - Scan lines in parallel across threads.
///
/// # Errors
///
/// Returns [`GridError::InvalidDimensions`] when either dimension is zero.
pub fn process_dimension<T: LineTrait + Send>(
    img: &GrayImage,
    primary_dim: u32,
    secondary_dim: u32,
    merge_threshold_ratio: f32,
    is_empty_fn: impl Fn(&GrayImage, u32, u32) -> bool + Sync,
    parallel: bool,
) -> Result<SmallVecLine<T>, GridError> {
    debug!(
        "Processing dimension with primary_dim={primary_dim}, secondary_dim={secondary_dim}, parallel={parallel}"
    );
    if primary_dim == 0 || secondary_dim == 0 {
        return Err(GridError::InvalidDimensions {
            width: secondary_dim,
            height: primary_dim,
        });
    }
    Ok(process_dimension_unchecked(
        img,
        primary_dim,
        secondary_dim,
        merge_threshold_ratio,
        is_empty_fn,
        parallel,
    ))
}

/// Infallible core of [`process_dimension`] for callers that have already
/// validated the dimensions (the image extents are always non-zero there).
///
/// # Arguments
/// * `img` - The binarized grayscale image.
/// * `primary_dim` - Number of scan lines on the processed axis.
/// * `secondary_dim` - Length of each scan line on the other axis.
/// * `merge_threshold_ratio` - Ratio of the average run size under which runs merge.
/// * `is_empty_fn` - Predicate deciding whether a scan line is background.
/// * `parallel` - Scan lines in parallel across threads.
pub(crate) fn process_dimension_unchecked<T: LineTrait + Send>(
    img: &GrayImage,
    primary_dim: u32,
    secondary_dim: u32,
    merge_threshold_ratio: f32,
    is_empty_fn: impl Fn(&GrayImage, u32, u32) -> bool + Sync,
    parallel: bool,
) -> SmallVecLine<T> {
    let flags: Vec<bool> = if parallel {
        (0..primary_dim)
            .into_par_iter()
            .map(|line| is_empty_fn(img, line, secondary_dim))
            .collect()
    } else {
        (0..primary_dim)
            .map(|line| is_empty_fn(img, line, secondary_dim))
            .collect()
    };

    let lines = run_length_encode(&flags);
    let average_size = calculate_average_line_size(&lines);
    let merge_threshold = merge_threshold_for(average_size, merge_threshold_ratio);
    let merged = merge_small_lines(lines, merge_threshold);

    merged.into_iter().map(T::new).collect()
}

/// Computes the merge threshold from an average run size and a ratio.
///
/// Mirrors the legacy saturating cast semantics: a NaN or negative product
/// yields `0` (nothing merges), an out-of-range product saturates to
/// `u32::MAX`. The computation happens in `f64` (lossless for `u32` inputs)
/// and the integer conversion is performed through string parsing because
/// `f64 -> u32` casts would silently truncate.
#[must_use]
fn merge_threshold_for(average_size: u32, ratio: f32) -> u32 {
    let scaled = f64::from(average_size) * f64::from(ratio);
    if scaled.is_nan() || scaled <= 0.0 {
        return 0;
    }
    if scaled >= f64::from(u32::MAX) {
        return u32::MAX;
    }
    scaled
        .round()
        .to_string()
        .parse::<u32>()
        .unwrap_or(u32::MAX)
}

/// A `SmallVec` backed by an inline buffer of [`DEFAULT_SMALLVEC_SIZE`] entries.
///
/// Typical grids hold a handful of rows/columns, so they avoid heap
/// allocations entirely.
pub const DEFAULT_SMALLVEC_SIZE: usize = 32;

/// Type alias for `SmallVec` with an optimized stack-allocated buffer size.
pub type SmallVecLine<T> = SmallVec<[T; DEFAULT_SMALLVEC_SIZE]>;

/// Minimum block size accepted by the pipeline (re-exported for convenience).
pub const MIN_BLOCK_SIZE: u32 = MIN_THRESHOLD_BLOCK_SIZE;

#[cfg(test)]
mod tests {
    use super::{
        calculate_average_line_size, is_column_empty, is_row_empty, merge_small_lines,
        merge_threshold_for, process_dimension, run_length_encode,
    };
    use crate::lines::{LineInfo, LineKind, LineTrait, Row};
    use image::GrayImage;

    fn image_from(width: u32, height: u32, rows: &[&[u8]]) -> GrayImage {
        let flat: Vec<u8> = rows.iter().flat_map(|row| row.iter().copied()).collect();
        GrayImage::from_raw(width, height, flat).expect("valid raw buffer")
    }

    #[test]
    fn is_row_empty_scans_only_requested_width() {
        let img = image_from(4, 1, &[&[255, 255, 0, 0]]);
        assert!(is_row_empty(&img, 0, 2));
        assert!(!is_row_empty(&img, 0, 3));
    }

    #[test]
    fn is_column_empty_scans_only_requested_height() {
        let img = image_from(2, 3, &[&[0, 255], &[0, 255], &[0, 0]]);
        assert!(!is_column_empty(&img, 0, 3));
        assert!(is_column_empty(&img, 1, 2));
        assert!(!is_column_empty(&img, 1, 3));
    }

    #[test]
    fn run_length_encode_handles_empty_input() {
        assert_eq!(run_length_encode(&[]), Vec::<LineInfo>::new());
    }

    #[test]
    fn run_length_encode_merges_consecutive_flags() {
        let runs = run_length_encode(&[true, true, false, true, true, true]);
        assert_eq!(
            runs,
            vec![
                LineInfo::new(0, 2, LineKind::Empty),
                LineInfo::new(2, 1, LineKind::Full),
                LineInfo::new(3, 3, LineKind::Empty),
            ]
        );
    }

    #[test]
    fn average_line_size_matches_manual_sum() {
        let lines = vec![
            LineInfo::new(0, 4, LineKind::Empty),
            LineInfo::new(4, 6, LineKind::Full),
        ];
        assert_eq!(calculate_average_line_size(&lines), 5);
        assert_eq!(calculate_average_line_size(&[]), 0);
    }

    #[test]
    fn merge_threshold_follows_ratio() {
        assert_eq!(merge_threshold_for(10, 0.8), 8);
        assert_eq!(merge_threshold_for(10, 0.0), 0);
        assert_eq!(merge_threshold_for(10, f32::NAN), 0);
        assert_eq!(merge_threshold_for(10, -1.0), 0);
        assert_eq!(merge_threshold_for(u32::MAX / 2 + 1, 2.0), u32::MAX);
    }

    #[test]
    fn merge_small_lines_with_single_run() {
        let merged = merge_small_lines(vec![LineInfo::new(0, 3, LineKind::Full)], 5);
        assert_eq!(merged.as_slice(), &[LineInfo::new(0, 3, LineKind::Full)]);
    }

    #[test]
    fn process_dimension_partitions_the_dimension() {
        let img = image_from(
            4,
            6,
            &[
                &[255, 255, 255, 255],
                &[0, 0, 0, 0],
                &[0, 0, 0, 0],
                &[255, 255, 255, 255],
                &[0, 0, 0, 0],
                &[255, 255, 255, 255],
            ],
        );
        let rows =
            process_dimension::<Row>(&img, 6, 4, 0.8, is_row_empty, false).expect("valid dims");
        assert_eq!(
            rows.as_slice(),
            &[
                Row::new(LineInfo::new(0, 1, LineKind::Empty)),
                Row::new(LineInfo::new(1, 2, LineKind::Full)),
                Row::new(LineInfo::new(3, 1, LineKind::Empty)),
                Row::new(LineInfo::new(4, 1, LineKind::Full)),
                Row::new(LineInfo::new(5, 1, LineKind::Empty)),
            ]
        );
    }

    #[test]
    fn process_dimension_parallel_matches_sequential() {
        let img = image_from(
            5,
            9,
            &[
                &[255, 255, 255, 255, 255],
                &[0, 0, 0, 0, 0],
                &[0, 0, 0, 0, 0],
                &[0, 0, 0, 0, 0],
                &[255, 255, 255, 255, 255],
                &[0, 0, 0, 0, 0],
                &[0, 0, 0, 0, 0],
                &[255, 255, 255, 255, 255],
                &[255, 255, 255, 255, 255],
            ],
        );
        let sequential =
            process_dimension::<Row>(&img, 9, 5, 0.5, is_row_empty, false).expect("valid dims");
        let parallel =
            process_dimension::<Row>(&img, 9, 5, 0.5, is_row_empty, true).expect("valid dims");
        assert_eq!(sequential, parallel);
    }

    #[test]
    fn process_dimension_rejects_zero_dimensions() {
        let img = GrayImage::new(2, 2);
        let result = process_dimension::<Row>(&img, 0, 2, 0.8, is_row_empty, false);
        assert_eq!(
            result,
            Err(crate::error::GridError::InvalidDimensions {
                width: 2,
                height: 0
            })
        );
    }
}
