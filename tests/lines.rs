//! Tests for the line primitives in [`grider::lines`].

use grider::{Cell, Column, LineInfo, LineKind, LineTrait, Row, clamp_to_i32};
use imageproc::rect::Rect;

#[test]
fn line_kind_helpers_reflect_variants() {
    assert!(LineKind::Empty.is_empty());
    assert!(!LineKind::Empty.is_full());
    assert!(LineKind::Full.is_full());
    assert!(!LineKind::Full.is_empty());
}

#[test]
fn line_info_stores_its_fields() {
    let line = LineInfo::new(10, 5, LineKind::Full);
    assert_eq!(line.start, 10);
    assert_eq!(line.length, 5);
    assert_eq!(line.kind, LineKind::Full);
}

#[test]
fn rows_and_columns_wrap_line_info() {
    let row = Row::new(LineInfo::new(10, 5, LineKind::Full));
    assert_eq!(
        row,
        Row {
            y: 10,
            height: 5,
            kind: LineKind::Full
        }
    );
    assert_eq!(row.end(), 15);

    let column = Column::new(LineInfo::new(2, 7, LineKind::Empty));
    assert_eq!(
        column,
        Column {
            x: 2,
            width: 7,
            kind: LineKind::Empty
        }
    );
    assert_eq!(column.end(), 9);
}

#[test]
fn end_saturates_instead_of_overflowing() {
    let row = Row {
        y: u32::MAX,
        height: 10,
        kind: LineKind::Full,
    };
    assert_eq!(row.end(), u32::MAX);
    let column = Column {
        x: u32::MAX - 1,
        width: 10,
        kind: LineKind::Empty,
    };
    assert_eq!(column.end(), u32::MAX);
}

#[test]
fn cells_expose_their_rectangle() {
    let row = Row::new(LineInfo::new(10, 4, LineKind::Full));
    let column = Column::new(LineInfo::new(20, 6, LineKind::Full));
    let cell = Cell {
        row: &row,
        column: &column,
    };

    let rect = Rect::from(&cell);
    assert_eq!(rect, Rect::at(20, 10).of_size(6, 4));
    assert_eq!(cell.rect(), rect);
}

#[test]
fn cells_debug_print_includes_both_lines() {
    let row = Row::new(LineInfo::new(0, 1, LineKind::Full));
    let column = Column::new(LineInfo::new(0, 1, LineKind::Full));
    let cell = Cell {
        row: &row,
        column: &column,
    };
    let debug = format!("{cell:?}");
    assert!(debug.contains("Row { y: 0, height: 1, kind: Full }"));
    assert!(debug.contains("Column { x: 0, width: 1, kind: Full }"));
}

#[test]
fn cells_compare_by_referenced_value() {
    let row = Row::new(LineInfo::new(0, 1, LineKind::Full));
    let column = Column::new(LineInfo::new(0, 1, LineKind::Full));
    let first = Cell {
        row: &row,
        column: &column,
    };
    let second = Cell {
        row: &row,
        column: &column,
    };
    assert_eq!(first, second);
}

#[test]
fn clamp_to_i32_saturates_huge_values() {
    assert_eq!(clamp_to_i32(0), 0);
    assert_eq!(clamp_to_i32(u32::try_from(i32::MAX).unwrap()), i32::MAX);
    assert_eq!(clamp_to_i32(u32::MAX), i32::MAX);
}

#[test]
fn line_types_are_copy() {
    let info = LineInfo::new(1, 2, LineKind::Full);
    let copied_info = info;
    assert_eq!(info, copied_info);

    let row = Row::new(info);
    let copied_row = row;
    assert_eq!(row, copied_row);

    let column = Column::new(info);
    let copied_column = column;
    assert_eq!(column, copied_column);
}
