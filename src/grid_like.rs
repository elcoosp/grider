use crate::grid::{Column, Row};

/// A trait for types that provide access to rows and columns.
pub trait GridLike {
    /// Returns an iterator over all rows.
    fn rows(&self) -> impl Iterator<Item = &Row>;

    /// Returns an iterator over all columns.
    fn columns(&self) -> impl Iterator<Item = &Column>;

    /// Returns the number of rows.
    fn row_count(&self) -> usize {
        self.rows().count()
    }

    /// Returns the number of columns.
    fn column_count(&self) -> usize {
        self.columns().count()
    }
}
