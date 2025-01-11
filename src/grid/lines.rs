use super::*;
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
/// A type alias for SmallVec with an optimized stack-allocated buffer size.
pub type SmallVecLine<T> = SmallVec<[T; DEFAULT_SMALLVEC_SIZE]>;

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
