use crate::grid::{Column, Row, SmallVecLine};
/// Represents a subset of a `Grid`, referencing specific rows and columns.
/// This type allows for operations like combining or filtering subsets.
#[derive(Debug, Clone, PartialEq)]
pub struct GridSubset<'a> {
    /// References to the rows in the parent `Grid`.
    pub rows: SmallVecLine<&'a Row>,
    /// References to the columns in the parent `Grid`.
    pub columns: SmallVecLine<&'a Column>,
}

impl<'a> GridSubset<'a> {
    /// Creates a new `GridSubset` from references to rows and columns.
    pub fn new(rows: SmallVecLine<&'a Row>, columns: SmallVecLine<&'a Column>) -> Self {
        Self { rows, columns }
    }

    /// Returns an iterator over the rows in the subset.
    pub fn rows(&self) -> impl Iterator<Item = &Row> {
        self.rows.iter().copied()
    }

    /// Returns an iterator over the columns in the subset.
    pub fn columns(&self) -> impl Iterator<Item = &Column> {
        self.columns.iter().copied()
    }

    /// Combines this subset with another, merging their rows and columns.
    pub fn combine_with(self, other: Self) -> Self {
        let mut rows = self.rows;
        rows.extend(other.rows);
        let mut columns = self.columns;
        columns.extend(other.columns);
        Self::new(rows, columns)
    }

    /// Filters out rows and columns that overlap with another subset.
    pub fn filter_out(self, other: Self) -> Self {
        let rows = self
            .rows
            .into_iter()
            .filter(|row| !other.rows.contains(row))
            .collect();
        let columns = self
            .columns
            .into_iter()
            .filter(|col| !other.columns.contains(col))
            .collect();
        Self::new(rows, columns)
    }
}
