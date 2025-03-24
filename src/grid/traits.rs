use super::*;

/// A trait providing a factory method to create instances from a [`LineInfo`].
///
/// This trait is implemented by types like [`Row`] and [`Column`],
/// allowing them to be instantiated from a `LineInfo` that contains
/// the start position, length, and kind of a line.
///
/// The `new` function is used to convert generic line information
/// into specific row or column instances, facilitating the processing
/// of grid lines in a uniform manner.
///
/// # Examples
///
/// ```
/// use grider::{LineTrait, LineInfo, Row, Column, LineKind};
///
/// // Create a LineInfo instance
/// let line_info = LineInfo::new(0, 100, LineKind::Full);
///
/// // Instantiate a Row from the LineInfo
/// let row = Row::new(line_info.clone());
///
/// // Similarly, instantiate a Column
/// let column = Column::new(line_info);
/// ```
///
/// # See Also
///
/// * [`LineInfo`] for details on line information.
/// * [`Row`] and [`Column`] for grid line representations.
///
/// [`Row`]: struct.Row.html
/// [`Column`]: struct.Column.html
/// [`LineInfo`]: struct.LineInfo.html
pub trait LineTrait {
    /// Creates a new instance from the given `LineInfo`.
    ///
    /// This function is responsible for mapping the generic line information
    /// to the specific attributes of the implementing type.
    ///
    /// # Parameters
    ///
    /// * `line` - The `LineInfo` containing the details for the new instance.
    ///
    /// # Returns
    ///
    /// A new instance of the implementing type initialized with the provided line information.
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
pub trait Len {
    fn len(&self) -> usize;
}
impl Len for Column {
    fn len(&self) -> usize {
        self.width as usize
    }
}
impl Len for Row {
    fn len(&self) -> usize {
        self.height as usize
    }
}
