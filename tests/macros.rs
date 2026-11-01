//! Tests for the [`make_line!`] and [`make_grid!`] macros.

use grider::{Column, Grid, LineInfo, LineKind, LineTrait, Row};

#[test]
fn make_line_row_with_kind() {
    let row = grider::make_line!(Row, (0, 10, LineKind::Full));
    assert_eq!(
        row,
        Row {
            y: 0,
            height: 10,
            kind: LineKind::Full
        }
    );
}

#[test]
fn make_line_row_defaults_to_empty() {
    let row = grider::make_line!(Row, (10, 20));
    assert_eq!(
        row,
        Row {
            y: 10,
            height: 20,
            kind: LineKind::Empty
        }
    );
}

#[test]
fn make_line_column_with_kind() {
    let column = grider::make_line!(Column, (5, 15, LineKind::Full));
    assert_eq!(
        column,
        Column {
            x: 5,
            width: 15,
            kind: LineKind::Full
        }
    );
}

#[test]
fn make_line_column_defaults_to_empty() {
    let column = grider::make_line!(Column, (20, 30));
    assert_eq!(
        column,
        Column {
            x: 20,
            width: 30,
            kind: LineKind::Empty
        }
    );
}

#[test]
fn make_grid_with_trailing_commas_and_kinds() {
    let grid = grider::make_grid!(
        rows: [
            (0, 10),
            (10, 20, LineKind::Full),
        ],
        columns: [
            (0, 5),
            (5, 15, LineKind::Full),
        ]
    );
    assert_eq!(grid.rows.len(), 2);
    assert_eq!(grid.columns.len(), 2);
    assert_eq!(grid.rows[1].kind, LineKind::Full);
    assert_eq!(grid.columns[1].kind, LineKind::Full);
}

#[test]
fn make_grid_without_trailing_commas() {
    let grid = grider::make_grid!(
        rows: [(0, 5)],
        columns: [(0, 5)]
    );
    assert_eq!(grid.rows.len(), 1);
    assert_eq!(grid.columns.len(), 1);
}

#[test]
fn make_grid_with_single_entries() {
    let grid =
        grider::make_grid!(rows: [(0, 5, LineKind::Full)], columns: [(0, 5, LineKind::Full)]);
    assert_eq!(
        grid,
        Grid {
            rows: [Row::new(LineInfo::new(0, 5, LineKind::Full))]
                .into_iter()
                .collect(),
            columns: [Column::new(LineInfo::new(0, 5, LineKind::Full))]
                .into_iter()
                .collect(),
        }
    );
}
