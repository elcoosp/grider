//! Tests for [`GridConfig`] and its builder.

use grider::{
    DEFAULT_MERGE_THRESHOLD_RATIO, DEFAULT_THRESHOLD_BLOCK_SIZE, DEFAULT_THRESHOLD_DELTA,
    GridConfig, GridError, MIN_THRESHOLD_BLOCK_SIZE,
};

#[test]
fn default_matches_documented_values() {
    let config = GridConfig::default();
    assert_eq!(config.threshold_block_size, DEFAULT_THRESHOLD_BLOCK_SIZE);
    assert_eq!(config.threshold_delta, DEFAULT_THRESHOLD_DELTA);
    assert_eq!(config.merge_threshold_ratio, DEFAULT_MERGE_THRESHOLD_RATIO);
    assert!(config.enable_parallel);
    assert!(config.validate().is_ok());
}

#[test]
fn new_applies_documented_sanitization() {
    let config = GridConfig::new(15, 0.9, false);
    assert_eq!(config.threshold_block_size, 15);
    assert_eq!(config.merge_threshold_ratio, 0.9);
    assert!(!config.enable_parallel);
    assert_eq!(config.threshold_delta, 0);

    assert_eq!(
        GridConfig::new(0, 0.5, true).threshold_block_size,
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
        .threshold_block_size(7)
        .threshold_delta(9)
        .merge_threshold_ratio(0.5)
        .parallel(false)
        .build();
    assert_eq!(config.threshold_block_size, 7);
    assert_eq!(config.threshold_delta, 9);
    assert_eq!(config.merge_threshold_ratio, 0.5);
    assert!(!config.enable_parallel);
    assert!(config.validate().is_ok());
}

#[test]
fn builder_clamps_like_the_constructor() {
    let config = GridConfig::builder()
        .threshold_block_size(0)
        .merge_threshold_ratio(f32::NAN)
        .build();
    assert_eq!(config.threshold_block_size, MIN_THRESHOLD_BLOCK_SIZE);
    assert_eq!(config.merge_threshold_ratio, DEFAULT_MERGE_THRESHOLD_RATIO);
}

#[test]
fn builder_default_equals_config_default() {
    assert_eq!(
        grider::GridConfigBuilder::default().build(),
        GridConfig::default()
    );
}

#[test]
fn validate_rejects_zero_block_size() {
    let config = GridConfig {
        threshold_block_size: 0,
        ..GridConfig::default()
    };
    match config.validate() {
        Err(GridError::InvalidConfig { reason }) => {
            assert!(reason.contains("threshold_block_size"));
        }
        other => panic!("unexpected result: {other:?}"),
    }
}

#[test]
fn validate_rejects_out_of_range_ratios() {
    for ratio in [-0.5, 1.5, f32::INFINITY, f32::NEG_INFINITY, f32::NAN] {
        let config = GridConfig {
            merge_threshold_ratio: ratio,
            ..GridConfig::default()
        };
        assert!(config.validate().is_err(), "ratio {ratio} must be rejected");
    }
}

#[test]
fn error_display_is_stable() {
    assert_eq!(
        GridError::ImageConversionError("boom".to_owned()).to_string(),
        "Failed to convert image: boom"
    );
    assert_eq!(
        GridError::InvalidDimensions {
            width: 640,
            height: 480
        }
        .to_string(),
        "Invalid image dimensions: width=640, height=480"
    );
    assert_eq!(
        GridError::RowNotFound { y: 3 }.to_string(),
        "Row not found at y=3"
    );
    assert_eq!(
        GridError::ColumnNotFound { x: 4 }.to_string(),
        "Column not found at x=4"
    );
    assert_eq!(
        GridError::invalid_config("bad").to_string(),
        "Invalid configuration: bad"
    );
}

#[cfg(feature = "serde")]
mod serde {
    use super::GridConfig;

    #[test]
    fn config_round_trips_through_json() {
        let config = GridConfig::builder()
            .threshold_block_size(9)
            .threshold_delta(-3)
            .merge_threshold_ratio(0.4)
            .parallel(false)
            .build();
        let json = serde_json::to_string(&config).unwrap();
        assert_eq!(serde_json::from_str::<GridConfig>(&json).unwrap(), config);
    }
}
