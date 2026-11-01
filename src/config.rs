//! Configuration for the grid extraction pipeline.

use crate::error::GridError;

/// Default block size used by [`GridConfig::default`].
///
/// Determined through benchmarking typical use cases: it acts as the *block
/// radius* of the adaptive threshold, i.e. the comparison window is
/// `2 * threshold_block_size + 1` pixels wide.
pub const DEFAULT_THRESHOLD_BLOCK_SIZE: u32 = 12;

/// Default merge ratio used by [`GridConfig::default`].
pub const DEFAULT_MERGE_THRESHOLD_RATIO: f32 = 0.8;

/// Default threshold delta used by [`GridConfig::default`].
pub const DEFAULT_THRESHOLD_DELTA: i32 = 0;

/// Minimum accepted `threshold_block_size` (values below are clamped up to it).
pub const MIN_THRESHOLD_BLOCK_SIZE: u32 = 3;

/// Configuration for grid processing.
///
/// # Examples
///
/// ```
/// use grider::GridConfig;
///
/// let config = GridConfig::default();
/// assert_eq!(config.threshold_block_size, 12);
/// assert_eq!(config.threshold_delta, 0);
/// assert_eq!(config.merge_threshold_ratio, 0.8);
/// assert!(config.enable_parallel);
///
/// // Builder style
/// let config = GridConfig::builder()
///     .threshold_block_size(15)
///     .threshold_delta(4)
///     .merge_threshold_ratio(0.9)
///     .parallel(false)
///     .build();
/// assert_eq!(config.threshold_block_size, 15);
/// assert_eq!(config.threshold_delta, 4);
/// ```
#[derive(Debug, Clone, Copy, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct GridConfig {
    /// Block size for adaptive thresholding (default: 12).
    ///
    /// The value is used as the *block radius* of the adaptive threshold: the
    /// mean is computed over a `(2 * size + 1)` square window. Values below 3
    /// are clamped to 3 by the constructors and builders.
    pub threshold_block_size: u32,
    /// Offset subtracted from the local mean during adaptive thresholding
    /// (default: 0). Positive values make the threshold more permissive (more
    /// pixels count as background), negative values more strict.
    pub threshold_delta: i32,
    /// Ratio of the average line size below which neighbouring lines are
    /// merged (default: 0.8). Values are clamped to `[0.0, 1.0]`.
    pub merge_threshold_ratio: f32,
    /// Enable parallel processing (default: true).
    pub enable_parallel: bool,
}

impl GridConfig {
    /// Creates a new `GridConfig` with the specified parameters and a zero
    /// threshold delta.
    ///
    /// `threshold_block_size` is clamped to at least 3 and
    /// `merge_threshold_ratio` is clamped to `[0.0, 1.0]` (NaN becomes 0.8).
    ///
    /// # Examples
    ///
    /// ```
    /// use grider::GridConfig;
    ///
    /// let config = GridConfig::new(15, 0.9, false);
    /// assert_eq!(config.threshold_block_size, 15);
    /// assert_eq!(config.merge_threshold_ratio, 0.9);
    /// assert!(!config.enable_parallel);
    /// assert_eq!(config.threshold_delta, 0);
    /// ```
    #[must_use]
    pub fn new(
        threshold_block_size: u32,
        merge_threshold_ratio: f32,
        enable_parallel: bool,
    ) -> Self {
        Self {
            threshold_block_size: threshold_block_size.max(MIN_THRESHOLD_BLOCK_SIZE),
            threshold_delta: DEFAULT_THRESHOLD_DELTA,
            merge_threshold_ratio: Self::sanitize_ratio(merge_threshold_ratio),
            enable_parallel,
        }
    }

    /// Returns a [`GridConfigBuilder`] for incremental configuration.
    ///
    /// # Examples
    ///
    /// ```
    /// use grider::GridConfig;
    ///
    /// let config = GridConfig::builder().parallel(false).build();
    /// assert!(!config.enable_parallel);
    /// ```
    pub fn builder() -> GridConfigBuilder {
        GridConfigBuilder::new()
    }

    /// Validates the configuration values.
    ///
    /// Hand-constructed values (struct literal or deserialization) may be out
    /// of range; this reports the first problem found.
    ///
    /// # Errors
    ///
    /// Returns [`GridError::InvalidConfig`] when `threshold_block_size` is `0`
    /// or `merge_threshold_ratio` is not a finite value in `[0.0, 1.0]`.
    ///
    /// # Examples
    ///
    /// ```
    /// use grider::GridConfig;
    ///
    /// assert!(GridConfig::default().validate().is_ok());
    /// assert!(GridConfig { threshold_block_size: 0, ..GridConfig::default() }.validate().is_err());
    /// ```
    pub fn validate(&self) -> Result<(), GridError> {
        if self.threshold_block_size == 0 {
            return Err(GridError::invalid_config(
                "threshold_block_size must be greater than 0",
            ));
        }
        if !(self.merge_threshold_ratio.is_finite()
            && (0.0..=1.0).contains(&self.merge_threshold_ratio))
        {
            return Err(GridError::invalid_config(
                "merge_threshold_ratio must be a finite value in [0.0, 1.0]",
            ));
        }
        Ok(())
    }

    /// Clamps a ratio into `[0.0, 1.0]`, mapping NaN onto the default ratio.
    #[must_use]
    pub fn sanitize_ratio(ratio: f32) -> f32 {
        if ratio.is_nan() {
            return DEFAULT_MERGE_THRESHOLD_RATIO;
        }
        ratio.clamp(0.0, 1.0)
    }
}

impl Default for GridConfig {
    fn default() -> Self {
        Self {
            threshold_block_size: DEFAULT_THRESHOLD_BLOCK_SIZE,
            threshold_delta: DEFAULT_THRESHOLD_DELTA,
            merge_threshold_ratio: DEFAULT_MERGE_THRESHOLD_RATIO,
            enable_parallel: true,
        }
    }
}

/// Builder for [`GridConfig`].
///
/// # Examples
///
/// ```
/// use grider::GridConfig;
///
/// let config = GridConfig::builder()
///     .threshold_block_size(7)
///     .threshold_delta(-2)
///     .merge_threshold_ratio(0.5)
///     .parallel(true)
///     .build();
///
/// assert!(config.validate().is_ok());
/// ```
#[derive(Debug, Clone, Copy, PartialEq)]
#[must_use]
pub struct GridConfigBuilder {
    config: GridConfig,
}

impl GridConfigBuilder {
    /// Starts a builder pre-populated with [`GridConfig::default`] values.
    pub fn new() -> Self {
        Self {
            config: GridConfig::default(),
        }
    }

    /// Sets the adaptive threshold block size (clamped to at least 3).
    pub fn threshold_block_size(mut self, threshold_block_size: u32) -> Self {
        self.config.threshold_block_size = threshold_block_size.max(MIN_THRESHOLD_BLOCK_SIZE);
        self
    }

    /// Sets the adaptive threshold delta.
    pub fn threshold_delta(mut self, threshold_delta: i32) -> Self {
        self.config.threshold_delta = threshold_delta;
        self
    }

    /// Sets the merge threshold ratio (clamped to `[0.0, 1.0]`).
    pub fn merge_threshold_ratio(mut self, merge_threshold_ratio: f32) -> Self {
        self.config.merge_threshold_ratio = GridConfig::sanitize_ratio(merge_threshold_ratio);
        self
    }

    /// Enables or disables parallel processing.
    pub fn parallel(mut self, enable_parallel: bool) -> Self {
        self.config.enable_parallel = enable_parallel;
        self
    }

    /// Builds the [`GridConfig`].
    #[must_use]
    pub fn build(self) -> GridConfig {
        self.config
    }
}

impl Default for GridConfigBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::{
        DEFAULT_MERGE_THRESHOLD_RATIO, DEFAULT_THRESHOLD_BLOCK_SIZE, GridConfig, GridConfigBuilder,
        MIN_THRESHOLD_BLOCK_SIZE,
    };
    use crate::error::GridError;

    #[test]
    fn default_matches_documented_values() {
        let config = GridConfig::default();
        assert_eq!(config.threshold_block_size, DEFAULT_THRESHOLD_BLOCK_SIZE);
        assert_eq!(config.threshold_delta, 0);
        assert_eq!(config.merge_threshold_ratio, DEFAULT_MERGE_THRESHOLD_RATIO);
        assert!(config.enable_parallel);
    }

    #[test]
    fn new_clamps_block_size() {
        assert_eq!(
            GridConfig::new(1, 0.5, true).threshold_block_size,
            MIN_THRESHOLD_BLOCK_SIZE
        );
    }

    #[test]
    fn new_sanitizes_ratio() {
        assert_eq!(GridConfig::new(8, 2.0, true).merge_threshold_ratio, 1.0);
        assert_eq!(GridConfig::new(8, -1.0, true).merge_threshold_ratio, 0.0);
        assert_eq!(
            GridConfig::new(8, f32::NAN, true).merge_threshold_ratio,
            DEFAULT_MERGE_THRESHOLD_RATIO
        );
    }

    #[test]
    fn builder_overrides_every_field() {
        let config = GridConfig::builder()
            .threshold_block_size(0)
            .threshold_delta(9)
            .merge_threshold_ratio(9.0)
            .parallel(false)
            .build();
        assert_eq!(config.threshold_block_size, MIN_THRESHOLD_BLOCK_SIZE);
        assert_eq!(config.threshold_delta, 9);
        assert_eq!(config.merge_threshold_ratio, 1.0);
        assert!(!config.enable_parallel);
    }

    #[test]
    fn builder_default_equals_config_default() {
        assert_eq!(GridConfigBuilder::default().build(), GridConfig::default());
    }

    #[test]
    fn validate_rejects_zero_block_size() {
        let config = GridConfig {
            threshold_block_size: 0,
            ..GridConfig::default()
        };
        assert_eq!(
            config.validate(),
            Err(GridError::invalid_config(
                "threshold_block_size must be greater than 0"
            ))
        );
    }

    #[test]
    fn validate_rejects_out_of_range_ratio() {
        for ratio in [-0.5, 1.5, f32::INFINITY, f32::NEG_INFINITY, f32::NAN] {
            let config = GridConfig {
                merge_threshold_ratio: ratio,
                ..GridConfig::default()
            };
            assert!(config.validate().is_err(), "ratio {ratio} must be rejected");
        }
    }

    #[test]
    fn validate_accepts_default() {
        assert_eq!(GridConfig::default().validate(), Ok(()));
    }
}
