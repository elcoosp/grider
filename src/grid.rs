use crate::{grid_like::GridLike, grid_subset::GridSubset};
use imageproc::rect::Rect;
use smallvec::SmallVec;
/// Represents the kind of a line (row or column).
#[derive(Debug, PartialEq, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub enum LineKind {
    Empty,
    Full,
}

/// Information about a line in the grid.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct LineInfo {
    pub start: u32,
    pub length: u32,
    pub kind: LineKind,
}

impl LineInfo {
    /// Creates a new `LineInfo` with the given start position, length, and kind.
    ///
    /// # Example
    /// ```
    /// use grider::{LineInfo, LineKind};
    ///
    /// let line = LineInfo::new(0, 10, LineKind::Full);
    /// assert_eq!(line.start, 0);
    /// assert_eq!(line.length, 10);
    /// assert_eq!(line.kind, LineKind::Full);
    /// ```
    pub fn new(start: u32, length: u32, kind: LineKind) -> Self {
        Self {
            start,
            length,
            kind,
        }
    }
}

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
/// A type alias for SmallVec with an optimized stack-allocated buffer size.
pub type SmallVecLine<T> = SmallVec<[T; 32]>;

/// Represents the grid of rows and columns extracted from an image.
#[derive(Debug, PartialEq, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct Grid {
    pub rows: SmallVecLine<Row>,
    pub columns: SmallVecLine<Column>,
}

impl Grid {
    /// Creates a new `Grid` from rows and columns.
    pub fn new(rows: SmallVecLine<Row>, columns: SmallVecLine<Column>) -> Self {
        Self { rows, columns }
    }

    /// Creates a `GridSubset` referencing specific rows and columns.
    pub fn create_subset<'a>(
        &'a self,
        row_indices: &[usize],
        column_indices: &[usize],
    ) -> GridSubset<'a> {
        let rows = row_indices.iter().map(|&i| &self.rows[i]).collect();
        let columns = column_indices.iter().map(|&i| &self.columns[i]).collect();
        GridSubset::new(rows, columns)
    }
}

impl GridLike for Grid {
    /// Returns an iterator over all rows in the grid.
    fn rows(&self) -> impl Iterator<Item = &Row> {
        self.rows.iter()
    }

    /// Returns an iterator over all columns in the grid.
    fn columns(&self) -> impl Iterator<Item = &Column> {
        self.columns.iter()
    }
}
