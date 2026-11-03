//! Longest-match dictionary for ICU Thai break patterns.

use cedarwood::Cedar;

/// A following dictionary word must end beyond this byte index, which means it spans more than one
/// Thai character, for the preceding match to be accepted as a word boundary.
const MINIMUM_FOLLOWING_WORD_END_INDEX: usize = 3;

/// Value stored for every entry; `cedarwood` requires one, but matching only uses match lengths.
const ENTRY_VALUE: i32 = 0;
/// Dictionary matches of one prefix kept without allocating.
const INLINE_MATCHES: usize = 32;

/// Prefix trie dictionary using `cedarwood` for matching ICU break patterns.
///
/// # Examples
///
/// ```
/// use thai_line_segmenter::icu_dictionary::IcuDictionary;
///
/// let dictionary = IcuDictionary::new("ปฐม\nกาล\n")?;
/// assert_eq!(dictionary.longest_word("ปฐมกาล"), Some("ปฐม"));
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
#[derive(Debug)]
pub struct IcuDictionary {
    cedar: Cedar,
}

/// Dictionary words found at the start of a text and the text no dictionary word covers.
///
/// # Examples
///
/// ```
/// use thai_line_segmenter::icu_dictionary::IcuDictionary;
///
/// let dictionary = IcuDictionary::new("ปฐม\nกาล\n")?;
/// // When the words cover the whole text there is no remainder.
/// assert_eq!(dictionary.longest_words("ปฐมกาล").remainder, None);
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
#[derive(Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct IcuSegmentation<'a> {
    /// Consecutive dictionary words from the start of the text.
    pub words: Vec<&'a str>,
    /// Text after the last dictionary word, or `None` when the words cover the whole text.
    pub remainder: Option<&'a str>,
}
