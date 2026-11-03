//! Binarization algorithms used by the grid extraction pipeline.
//!
//! Every algorithm here runs in `O(width * height)` regardless of the window
//! radius by combining two classic techniques:
//!
//! 1. **Integral images** (Bradley & Roth, 2005) turn the local mean of a
//!    `(2r+1)²` window into four array lookups.
//! 2. **A streaming sliding-window ring** keeps only the `2r+2` integral rows
//!    that the current output row can ever touch, so peak memory stays at
//!    `O(width * radius)` instead of materializing a whole integral image.
//!
//! The adaptive mean threshold reproduces the semantics of
//! `imageproc::contrast::adaptive_threshold` bit-for-bit (verified by property
//! tests), while the Sauvola threshold implements Sauvola & Pietikäinen (2000)
//! through the integral-image formulation of Shafait, Beusekom, Keysers and
//! Tholey (2008) — the state of the art for degraded document images.
//!
//! # Floating point exactness
//!
//! Integral rows are stored as `f64`. Every stored value is an integer below
//! `2^53` (any image whose total brightness fits a `u53`, i.e. any image that
//! fits in addressable memory), so all additions, subtractions and comparisons
//! on them are exact. The adaptive mean comparison is therefore an exact
//! integer comparison:
//!
//! ```text
//! pixel >= floor(sum / area) - delta   ⇔   sum < (pixel + delta + 1) * area
//! ```
//!
//! which removes the per-pixel integer division and lets LLVM vectorize the
//! threshold loop.

use image::GrayImage;
use rayon::iter::{IndexedParallelIterator, ParallelIterator};
use rayon::slice::ParallelSliceMut;

use crate::config::{Binarization, GridConfig, SAUVOLA_DYNAMIC_RANGE};

/// Minimum radius accepted by the threshold functions.
///
/// Mirrors `imageproc::contrast::adaptive_threshold`, which asserts
/// `block_radius > 0`.
pub const MIN_BLOCK_RADIUS: u32 = 1;

/// Applies the binarization method described by `config` to a grayscale image.
///
/// This is the entry point used by [`crate::Grid::try_from_image_with_config`]:
/// it dispatches on [`GridConfig::binarization`] and feeds it the block size,
/// delta and parallelism settings of the configuration.
///
/// # Panics
///
/// Panics when `config.threshold_block_size` is `0`; the constructors and
/// builders of [`GridConfig`] clamp it to at least
/// [`crate::MIN_THRESHOLD_BLOCK_SIZE`], so only hand-built configurations can
/// trigger this.
///
/// # Examples
///
/// ```
/// use grider::config::{Binarization, GridConfig};
/// use grider::binarize::binarize;
/// use image::{GrayImage, Luma};
///
/// let img = GrayImage::from_pixel(4, 4, Luma([255]));
/// let config = GridConfig::default();
/// let binarized = binarize(&img, &config);
/// assert!(binarized.as_raw().iter().all(|&p| p == 255));
/// ```
#[must_use]
pub fn binarize(img: &GrayImage, config: &GridConfig) -> GrayImage {
    match config.binarization {
        Binarization::AdaptiveMean => adaptive_mean_threshold(
            img,
            config.threshold_block_size,
            config.threshold_delta,
            config.enable_parallel,
        ),
        Binarization::Sauvola { k } => {
            sauvola_threshold(img, config.threshold_block_size, k, config.enable_parallel)
        }
    }
}

/// Binarizes an image with the adaptive mean threshold of Bradley & Roth
/// (2005).
///
/// A pixel becomes white (`255`, background) when it is at least as bright as
/// the mean of its `(2 * block_radius + 1)²` window minus `delta`, black
/// (`0`, content) otherwise. The output matches
/// `imageproc::contrast::adaptive_threshold(image, block_radius, delta)`
/// pixel-for-pixel.
///
/// # Panics
///
/// Panics when `block_radius` is `0`.
///
/// # Examples
///
/// ```
/// use grider::binarize::adaptive_mean_threshold;
/// use image::{GrayImage, Luma};
///
/// let img = GrayImage::from_raw(3, 1, vec![255, 0, 255]).unwrap();
/// let binarized = adaptive_mean_threshold(&img, 1, 0, false);
/// assert_eq!(binarized.as_raw(), &[255, 0, 255]);
/// ```
#[must_use]
pub fn adaptive_mean_threshold(
    img: &GrayImage,
    block_radius: u32,
    delta: i32,
    parallel: bool,
) -> GrayImage {
    assert!(
        block_radius >= MIN_BLOCK_RADIUS,
        "block_radius must be at least {MIN_BLOCK_RADIUS}"
    );
    let width = img.width() as usize;
    let height = img.height() as usize;
    let mut out = vec![0_u8; width * height];
    if width > 0 && height > 0 {
        let radius = block_radius as usize;
        let rows_per_band = band_plan(width, height, radius, parallel);
        if rows_per_band >= height {
            threshold_band_mean(
                img.as_raw(),
                &mut out,
                width,
                height,
                0..height,
                radius,
                delta,
            );
        } else {
            out.par_chunks_mut(width * rows_per_band)
                .enumerate()
                .for_each(|(band, chunk)| {
                    let start = band * rows_per_band;
                    let end = start + chunk.len() / width;
                    threshold_band_mean(
                        img.as_raw(),
                        chunk,
                        width,
                        height,
                        start..end,
                        radius,
                        delta,
                    );
                });
        }
    }
    GrayImage::from_raw(img.width(), img.height(), out)
        .expect("output buffer matches the image dimensions")
}

/// Binarizes an image with the Sauvola & Pietikäinen (2000) threshold.
///
/// The local threshold is `mean * (1 + k * (std / R - 1))` with `R = 128`,
/// computed from the mean and standard deviation of the
/// `(2 * block_radius + 1)²` window through two streaming integral rows
/// (Shafait et al., 2008). Pixels *strictly brighter* than the threshold become
/// white (`255`, background), the rest black (`0`, content). This makes
/// Sauvola robust against uneven illumination and low contrast, at the cost of
/// being roughly two times slower than the adaptive mean threshold.
///
/// The parameter `k` controls sensitivity to local contrast (see
/// [`crate::SAUVOLA_DEFAULT_K`]); [`GridConfig::validate`] rejects non-finite values.
///
/// # Panics
///
/// Panics when `block_radius` is `0`.
///
/// # Examples
///
/// ```
/// use grider::binarize::sauvola_threshold;
/// use grider::SAUVOLA_DEFAULT_K;
/// use image::{GrayImage, Luma};
///
/// let img = GrayImage::from_raw(3, 1, vec![255, 0, 255]).unwrap();
/// let binarized = sauvola_threshold(&img, 1, SAUVOLA_DEFAULT_K, false);
/// assert_eq!(binarized.as_raw(), &[255, 0, 255]);
/// ```
#[must_use]
pub fn sauvola_threshold(img: &GrayImage, block_radius: u32, k: f32, parallel: bool) -> GrayImage {
    assert!(
        block_radius >= MIN_BLOCK_RADIUS,
        "block_radius must be at least {MIN_BLOCK_RADIUS}"
    );
    let width = img.width() as usize;
    let height = img.height() as usize;
    let mut out = vec![0_u8; width * height];
    if width > 0 && height > 0 {
        let radius = block_radius as usize;
        let rows_per_band = band_plan(width, height, radius, parallel);
        if rows_per_band >= height {
            threshold_band_sauvola(
                img.as_raw(),
                &mut out,
                width,
                height,
                0..height,
                radius,
                f64::from(k),
            );
        } else {
            out.par_chunks_mut(width * rows_per_band)
                .enumerate()
                .for_each(|(band, chunk)| {
                    let start = band * rows_per_band;
                    let end = start + chunk.len() / width;
                    threshold_band_sauvola(
                        img.as_raw(),
                        chunk,
                        width,
                        height,
                        start..end,
                        radius,
                        f64::from(k),
                    );
                });
        }
    }
    GrayImage::from_raw(img.width(), img.height(), out)
        .expect("output buffer matches the image dimensions")
}

/// Number of threads available to the process, cached after the first call.
pub(crate) fn thread_count() -> usize {
    static THREADS: std::sync::OnceLock<usize> = std::sync::OnceLock::new();
    *THREADS
        .get_or_init(|| std::thread::available_parallelism().map_or(1, std::num::NonZeroUsize::get))
}

/// Returns `true` when the parallel path can pay for its coordination
/// overhead.
///
/// Each thread must get at least `pixels_per_thread` pixels of work; below
/// that the rayon scheduling and band warm-up cost more than they save, and
/// the calling thread does the whole job on its own.
pub(crate) fn parallel_worthwhile(pixels: usize, pixels_per_thread: usize) -> bool {
    let threads = thread_count();
    threads > 1 && pixels / threads >= pixels_per_thread
}

/// Decides how many rows each parallel band processes.
///
/// Bands must stay large enough to amortize the ring warm-up
/// (`radius + 1` rows), hence the `4 * (radius + 1)` lower bound per band.
/// With `parallel = false` (or images too small to benefit) the whole height
/// is one band and the work runs on the calling thread.
fn band_plan(width: usize, height: usize, radius: usize, parallel: bool) -> usize {
    if !parallel || !parallel_worthwhile(width * height, 262_144) {
        return height;
    }
    let cores = thread_count();
    let max_bands = (height / (4 * (radius + 1))).max(1);
    height.div_ceil(cores.min(max_bands))
}

/// Streaming sliding-window integral image over one band of rows.
///
/// The ring holds the last `cap = min(2 * radius + 2, height + 1)` integral
/// rows — exactly the window of rows a single output row can reference — so
/// each push is `O(width)` and memory stays `O(width * radius)`.
///
/// Integral row `local` (a local index starting at the band's first reachable
/// row) holds the cumulative brightness of all pixel rows above
/// `base_row + local`, with entry `0` pinned to `0.0`.
struct StreamingIntegral {
    /// Ring of integral brightness rows, each of length `width + 1`.
    sums: Vec<Vec<f64>>,
    /// Optional ring of integral squared-brightness rows (Sauvola only).
    sums_sq: Option<Vec<Vec<f64>>>,
    /// Scratch rows used to build the next integral row without aliasing.
    scratch: Vec<f64>,
    scratch_sq: Vec<f64>,
    /// Width of the image; integral rows hold `width + 1` entries.
    width: usize,
    /// Ring capacity: `min(2 * radius + 2, height + 1)` rows.
    cap: usize,
    /// Absolute pixel row covered by local row `0`.
    base_row: usize,
    /// Number of integral rows already pushed (local indices `0..pushed`).
    pushed: usize,
}

impl StreamingIntegral {
    /// Creates an empty ring for a band whose reachable rows start at
    /// `base_row`.
    fn new(
        width: usize,
        radius: usize,
        height: usize,
        with_squares: bool,
        base_row: usize,
    ) -> Self {
        let cap = (2 * radius + 2).min(height + 1);
        Self {
            sums: (0..cap).map(|_| vec![0.0; width + 1]).collect(),
            sums_sq: with_squares.then(|| (0..cap).map(|_| vec![0.0; width + 1]).collect()),
            scratch: vec![0.0; width + 1],
            scratch_sq: vec![0.0; width + 1],
            width,
            cap,
            base_row,
            pushed: 0,
        }
    }

    /// Pushes integral rows until local row `target` is available.
    ///
    /// `raw` is the full image buffer; the row read for local row `local > 0`
    /// is the pixel row `base_row + local - 1`.
    fn push_until(&mut self, target: usize, raw: &[u8]) {
        while self.pushed <= target {
            let local = self.pushed;
            let slot = local % self.cap;
            if local == 0 {
                self.sums[slot].fill(0.0);
                if let Some(sq) = &mut self.sums_sq {
                    sq[slot].fill(0.0);
                }
            } else {
                let prev_slot = (local - 1) % self.cap;
                let pixels = &raw[(self.base_row + local - 1) * self.width..][..self.width];
                let mut sum = 0.0_f64;
                let mut sum_sq = 0.0_f64;
                for (x, &pixel) in pixels.iter().enumerate() {
                    let value = f64::from(pixel);
                    sum += value;
                    sum_sq += value * value;
                    self.scratch[x + 1] = self.sums[prev_slot][x + 1] + sum;
                    if let Some(sq) = &self.sums_sq {
                        self.scratch_sq[x + 1] = sq[prev_slot][x + 1] + sum_sq;
                    }
                }
                // The swap hands the stale ring row back as the next scratch
                // buffer: entry 0 is always 0.0 and entries 1.. are rewritten
                // by the next push, so no reallocation ever happens.
                std::mem::swap(&mut self.sums[slot], &mut self.scratch);
                if let Some(sq) = &mut self.sums_sq {
                    std::mem::swap(&mut sq[slot], &mut self.scratch_sq);
                }
            }
            self.pushed += 1;
        }
    }

    /// Borrows the integral row with local index `local`.
    ///
    /// The caller must have pushed `local` and must only query rows within
    /// `cap - 1` of the last pushed row (guaranteed by the ring sizing).
    fn row(&self, local: usize) -> &[f64] {
        &self.sums[local % self.cap]
    }

    /// Borrows the squared-value integral row with local index `local`.
    fn row_sq(&self, local: usize) -> &[f64] {
        &self
            .sums_sq
            .as_ref()
            .expect("squared rings exist when the ring is built with_squares")[local % self.cap]
    }
}

/// Brightness sum of the window `[x_low, x_high] × [y_low, y_high]` from two
/// integral rows (`top` at `y_low`, `bottom` at `y_high + 1`).
#[inline]
fn window_sum(top: &[f64], bottom: &[f64], x_low: usize, x_high: usize) -> f64 {
    bottom[x_high + 1] - bottom[x_low] - top[x_high + 1] + top[x_low]
}

/// Thresholds one output row with the adaptive mean rule.
///
/// The row is split into left edge, interior and right edge so the interior
/// loop keeps its window area constant and vectorizes.
fn threshold_row_mean(
    src: &[u8],
    out: &mut [u8],
    top: &[f64],
    bottom: &[f64],
    height_extent: u32,
    radius: usize,
    offset: f64,
) {
    let width = src.len();
    if width > 2 * radius {
        for x in 0..radius {
            let extent = u32::try_from(x + radius + 1).unwrap_or(u32::MAX);
            let area = f64::from(height_extent) * f64::from(extent);
            let sum = window_sum(top, bottom, 0, x + radius);
            out[x] = mean_white(src[x], sum, area, offset);
        }
        let area =
            f64::from(height_extent) * f64::from(u32::try_from(2 * radius + 1).unwrap_or(u32::MAX));
        for x in radius..width - radius {
            let sum = window_sum(top, bottom, x - radius, x + radius);
            out[x] = mean_white(src[x], sum, area, offset);
        }
        for x in width - radius..width {
            let extent = u32::try_from(width - x + radius).unwrap_or(u32::MAX);
            let area = f64::from(height_extent) * f64::from(extent);
            let sum = window_sum(top, bottom, x - radius, width - 1);
            out[x] = mean_white(src[x], sum, area, offset);
        }
    } else {
        for x in 0..width {
            let x_low = x.saturating_sub(radius);
            let x_high = (x + radius).min(width - 1);
            let extent = u32::try_from(x_high - x_low + 1).unwrap_or(u32::MAX);
            let area = f64::from(height_extent) * f64::from(extent);
            let sum = window_sum(top, bottom, x_low, x_high);
            out[x] = mean_white(src[x], sum, area, offset);
        }
    }
}

/// Adaptive mean decision for one pixel.
///
/// `sum` and `area` are exact integers stored in `f64`, so
/// `pixel >= floor(sum / area) - delta` is decided exactly by
/// `sum < (pixel + delta + 1) * area`. The three integer regimes fold into
/// one branchless comparison by saturating the offset `t` to `[0, 256]`:
/// `t = 0` makes the right side `0` (never white), `t = 256` makes it
/// `256 * area` which exceeds any window sum (`sum <= 255 * area`). Every
/// value stays an exact integer in `f64`, so the comparison is exact and the
/// whole pipeline vectorizes — no per-pixel division, no branches.
#[inline]
fn mean_white(pixel: u8, sum: f64, area: f64, offset: f64) -> u8 {
    let t = (f64::from(pixel) + offset).clamp(0.0, 256.0);
    if sum < t * area { 255 } else { 0 }
}

/// The four integral rows one output row reads its window sums from.
struct WindowRows<'a> {
    /// Integral brightness row at `y_low`.
    top: &'a [f64],
    /// Integral brightness row at `y_high + 1`.
    bottom: &'a [f64],
    /// Integral squared-brightness row at `y_low`.
    top_sq: &'a [f64],
    /// Integral squared-brightness row at `y_high + 1`.
    bottom_sq: &'a [f64],
}

/// Thresholds one output row with the Sauvola rule (same three-zone shape).
fn threshold_row_sauvola(
    src: &[u8],
    out: &mut [u8],
    rows: &WindowRows<'_>,
    height_extent: u32,
    radius: usize,
    k: f64,
) {
    let WindowRows {
        top,
        bottom,
        top_sq,
        bottom_sq,
    } = *rows;
    let width = src.len();
    if width > 2 * radius {
        for x in 0..radius {
            let extent = u32::try_from(x + radius + 1).unwrap_or(u32::MAX);
            out[x] = sauvola_white(
                src[x],
                window_sum(top, bottom, 0, x + radius),
                window_sum(top_sq, bottom_sq, 0, x + radius),
                f64::from(height_extent) * f64::from(extent),
                k,
            );
        }
        let area =
            f64::from(height_extent) * f64::from(u32::try_from(2 * radius + 1).unwrap_or(u32::MAX));
        for x in radius..width - radius {
            out[x] = sauvola_white(
                src[x],
                window_sum(top, bottom, x - radius, x + radius),
                window_sum(top_sq, bottom_sq, x - radius, x + radius),
                area,
                k,
            );
        }
        for x in width - radius..width {
            let extent = u32::try_from(width - x + radius).unwrap_or(u32::MAX);
            out[x] = sauvola_white(
                src[x],
                window_sum(top, bottom, x - radius, width - 1),
                window_sum(top_sq, bottom_sq, x - radius, width - 1),
                f64::from(height_extent) * f64::from(extent),
                k,
            );
        }
    } else {
        for x in 0..width {
            let x_low = x.saturating_sub(radius);
            let x_high = (x + radius).min(width - 1);
            let extent = u32::try_from(x_high - x_low + 1).unwrap_or(u32::MAX);
            out[x] = sauvola_white(
                src[x],
                window_sum(top, bottom, x_low, x_high),
                window_sum(top_sq, bottom_sq, x_low, x_high),
                f64::from(height_extent) * f64::from(extent),
                k,
            );
        }
    }
}

/// Sauvola decision for one pixel: white when the pixel is strictly brighter
/// than `mean * (1 + k * (std / R - 1))`.
#[inline]
fn sauvola_white(pixel: u8, sum: f64, sum_sq: f64, area: f64, k: f64) -> u8 {
    let mean = sum / area;
    let variance = (sum_sq / area - mean * mean).max(0.0);
    let std = variance.sqrt();
    let threshold = mean * (1.0 + k * (std / SAUVOLA_DYNAMIC_RANGE - 1.0));
    if f64::from(pixel) > threshold { 255 } else { 0 }
}

/// Thresholds the rows `rows` of `raw` into `out` with the adaptive mean rule.
///
/// `out` must hold exactly `rows.len() * width` bytes. The band builds its own
/// ring starting `radius` rows above `rows.start`, reading only the rows it is
/// allowed to touch.
fn threshold_band_mean(
    raw: &[u8],
    out: &mut [u8],
    width: usize,
    height: usize,
    rows: std::ops::Range<usize>,
    radius: usize,
    delta: i32,
) {
    // `offset = delta + 1` in f64: exact for the whole i32 delta range, and
    // the per-pixel `t = pixel + offset` stays an exact integer.
    let offset = f64::from(delta) + 1.0;
    let base_row = rows.start.saturating_sub(radius);
    let mut ring = StreamingIntegral::new(width, radius, height, false, base_row);
    for (index, y) in rows.clone().enumerate() {
        let y_low = y.saturating_sub(radius);
        let y_high = (y + radius).min(height - 1);
        let bottom_row = (y + radius + 1).min(height);
        ring.push_until(bottom_row - base_row, raw);
        let top = ring.row(y_low - base_row);
        let bottom = ring.row(bottom_row - base_row);
        let height_extent = u32::try_from(y_high - y_low + 1).unwrap_or(u32::MAX);
        let src = &raw[y * width..(y + 1) * width];
        let out_row = &mut out[index * width..(index + 1) * width];
        threshold_row_mean(src, out_row, top, bottom, height_extent, radius, offset);
    }
}

/// Thresholds the rows `rows` of `raw` into `out` with the Sauvola rule.
fn threshold_band_sauvola(
    raw: &[u8],
    out: &mut [u8],
    width: usize,
    height: usize,
    rows: std::ops::Range<usize>,
    radius: usize,
    k: f64,
) {
    let base_row = rows.start.saturating_sub(radius);
    let mut ring = StreamingIntegral::new(width, radius, height, true, base_row);
    for (index, y) in rows.clone().enumerate() {
        let y_low = y.saturating_sub(radius);
        let y_high = (y + radius).min(height - 1);
        let bottom_row = (y + radius + 1).min(height);
        ring.push_until(bottom_row - base_row, raw);
        let top = ring.row(y_low - base_row);
        let bottom = ring.row(bottom_row - base_row);
        let top_sq = ring.row_sq(y_low - base_row);
        let bottom_sq = ring.row_sq(bottom_row - base_row);
        let height_extent = u32::try_from(y_high - y_low + 1).unwrap_or(u32::MAX);
        let src = &raw[y * width..(y + 1) * width];
        let out_row = &mut out[index * width..(index + 1) * width];
        threshold_row_sauvola(
            src,
            out_row,
            &WindowRows {
                top,
                bottom,
                top_sq,
                bottom_sq,
            },
            height_extent,
            radius,
            k,
        );
    }
}

#[cfg(test)]
mod tests {
    use super::{SAUVOLA_DYNAMIC_RANGE, adaptive_mean_threshold, binarize, sauvola_threshold};
    use crate::config::{Binarization, GridConfig, SAUVOLA_DEFAULT_K};
    use image::{GrayImage, Luma};

    fn image_from(width: u32, height: u32, pixels: &[u8]) -> GrayImage {
        GrayImage::from_raw(width, height, pixels.to_vec()).expect("valid raw buffer")
    }

    #[test]
    fn adaptive_mean_matches_imageproc_on_a_structured_case() {
        // 5x4 with a vertical black stripe and a horizontal black stripe.
        let mut pixels = vec![255_u8; 20];
        for y in 0..4 {
            pixels[y * 5 + 2] = 0;
        }
        for x in 0..5 {
            pixels[3 * 5 + x] = 0;
        }
        let img = image_from(5, 4, &pixels);
        for delta in [-40_i32, -3, 0, 7, 40] {
            for radius in [1_u32, 2, 5] {
                let mine = adaptive_mean_threshold(&img, radius, delta, false);
                let reference = imageproc::contrast::adaptive_threshold(&img, radius, delta);
                assert_eq!(
                    mine.as_raw(),
                    reference.as_raw(),
                    "radius {radius}, delta {delta}"
                );
            }
        }
    }

    #[test]
    fn adaptive_mean_extreme_deltas_saturate_exactly() {
        // delta >= 255: every pixel passes (t >= 256 branch).
        let img = image_from(3, 3, &[0, 128, 255, 0, 128, 255, 0, 128, 255]);
        let all_white = adaptive_mean_threshold(&img, 1, 255, false);
        assert!(all_white.as_raw().iter().all(|&p| p == 255));
        // delta <= -256: no pixel passes (t <= 0 branch).
        let all_black = adaptive_mean_threshold(&img, 1, -256, false);
        assert!(all_black.as_raw().iter().all(|&p| p == 0));
        // Compare both against the imageproc reference for full confidence.
        assert_eq!(
            all_white.as_raw(),
            imageproc::contrast::adaptive_threshold(&img, 1, 255).as_raw()
        );
        assert_eq!(
            all_black.as_raw(),
            imageproc::contrast::adaptive_threshold(&img, 1, -256).as_raw()
        );
    }

    #[test]
    fn adaptive_mean_thresholds_a_flat_image_deterministically() {
        // Uniform 128 image: mean = 128, so pixel >= mean - delta decides.
        let img = GrayImage::from_pixel(6, 6, Luma([128]));
        assert_eq!(
            adaptive_mean_threshold(&img, 2, 0, false).as_raw(),
            &[255; 36]
        );
        assert_eq!(
            adaptive_mean_threshold(&img, 2, -1, false).as_raw(),
            &[0; 36]
        );
    }

    #[test]
    fn adaptive_mean_handles_degenerate_dimensions() {
        let empty = GrayImage::new(0, 0);
        assert_eq!(
            adaptive_mean_threshold(&empty, 1, 0, false).into_raw(),
            Vec::<u8>::new()
        );
        let column = GrayImage::new(1, 5);
        assert_eq!(
            adaptive_mean_threshold(&column, 1, 0, true).as_raw(),
            &[255; 5]
        );
    }

    #[test]
    fn adaptive_mean_parallel_matches_sequential() {
        let mut pixels = Vec::new();
        for y in 0..23_u32 {
            for x in 0..31_u32 {
                let value = if (x * 7 + y * 13) % 3 == 0 { 0 } else { 255 };
                pixels.push(value);
            }
        }
        let img = image_from(31, 23, &pixels);
        for radius in [1_u32, 3, 16] {
            let sequential = adaptive_mean_threshold(&img, radius, 5, false);
            let parallel = adaptive_mean_threshold(&img, radius, 5, true);
            assert_eq!(sequential.as_raw(), parallel.as_raw(), "radius {radius}");
        }
    }

    #[test]
    #[should_panic(expected = "block_radius must be at least 1")]
    fn adaptive_mean_rejects_zero_radius() {
        let img = GrayImage::from_pixel(2, 2, Luma([255]));
        let _ = adaptive_mean_threshold(&img, 0, 0, false);
    }

    #[test]
    #[should_panic(expected = "block_radius must be at least 1")]
    fn sauvola_rejects_zero_radius() {
        let img = GrayImage::from_pixel(2, 2, Luma([255]));
        let _ = sauvola_threshold(&img, 0, SAUVOLA_DEFAULT_K, false);
    }

    /// Naive Sauvola reference: explicit window walk with the exact same
    /// floating point expression order as the streaming implementation.
    fn sauvola_reference(img: &GrayImage, radius: u32, k: f32) -> Vec<u8> {
        let (width, height) = img.dimensions();
        let raw = img.as_raw();
        let mut out = Vec::with_capacity((width * height) as usize);
        for y in 0..height {
            for x in 0..width {
                let y_low = y.saturating_sub(radius);
                let y_high = (y + radius).min(height - 1);
                let x_low = x.saturating_sub(radius);
                let x_high = (x + radius).min(width - 1);
                let mut sum = 0.0_f64;
                let mut sum_sq = 0.0_f64;
                for wy in y_low..=y_high {
                    for wx in x_low..=x_high {
                        let value = f64::from(raw[(wy * width + wx) as usize]);
                        sum += value;
                        sum_sq += value * value;
                    }
                }
                let area = f64::from(y_high - y_low + 1) * f64::from(x_high - x_low + 1);
                let mean = sum / area;
                let variance = (sum_sq / area - mean * mean).max(0.0);
                let std = variance.sqrt();
                let threshold = mean * (1.0 + f64::from(k) * (std / SAUVOLA_DYNAMIC_RANGE - 1.0));
                out.push(if f64::from(raw[(y * width + x) as usize]) > threshold {
                    255
                } else {
                    0
                });
            }
        }
        out
    }

    #[test]
    fn sauvola_matches_the_naive_reference_exactly() {
        // Small asymmetric images exercise every edge of the three zones.
        for (width, height) in [(1, 1), (1, 7), (9, 1), (4, 5), (13, 6), (17, 17)] {
            let mut pixels = Vec::with_capacity((width * height) as usize);
            for y in 0..height {
                for x in 0..width {
                    pixels.push(u8::try_from((x * 31 + y * 17) % 256).unwrap_or(200));
                }
            }
            let img = image_from(width, height, &pixels);
            for radius in [1_u32, 2, 5] {
                for k in [0.0_f32, 0.2, 0.5, -0.3] {
                    let mine = sauvola_threshold(&img, radius, k, false);
                    assert_eq!(
                        mine.as_raw(),
                        &sauvola_reference(&img, radius, k),
                        "size {width}x{height}, radius {radius}, k {k}"
                    );
                }
            }
        }
    }

    #[test]
    fn sauvola_keeps_uniform_background_white() {
        let img = GrayImage::from_pixel(9, 9, Luma([255]));
        let out = sauvola_threshold(&img, 2, SAUVOLA_DEFAULT_K, false);
        assert!(out.as_raw().iter().all(|&p| p == 255));
        let black = GrayImage::from_pixel(9, 9, Luma([0]));
        let out = sauvola_threshold(&black, 2, SAUVOLA_DEFAULT_K, false);
        assert!(out.as_raw().iter().all(|&p| p == 0));
    }

    #[test]
    fn sauvola_parallel_matches_sequential() {
        let mut pixels = Vec::new();
        for y in 0..19_u32 {
            for x in 0..29_u32 {
                pixels.push(u8::try_from((x * 11 + y * 7) % 256).unwrap_or(128));
            }
        }
        let img = image_from(29, 19, &pixels);
        for radius in [1_u32, 4] {
            let sequential = sauvola_threshold(&img, radius, 0.3, false);
            let parallel = sauvola_threshold(&img, radius, 0.3, true);
            assert_eq!(sequential.as_raw(), parallel.as_raw(), "radius {radius}");
        }
    }

    #[test]
    fn sauvola_handles_degenerate_dimensions() {
        let empty = GrayImage::new(0, 0);
        assert_eq!(
            sauvola_threshold(&empty, 1, 0.2, false).into_raw(),
            Vec::<u8>::new()
        );
    }

    #[test]
    fn binarize_dispatches_on_the_config() {
        let img = image_from(3, 1, &[255, 0, 255]);
        let mean_config = GridConfig::default();
        assert_eq!(
            binarize(&img, &mean_config).as_raw(),
            adaptive_mean_threshold(&img, mean_config.threshold_block_size, 0, true).as_raw()
        );
        let sauvola_config = GridConfig::builder().sauvola(0.2).build();
        assert_eq!(
            binarize(&img, &sauvola_config).as_raw(),
            sauvola_threshold(&img, sauvola_config.threshold_block_size, 0.2, true).as_raw()
        );
        // The enum must expose both variants through the config type.
        assert_eq!(mean_config.binarization, Binarization::AdaptiveMean);
        assert_eq!(
            sauvola_config.binarization,
            Binarization::Sauvola { k: 0.2 }
        );
    }

    #[test]
    fn sauvola_with_zero_k_compares_strictly_against_the_mean() {
        // k = 0 reduces the threshold to the local mean; a pixel exactly at
        // the mean is black for Sauvola (strict >) but white for the adaptive
        // mean (>=).
        let img = GrayImage::from_pixel(5, 5, Luma([128]));
        let sauvola = sauvola_threshold(&img, 2, 0.0, false);
        assert!(sauvola.as_raw().iter().all(|&p| p == 0));
        let mean = adaptive_mean_threshold(&img, 2, 0, false);
        assert!(mean.as_raw().iter().all(|&p| p == 255));
    }
}
