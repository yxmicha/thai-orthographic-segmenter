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

    /// Finds the longest matching prefix word that leaves either nothing or a text that can itself
    /// start with a dictionary word. Falls back to the longest match when none does.
    ///
    /// # Examples
    ///
    /// ```
    /// use thai_line_segmenter::icu_dictionary::IcuDictionary;
    ///
    /// let dictionary = IcuDictionary::new("ปฐม\nกาล\n")?;
    /// assert_eq!(dictionary.longest_word("ปฐมกาล"), Some("ปฐม"));
    /// assert_eq!(dictionary.longest_word("xyz"), None);
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    #[must_use]
    pub fn longest_word<'a>(&self, text: &'a str) -> Option<&'a str> {
        let mut buffer = [0usize; INLINE_MATCHES];
        let mut count = 0;
        let mut overflow = Vec::new();

        for (_, end_index) in self.cedar.common_prefix_iter(text) {
            if count < buffer.len() {
                buffer[count] = end_index;
                count += 1;
            } else if overflow.is_empty() {
                overflow.reserve(buffer.len() * 2);
                overflow.extend_from_slice(&buffer);
                overflow.push(end_index);
            } else {
                overflow.push(end_index);
            }
        }

        let slice = if overflow.is_empty() {
            &mut buffer[..count]
        } else {
            &mut overflow[..]
        };

        if slice.is_empty() {
            return None;
        }

        slice.sort_unstable_by(|left, right| right.cmp(left));
        let longest = slice[0];
        let end_index = slice
            .iter()
            .copied()
            .find(|&end_index| {
                let remaining_text = &text[end_index + 1..];
                remaining_text.is_empty()
                    || self
                        .cedar
                        .common_prefix_iter(remaining_text)
                        .any(|(_, end_index)| end_index > MINIMUM_FOLLOWING_WORD_END_INDEX)
            })
            .unwrap_or(longest);
        Some(&text[..=end_index])
    }
}
