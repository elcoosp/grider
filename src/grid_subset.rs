use crate::grid::{Column, Grid, Row, SmallVecLine};
use std::ops::{Add, Sub};

/// Represents a subset of a `Grid`, referencing specific rows and columns.
#[derive(Debug, Clone, PartialEq)]
pub struct GridSubset<'a> {
    /// References to the rows in the parent `Grid`.
    rows: SmallVecLine<&'a Row>,
    /// References to the columns in the parent `Grid`.
    columns: SmallVecLine<&'a Column>,
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
}

/// Combines two `GridSubset` instances by merging their rows and columns.
impl Add for GridSubset<'_> {
    type Output = Self;

    fn add(self, other: Self) -> Self::Output {
        let mut rows = self.rows;
        rows.extend(other.rows);
        let mut columns = self.columns;
        columns.extend(other.columns);
        Self::new(rows, columns)
    }
}

/// Subtracts one `GridSubset` from another by removing overlapping rows and columns.
impl Sub for GridSubset<'_> {
    type Output = Self;

    fn sub(self, other: Self) -> Self::Output {
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

/// Creates a `GridSubset` from a reference to a `Grid`.
impl<'a> From<&'a Grid> for GridSubset<'a> {
    fn from(grid: &'a Grid) -> Self {
        let rows = grid.rows.iter().collect();
        let columns = grid.columns.iter().collect();
        Self::new(rows, columns)
    }
}

/// Updates a `Grid` from a `GridSubset`.
impl<'a> From<GridSubset<'a>> for Grid {
    fn from(subset: GridSubset<'a>) -> Self {
        let rows = subset.rows.into_iter().cloned().collect();
        let columns = subset.columns.into_iter().cloned().collect();
        Grid::new(rows, columns)
    }
}
