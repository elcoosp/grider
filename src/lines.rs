//! Line primitives: kinds, raw line info, rows, columns and cells.
//!
//! A grid is composed of horizontal [`Row`]s and vertical [`Column`]s. Both are
//! projections of the same run of [`LineInfo`] produced by the detection
//! pipeline, unified by the [`LineTrait`] factory trait.

use imageproc::rect::Rect;

/// Represents the kind of a line (row or column): a run of fully background
/// pixels (`Empty`) or a run containing content (`Full`).
#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum LineKind {
    /// The line contains only background pixels.
    Empty,
    /// The line contains content.
    Full,
}

impl LineKind {
    /// Returns `true` if the line is [`LineKind::Empty`].
    ///
    /// # Examples
    ///
    /// ```
    /// use grider::LineKind;
    ///
    /// assert!(LineKind::Empty.is_empty());
    /// assert!(!LineKind::Full.is_empty());
    /// ```
    #[must_use]
    pub fn is_empty(self) -> bool {
        self == Self::Empty
    }

    /// Returns `true` if the line is [`LineKind::Full`].
    ///
    /// # Examples
    ///
    /// ```
    /// use grider::LineKind;
    ///
    /// assert!(LineKind::Full.is_full());
    /// assert!(!LineKind::Empty.is_full());
    /// ```
    #[must_use]
    pub fn is_full(self) -> bool {
        self == Self::Full
    }
}

/// Raw information about a run of consecutive lines sharing a [`LineKind`].
#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct LineInfo {
    /// Start position of the run along the scanned axis.
    pub start: u32,
    /// Number of consecutive scan lines in the run.
    pub length: u32,
    /// Kind of the run.
    pub kind: LineKind,
}

impl LineInfo {
    /// Creates a new `LineInfo` with the given start position, length, and kind.
    ///
    /// # Examples
    ///
    /// ```
    /// use grider::{LineInfo, LineKind};
    ///
    /// let line = LineInfo::new(0, 10, LineKind::Full);
    /// assert_eq!(line.start, 0);
    /// assert_eq!(line.length, 10);
    /// assert_eq!(line.kind, LineKind::Full);
    /// ```
    #[must_use]
    pub fn new(start: u32, length: u32, kind: LineKind) -> Self {
        Self {
            start,
            length,
            kind,
        }
    }
}

/// Represents a row (a horizontal band) in the grid.
#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Row {
    /// `y` coordinate where the row starts.
    pub y: u32,
    /// Height of the row in pixels.
    pub height: u32,
    /// Kind of the row.
    pub kind: LineKind,
}

impl Row {
    /// `y` coordinate just below the row (`y + height`, saturated).
    ///
    /// # Examples
    ///
    /// ```
    /// use grider::{LineInfo, LineKind, LineTrait, Row};
    ///
    /// let row = Row::new(LineInfo::new(2, 8, LineKind::Full));
    /// assert_eq!(row.end(), 10);
    /// ```
    #[must_use]
    pub fn end(&self) -> u32 {
        self.y.saturating_add(self.height)
    }
}

/// Represents a column (a vertical band) in the grid.
#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Column {
    /// `x` coordinate where the column starts.
    pub x: u32,
    /// Width of the column in pixels.
    pub width: u32,
    /// Kind of the column.
    pub kind: LineKind,
}

impl Column {
    /// `x` coordinate just right of the column (`x + width`, saturated).
    ///
    /// # Examples
    ///
    /// ```
    /// use grider::{Column, LineInfo, LineKind, LineTrait};
    ///
    /// let column = Column::new(LineInfo::new(4, 6, LineKind::Full));
    /// assert_eq!(column.end(), 10);
    /// ```
    #[must_use]
    pub fn end(&self) -> u32 {
        self.x.saturating_add(self.width)
    }
}

/// A cell of the grid: the intersection of a [`Row`] and a [`Column`].
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct Cell<'a> {
    /// The row defining the vertical extent of the cell.
    pub row: &'a Row,
    /// The column defining the horizontal extent of the cell.
    pub column: &'a Column,
}

impl Cell<'_> {
    /// The rectangle covered by this cell on the image.
    ///
    /// # Examples
    ///
    /// ```
    /// use grider::{make_grid, LineInfo};
    ///
    /// let grid = make_grid!(rows: [(0, 10)], columns: [(5, 15)]);
    /// let cell = grid.cell(0, 0).expect("single cell grid");
    /// let rect = cell.rect();
    /// assert_eq!((rect.left(), rect.top()), (5, 0));
    /// assert_eq!((rect.width(), rect.height()), (15, 10));
    /// ```
    #[must_use]
    pub fn rect(&self) -> Rect {
        Rect::from(self)
    }
}

impl From<&Cell<'_>> for Rect {
    fn from(cell: &Cell<'_>) -> Self {
        Rect::at(clamp_to_i32(cell.column.x), clamp_to_i32(cell.row.y))
            .of_size(cell.column.width, cell.row.height)
    }
}

/// Clamps a `u32` coordinate into the `i32` range expected by `imageproc`.
///
/// Coordinates beyond `i32::MAX` cannot occur for images that fit in memory,
/// but clamping keeps the conversion total and panic free.
#[must_use]
pub fn clamp_to_i32(value: u32) -> i32 {
    i32::try_from(value).unwrap_or(i32::MAX)
}

/// A trait providing a factory method to create instances from a [`LineInfo`].
///
/// Implemented by [`Row`] and [`Column`], allowing the detection pipeline to
/// convert generic line runs into the matching orientation-specific type.
///
/// # Examples
///
/// ```
/// use grider::{Column, LineInfo, LineTrait, LineKind, Row};
///
/// let line_info = LineInfo::new(0, 100, LineKind::Full);
/// let row = Row::new(line_info);
/// let column = Column::new(line_info);
///
/// assert_eq!(row.y, 0);
/// assert_eq!(column.x, 0);
/// ```
pub trait LineTrait {
    /// Creates a new instance from the given `LineInfo`.
    fn new(line: LineInfo) -> Self;
}

impl LineTrait for Row {
    fn new(line: LineInfo) -> Self {
        Row {
            y: line.start,
            height: line.length,
            kind: line.kind,
        }
    }
}

impl LineTrait for Column {
    fn new(line: LineInfo) -> Self {
        Column {
            x: line.start,
            width: line.length,
            kind: line.kind,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{Cell, Column, LineInfo, LineKind, LineTrait, Row, clamp_to_i32};
    use imageproc::rect::Rect;

    #[test]
    fn line_kind_helpers() {
        assert!(LineKind::Empty.is_empty());
        assert!(!LineKind::Empty.is_full());
        assert!(LineKind::Full.is_full());
        assert!(!LineKind::Full.is_empty());
    }

    #[test]
    fn row_and_column_wrap_line_info() {
        let row = Row::new(LineInfo::new(3, 9, LineKind::Full));
        assert_eq!(
            row,
            Row {
                y: 3,
                height: 9,
                kind: LineKind::Full
            }
        );
        let column = Column::new(LineInfo::new(1, 5, LineKind::Empty));
        assert_eq!(
            column,
            Column {
                x: 1,
                width: 5,
                kind: LineKind::Empty
            }
        );
    }

    #[test]
    fn end_is_saturating() {
        let row = Row {
            y: u32::MAX,
            height: 5,
            kind: LineKind::Full,
        };
        assert_eq!(row.end(), u32::MAX);
        let column = Column {
            x: 2,
            width: 3,
            kind: LineKind::Empty,
        };
        assert_eq!(column.end(), 5);
    }

    #[test]
    fn cell_rect_uses_column_then_row_extent() {
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
    fn cells_compare_by_value_not_identity() {
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
        assert_eq!(
            format!("{first:?}"),
            "Cell { row: Row { y: 0, height: 1, kind: Full }, column: Column { x: 0, width: 1, kind: Full } }"
        );
    }

    #[test]
    fn clamp_saturates_huge_values() {
        assert_eq!(clamp_to_i32(0), 0);
        assert_eq!(clamp_to_i32(1_000), 1_000);
        assert_eq!(clamp_to_i32(u32::MAX), i32::MAX);
    }

    #[test]
    fn line_info_is_copy() {
        let info = LineInfo::new(1, 2, LineKind::Full);
        let copy = info;
        assert_eq!(info, copy);
    }
}
