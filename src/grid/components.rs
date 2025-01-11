use super::*;

/// Represents a row in the grid.
#[derive(Debug, PartialEq, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct Row {
    pub y: u32,
    pub height: u32,
    pub kind: LineKind,
}

/// Represents a column in the grid.
#[derive(Debug, PartialEq, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct Column {
    pub x: u32,
    pub width: u32,
    pub kind: LineKind,
}
/// Represents a cell in the grid, referencing a row and a column.
pub struct Cell<'a> {
    pub row: &'a Row,
    pub column: &'a Column,
}
impl From<&Cell<'_>> for Rect {
    fn from(cell: &Cell) -> Self {
        Rect::at(cell.column.x as i32, cell.row.y as i32)
            .of_size(cell.column.width, cell.row.height)
    }
}
