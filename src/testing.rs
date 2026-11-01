//! Shared fixtures for tests and benchmarks.
//!
//! This module is only available behind the non-default `test-utils` feature:
//!
//! ```toml
//! [dev-dependencies]
//! grider = { version = "0.2", features = ["test-utils"] }
//! ```
//!
//! # Examples
//!
//! ```
//! use grider::testing::{Pattern, test_image};
//! use image::GenericImageView;
//!
//! let img = test_image(8, 8, Pattern::Checkerboard);
//! assert_eq!(img.dimensions(), (8, 8));
//! ```

use image::{DynamicImage, GrayImage, Luma};

/// Standard image patterns used by the fixtures.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Pattern {
    /// Alternating black and white pixels.
    Checkerboard,
    /// Horizontal black/white stripes.
    HorizontalStripes,
    /// Vertical black/white stripes.
    VerticalStripes,
    /// White image with isolated black pixels every 10 px.
    Sparse,
    /// Black grid every 3 px on a white background.
    Dense,
    /// Left-to-right luminance gradient.
    Gradient,
    /// Fully white image.
    Empty,
    /// Fully black image.
    Full,
}

/// Creates a grayscale [`DynamicImage`] of `width` × `height` filled with the
/// given `pattern`.
#[must_use]
pub fn test_image(width: u32, height: u32, pattern: Pattern) -> DynamicImage {
    DynamicImage::ImageLuma8(gray_test_image(width, height, pattern))
}

/// Creates a grayscale [`GrayImage`] filled with the given `pattern`.
#[must_use]
pub fn gray_test_image(width: u32, height: u32, pattern: Pattern) -> GrayImage {
    match pattern {
        Pattern::Checkerboard => GrayImage::from_fn(width, height, |x, y| {
            if (x + y).is_multiple_of(2) {
                Luma([255])
            } else {
                Luma([0])
            }
        }),
        Pattern::HorizontalStripes => GrayImage::from_fn(width, height, |_, y| {
            if y.is_multiple_of(2) {
                Luma([255])
            } else {
                Luma([0])
            }
        }),
        Pattern::VerticalStripes => GrayImage::from_fn(width, height, |x, _| {
            if x.is_multiple_of(2) {
                Luma([255])
            } else {
                Luma([0])
            }
        }),
        Pattern::Sparse => GrayImage::from_fn(width, height, |x, y| {
            if x.is_multiple_of(10) && y.is_multiple_of(10) {
                Luma([0])
            } else {
                Luma([255])
            }
        }),
        Pattern::Dense => GrayImage::from_fn(width, height, |x, y| {
            if x.is_multiple_of(3) || y.is_multiple_of(3) {
                Luma([0])
            } else {
                Luma([255])
            }
        }),
        Pattern::Gradient => GrayImage::from_fn(width, height, |x, _| {
            Luma([u8::try_from((x as usize * 255) / width.max(1) as usize).unwrap_or(u8::MAX)])
        }),
        Pattern::Empty => GrayImage::from_pixel(width, height, Luma([255])),
        Pattern::Full => GrayImage::from_pixel(width, height, Luma([0])),
    }
}

#[cfg(test)]
mod tests {
    use super::{Pattern, gray_test_image, test_image};
    use image::GenericImageView;

    #[test]
    fn test_image_reports_requested_dimensions() {
        assert_eq!(test_image(9, 7, Pattern::Checkerboard).dimensions(), (9, 7));
    }

    #[test]
    fn checkerboard_alternates() {
        let img = gray_test_image(2, 2, Pattern::Checkerboard);
        assert_eq!(img.get_pixel(0, 0)[0], 255);
        assert_eq!(img.get_pixel(1, 0)[0], 0);
        assert_eq!(img.get_pixel(0, 1)[0], 0);
        assert_eq!(img.get_pixel(1, 1)[0], 255);
    }

    #[test]
    fn stripes_orient_as_named() {
        let horizontal = gray_test_image(2, 2, Pattern::HorizontalStripes);
        assert_eq!(horizontal.get_pixel(0, 0)[0], 255);
        assert_eq!(horizontal.get_pixel(0, 1)[0], 0);

        let vertical = gray_test_image(2, 2, Pattern::VerticalStripes);
        assert_eq!(vertical.get_pixel(0, 0)[0], 255);
        assert_eq!(vertical.get_pixel(1, 0)[0], 0);
    }

    #[test]
    fn sparse_marks_isolated_points() {
        let img = gray_test_image(11, 11, Pattern::Sparse);
        assert_eq!(img.get_pixel(0, 0)[0], 0);
        assert_eq!(img.get_pixel(10, 10)[0], 0);
        assert_eq!(img.get_pixel(5, 5)[0], 255);
    }

    #[test]
    fn dense_marks_every_third_line() {
        let img = gray_test_image(4, 4, Pattern::Dense);
        assert_eq!(img.get_pixel(3, 3)[0], 0);
        assert_eq!(img.get_pixel(1, 1)[0], 255);
    }

    #[test]
    fn gradient_increases_left_to_right() {
        let img = gray_test_image(4, 1, Pattern::Gradient);
        assert!(img.get_pixel(0, 0)[0] < img.get_pixel(3, 0)[0]);
    }

    #[test]
    fn gradient_survives_zero_width() {
        let img = gray_test_image(1, 1, Pattern::Gradient);
        assert_eq!(img.get_pixel(0, 0)[0], 0);
    }

    #[test]
    fn empty_and_full_are_extremes() {
        let empty = gray_test_image(2, 2, Pattern::Empty);
        let full = gray_test_image(2, 2, Pattern::Full);
        assert_eq!(empty.get_pixel(0, 0)[0], 255);
        assert_eq!(full.get_pixel(0, 0)[0], 0);
    }

    #[test]
    fn test_image_returns_luma8_dynamic_image() {
        let img = test_image(1, 1, Pattern::Empty);
        assert!(
            img.as_luma8().is_some(),
            "test_image must return an 8-bit luma image"
        );
    }
}
