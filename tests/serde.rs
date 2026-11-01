//! Serde round-trip tests for the public data types.

use grider::{Column, Grid, GridConfig, LineInfo, LineKind, LineTrait, Row};

fn sample_grid() -> Grid {
    grider::make_grid!(
        rows: [(0, 5), (5, 5, LineKind::Full)],
        columns: [(0, 5), (5, 5, LineKind::Full)]
    )
}

#[test]
fn grid_serializes_to_expected_json() {
    let grid = sample_grid();
    let json = serde_json::to_string(&grid).unwrap();
    assert!(json.contains("\"rows\""));
    assert!(json.contains("\"kind\":\"Full\""));
}

#[test]
fn grid_round_trips_through_json() {
    let grid = sample_grid();
    let json = serde_json::to_string_pretty(&grid).unwrap();
    let restored: Grid = serde_json::from_str(&json).unwrap();
    assert_eq!(restored, grid);
}

#[test]
fn lines_and_line_info_round_trip() {
    let line = LineInfo::new(3, 9, LineKind::Full);
    let json = serde_json::to_string(&line).unwrap();
    assert_eq!(serde_json::from_str::<LineInfo>(&json).unwrap(), line);

    let row = Row::new(line);
    let json = serde_json::to_string(&row).unwrap();
    assert_eq!(serde_json::from_str::<Row>(&json).unwrap(), row);

    let column = Column::new(line);
    let json = serde_json::to_string(&column).unwrap();
    assert_eq!(serde_json::from_str::<Column>(&json).unwrap(), column);
}

#[test]
fn line_kind_serializes_as_strings() {
    assert_eq!(serde_json::to_string(&LineKind::Full).unwrap(), "\"Full\"");
    assert_eq!(
        serde_json::to_string(&LineKind::Empty).unwrap(),
        "\"Empty\""
    );
    assert_eq!(
        serde_json::from_str::<LineKind>("\"Full\"").unwrap(),
        LineKind::Full
    );
}

#[test]
fn config_round_trips() {
    let config = GridConfig::builder()
        .threshold_block_size(11)
        .threshold_delta(2)
        .merge_threshold_ratio(0.6)
        .parallel(false)
        .build();
    let json = serde_json::to_string(&config).unwrap();
    assert_eq!(serde_json::from_str::<GridConfig>(&json).unwrap(), config);
}
