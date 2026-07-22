use std::ops::Range;

/// A source range in UTF-8 byte offsets.
///
/// # Examples
///
/// ```
/// use thai_syllable_rules::{ByteRange, Part};
///
/// let range: ByteRange = Part::Dot(6..7).byte_range();
/// assert_eq!(range, 6..7);
/// ```
pub type ByteRange = Range<usize>;
