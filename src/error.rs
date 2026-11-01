//! Error types produced by grid extraction and drawing.

use thiserror::Error;

/// All errors that [`crate::Grid`] operations can produce.
///
/// The enum is `#[non_exhaustive]`: new variants may be added in minor releases,
/// so downstream `match` arms should include a wildcard.
#[derive(Error, Debug, PartialEq, Eq, Clone)]
#[non_exhaustive]
pub enum GridError {
    /// The image has zero width or zero height.
    #[error("Invalid image dimensions: width={width}, height={height}")]
    InvalidDimensions {
        /// The offending image width.
        width: u32,
        /// The offending image height.
        height: u32,
    },

    /// A row was looked up by `y` coordinate but no row starts there.
    #[error("Row not found at y={y}")]
    RowNotFound {
        /// The `y` coordinate that was searched.
        y: u32,
    },

    /// A column was looked up by `x` coordinate but no column starts there.
    #[error("Column not found at x={x}")]
    ColumnNotFound {
        /// The `x` coordinate that was searched.
        x: u32,
    },

    /// A configuration value is out of its valid range.
    #[error("Invalid configuration: {reason}")]
    InvalidConfig {
        /// Human readable description of the invalid value.
        reason: String,
    },

    /// The image could not be converted or saved.
    #[error("Failed to convert image: {0}")]
    ImageConversionError(String),
}

impl GridError {
    /// Convenience constructor for [`GridError::InvalidConfig`].
    ///
    /// # Examples
    ///
    /// ```
    /// use grider::GridError;
    ///
    /// let error = GridError::invalid_config("block size must be > 0");
    /// assert_eq!(error.to_string(), "Invalid configuration: block size must be > 0");
    /// ```
    #[must_use]
    pub fn invalid_config(reason: impl Into<String>) -> Self {
        Self::InvalidConfig {
            reason: reason.into(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::GridError;

    #[test]
    fn display_includes_dimensions() {
        let error = GridError::InvalidDimensions {
            width: 3,
            height: 7,
        };
        assert_eq!(
            error.to_string(),
            "Invalid image dimensions: width=3, height=7"
        );
    }

    #[test]
    fn display_includes_coordinates() {
        assert_eq!(
            GridError::RowNotFound { y: 12 }.to_string(),
            "Row not found at y=12"
        );
        assert_eq!(
            GridError::ColumnNotFound { x: 4 }.to_string(),
            "Column not found at x=4"
        );
    }

    #[test]
    fn display_includes_reason() {
        assert_eq!(
            GridError::invalid_config("nope").to_string(),
            "Invalid configuration: nope"
        );
    }

    #[test]
    fn display_includes_message() {
        let error = GridError::ImageConversionError("boom".to_owned());
        assert_eq!(error.to_string(), "Failed to convert image: boom");
    }
}
