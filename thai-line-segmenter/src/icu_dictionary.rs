//! Longest-match dictionary for ICU Thai break patterns.

use cedarwood::Cedar;

use crate::Error;

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

impl IcuDictionary {
    /// Creates an ICU dictionary from newline-separated word entries. Fails for an entry that
    /// contains a NUL character.
    ///
    /// # Errors
    ///
    /// Returns [`Error::IcuDictionary`] when an entry contains a NUL character.
    ///
    /// # Examples
    ///
    /// ```
    /// use thai_line_segmenter::{Error, icu_dictionary::IcuDictionary};
    ///
    /// assert!(IcuDictionary::new("ปฐม\nกาล\n").is_ok());
    /// // A NUL character cannot be stored in the trie.
    /// assert!(matches!(IcuDictionary::new("a\0b"), Err(Error::IcuDictionary(_))));
    /// ```
    pub fn new(words: &str) -> Result<Self, Error> {
        let mut cedar = Cedar::new();
        for word in words.split('\n').filter(|word| !word.is_empty()) {
            cedar
                .update(word, ENTRY_VALUE)
                .map_err(|error| Error::IcuDictionary(Box::new(error)))?;
        }
        Ok(Self { cedar })
    }
}
