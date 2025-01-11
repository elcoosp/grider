use super::*;

/// Configuration for grid processing.
///
/// # Example
/// ```
/// use grider::GridConfig;
///
/// let config = GridConfig::default();
/// assert_eq!(config.threshold_block_size, 12);
/// assert_eq!(config.merge_threshold_ratio, 0.8);
/// assert_eq!(config.enable_parallel, true);
/// ```
#[derive(Debug, Clone)]
pub struct GridConfig {
    /// Block size for adaptive thresholding (default: 12)
    pub threshold_block_size: u32,
    /// Ratio for merging small lines (default: 0.8)
    pub merge_threshold_ratio: f32,
    /// Enable parallel processing (default: true)
    pub enable_parallel: bool,
}

impl GridConfig {
    /// Creates a new `GridConfig` with the specified parameters.
    ///
    /// # Example
    /// ```
    /// use grider::GridConfig;
    ///
    /// let config = GridConfig::new(15, 0.9, false);
    /// assert_eq!(config.threshold_block_size, 15);
    /// assert_eq!(config.merge_threshold_ratio, 0.9);
    /// assert_eq!(config.enable_parallel, false);
    /// ```
    pub fn new(
        threshold_block_size: u32,
        merge_threshold_ratio: f32,
        enable_parallel: bool,
    ) -> Self {
        Self {
            threshold_block_size: threshold_block_size.max(3), // Minimum block size
            merge_threshold_ratio,
            enable_parallel,
        }
    }
}

impl Default for GridConfig {
    fn default() -> Self {
        GridConfig::new(
            DEFAULT_THRESHOLD_BLOCK_SIZE,
            DEFAULT_MERGE_THRESHOLD_RATIO,
            true,
        )
    }
}
