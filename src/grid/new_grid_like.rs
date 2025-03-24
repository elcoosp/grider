pub trait GridLike {
    type Row;
    type Column;

    /// Returns an iterator over all rows.
    fn rows_iter(&self) -> impl Iterator<Item = &Self::Row>;

    /// Returns an iterator over all columns.
    fn columns_iter(&self) -> impl Iterator<Item = &Self::Column>;

    /// Filters the rows based on a predicate closure and returns an iterator over matching rows.
    fn filter_rows<F>(&self, predicate: F) -> impl Iterator<Item = &Self::Row>
    where
        F: Fn(&Self::Row) -> bool,
    {
        self.rows_iter().filter(predicate)
    }

    /// Filters the columns based on a predicate closure and returns an iterator over matching columns.
    fn filter_columns<F>(&self, predicate: F) -> impl Iterator<Item = &Self::Column>
    where
        F: Fn(&Self::Column) -> bool,
    {
        self.columns_iter().filter(predicate)
    }

    /// Returns the number of rows.
    fn row_count(&self) -> usize {
        self.rows_iter().count()
    }

    /// Returns the number of columns.
    fn column_count(&self) -> usize {
        self.columns_iter().count()
    }

    /// Filters and returns an iterator over the shortest rows, considering a percentage tolerance for variation.
    ///
    /// # Examples
    /// ```
    /// let shortest_rows: Vec<&Self::Row> = grid.filter_smallest_rows_with_tolerance(10.0).collect();
    /// ```
    fn filter_smallest_rows_with_tolerance(
        &self,
        tolerance_percent: f64,
    ) -> impl Iterator<Item = &Self::Row> {
        let min_length = self.rows_iter().map(|row| row.len()).min().unwrap_or(1); // Avoid division by zero.

        let max_length = min_length as f64 * (1.0 + tolerance_percent / 100.0);

        self.filter_rows(move |row| row.len() as f64 <= max_length)
    }

    /// Filters and returns an iterator over the shortest columns, considering a percentage tolerance for variation.
    ///
    /// # Examples
    /// ```
    /// let shortest_columns: Vec<&Self::Column> = grid.filter_smallest_columns_with_tolerance(10.0).collect();
    /// ```
    fn filter_smallest_columns_with_tolerance(
        &self,
        tolerance_percent: f64,
    ) -> impl Iterator<Item = &Self::Column> {
        let min_length = self
            .columns_iter()
            .map(|column| column.len())
            .min()
            .unwrap_or(1); // Avoid division by zero.

        let max_length = min_length as f64 * (1.0 + tolerance_percent / 100.0);

        self.filter_columns(move |column| column.len() as f64 <= max_length)
    }
}
