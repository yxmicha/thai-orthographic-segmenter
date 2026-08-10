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

/// A rule-based part of the original source text.
///
/// The variants preserve the source range exactly. `Syllable` and `Undecided`
/// are structural results, `Invalid` preserves a malformed local Thai unit,
/// and `Dot` and `Text` preserve source material handled by the outer scanner.
///
/// # Examples
///
/// ```
/// use thai_syllable_rules::{Part, split};
///
/// // `กะ` is a syllable; the two bare consonants after it stay undecided until a later
/// // stage chooses between `กะต` + `กลม` and `กะ` + `ตกลม`.
/// let source = "กะตกลม";
/// let parts = split(source);
/// assert_eq!(
///     parts,
///     [
///         Part::Syllable(0..6),
///         Part::Undecided(6..9),
///         Part::Undecided(9..12),
///         Part::Syllable(12..18),
///     ]
/// );
/// // The parts cover the source exactly.
/// let joined: String = parts.iter().map(|part| part.text(source)).collect();
/// assert_eq!(joined, source);
/// ```
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Part {
    /// A Thai orthographic candidate or resolved anchor. Adjacent `Undecided`
    /// material may still belong to it; this is not a guarantee of a complete
    /// spoken syllable or a dictionary-selected boundary.
    Syllable(ByteRange),
    /// Thai material whose attachment to the preceding or following syllable
    /// cannot be decided from orthography alone, including an abbreviation-
    /// like singleton immediately before a recognized period.
    Undecided(ByteRange),
    /// A cell violating the parser's combining-mark/source-order policy, or a construction still
    /// missing its required ending after legal completion has been considered.
    Invalid(ByteRange),
    /// Exactly one valid ASCII period following a syllable or undecided part.
    Dot(ByteRange),
    /// Non-Thai material, orphan Thai marks, and invalid or misplaced periods.
    Text(ByteRange),
}

impl Part {
    /// Returns the exact UTF-8 byte range in the input source.
    ///
    /// # Examples
    ///
    /// ```
    /// use thai_syllable_rules::Part;
    ///
    /// let part = Part::Syllable(0..6);
    /// assert_eq!(part.byte_range(), 0..6);
    /// ```
    #[must_use]
    pub fn byte_range(&self) -> ByteRange {
        match self {
            Self::Syllable(range)
            | Self::Undecided(range)
            | Self::Invalid(range)
            | Self::Dot(range)
            | Self::Text(range) => range.clone(),
        }
    }

    pub(crate) fn is_syllable(&self) -> bool {
        matches!(self, Self::Syllable(_))
    }
}

/// Appends preserved source text while coalescing adjacent text ranges.
pub(crate) fn push_text(parts: &mut Vec<Part>, range: ByteRange) {
    if let Some(Part::Text(previous)) = parts.last_mut()
        && previous.end == range.start
    {
        previous.end = range.end;
    } else {
        parts.push(Part::Text(range));
    }
}
