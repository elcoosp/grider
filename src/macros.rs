//! Convenience macros for building grids and lines in tests and fixtures.

/// Creates a `Row` or `Column` instance from a tuple of values.
///
/// This macro simplifies the creation of `Row` or `Column` instances by
/// providing a concise syntax. It supports optional specification of the
/// `LineKind` (defaulting to `LineKind::Empty` when omitted).
///
/// # Syntax
///
/// ```rust
/// use grider::*;
/// let _ = make_line!(Row, (0, 0, LineKind::Empty));
/// let _ = make_line!(Column, (0, 0, LineKind::Full));
/// ```
///
/// - `y` / `x`: The starting position of the row/column.
/// - `height` / `width`: The length of the row/column.
/// - `kind`: The `LineKind` (`LineKind::Empty` or `LineKind::Full`).
///
/// # Examples
///
/// ## Creating a `Row`
///
/// ```rust
/// use grider::*;
///
/// // With LineKind::Full
/// let row = make_line!(Row, (0, 10, LineKind::Full));
/// assert_eq!(row.y, 0);
/// assert_eq!(row.height, 10);
/// assert_eq!(row.kind, LineKind::Full);
///
/// // Without LineKind (defaults to LineKind::Empty)
/// let row = make_line!(Row, (10, 20));
/// assert_eq!(row.y, 10);
/// assert_eq!(row.height, 20);
/// assert_eq!(row.kind, LineKind::Empty);
/// ```
///
/// ## Creating a `Column`
///
/// ```rust
/// use grider::*;
///
/// // With LineKind::Full
/// let column = make_line!(Column, (5, 15, LineKind::Full));
/// assert_eq!(column.x, 5);
/// assert_eq!(column.width, 15);
/// assert_eq!(column.kind, LineKind::Full);
///
/// // Without LineKind (defaults to LineKind::Empty)
/// let column = make_line!(Column, (20, 30));
/// assert_eq!(column.x, 20);
/// assert_eq!(column.width, 30);
/// assert_eq!(column.kind, LineKind::Empty);
/// ```
#[macro_export]
macro_rules! make_line {
    // For Rows
    (Row, ($y:expr, $height:expr, $kind:expr)) => {
        <$crate::Row as $crate::LineTrait>::new($crate::LineInfo::new($y, $height, $kind))
    };
    (Row, ($y:expr, $height:expr)) => {
        <$crate::Row as $crate::LineTrait>::new($crate::LineInfo::new(
            $y,
            $height,
            $crate::LineKind::Empty,
        ))
    };

    // For Columns
    (Column, ($x:expr, $width:expr, $kind:expr)) => {
        <$crate::Column as $crate::LineTrait>::new($crate::LineInfo::new($x, $width, $kind))
    };
    (Column, ($x:expr, $width:expr)) => {
        <$crate::Column as $crate::LineTrait>::new($crate::LineInfo::new(
            $x,
            $width,
            $crate::LineKind::Empty,
        ))
    };
}

/// Creates a `Grid` instance from a list of rows and columns.
///
/// Each list contains tuples that define the properties of the rows or
/// columns; the macro internally uses [`make_line!`] to construct them. A
/// trailing comma after each tuple is optional.
///
/// # Syntax
///
/// ```rust
/// use grider::*;
/// let x = 0;
/// let y = 0;
/// let height = 0;
/// let width = 0;
/// let grid = make_grid!(
///     rows: [
///         (y, height),                 // Row with default LineKind::Empty
///         (y, height, LineKind::Full), // Row with explicit LineKind
///     ],
///     columns: [
///         (x, width),                  // Column with default LineKind::Empty
///         (x, width, LineKind::Full),  // Column with explicit LineKind
///     ]
/// );
/// ```
///
/// # Examples
///
/// ```rust
/// use grider::{make_grid, LineKind};
///
/// let grid = make_grid!(
///     rows: [
///         (0, 10),                     // Row at y=0, height=10, LineKind::Empty
///         (10, 20, LineKind::Full),    // Row at y=10, height=20, LineKind::Full
///     ],
///     columns: [
///         (0, 5),                      // Column at x=0, width=5, LineKind::Empty
///         (5, 15, LineKind::Full),     // Column at x=5, width=15, LineKind::Full
///     ]
/// );
///
/// assert_eq!(grid.rows.len(), 2);
/// assert_eq!(grid.columns.len(), 2);
/// ```
///
/// # Notes
///
/// - If `LineKind` is not provided for a row or column, it defaults to
///   `LineKind::Empty`.
/// - The resulting `Grid` stores rows and columns in
///   [`SmallVecLine`](crate::detection::SmallVecLine) collections.
#[macro_export]
macro_rules! make_grid {
    // Match rows and columns as separate lists with tuple syntax; a trailing
    // comma after each tuple is optional.
    (rows: [$($row:tt),* $(,)?], columns: [$($col:tt),* $(,)?]) => {{
        $crate::Grid {
            rows: $crate::SmallVecLine::from_vec(vec![
                $($crate::make_line!(Row, $row)),*
            ]),
            columns: $crate::SmallVecLine::from_vec(vec![
                $($crate::make_line!(Column, $col)),*
            ]),
        }
    }};
}
