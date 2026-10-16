//! Longest accepted matches with source-syllable alignment.

use std::ops::Range;
use std::{collections::BTreeMap, io::BufRead};

use crate::Error;
use thai_syllable_rules::ThaiScalar;

/// Reusable lexical trie; selection is leftmost-longest with no overlapping chains.
///
/// # Examples
///
/// ```
/// use thai_word_segmenter::dictionary::ClusterDictionary;
///
/// let dictionary = ClusterDictionary::read("ปฐมกาล\nพระวาทะ\n".as_bytes())?;
/// let syllables = ["ใน", "ปฐม", "กาล", "พระ", "วา", "ทะ"];
/// assert_eq!(dictionary.longest_matches(&syllables), [1..3, 3..6]);
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
pub struct ClusterDictionary {
    nodes: Vec<Node>,
    entries: usize,
}

impl std::fmt::Debug for ClusterDictionary {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("ClusterDictionary")
            .finish_non_exhaustive()
    }
}

impl ClusterDictionary {
    /// Reads one Thai-only accepted cluster per line, retaining short and long forms.
    ///
    /// # Errors
    ///
    /// Returns [`Error::DictionaryRead`] when a line cannot be read, or
    /// [`Error::DictionaryEntry`] when an entry is empty or contains non-Thai characters.
    ///
    /// # Examples
    ///
    /// ```
    /// use thai_word_segmenter::{Error, dictionary::ClusterDictionary};
    ///
    /// assert_eq!(ClusterDictionary::read("ปฐมกาล\n".as_bytes())?.len(), 1);
    /// assert!(matches!(
    ///     ClusterDictionary::read("abc\n".as_bytes()),
    ///     Err(Error::DictionaryEntry(_))
    /// ));
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn read(reader: impl BufRead) -> Result<Self, Error> {
        let mut result = Self {
            nodes: vec![Node::default()],
            entries: 0,
        };
        for line in reader.lines() {
            let word = line.map_err(Error::DictionaryRead)?;
            if word.is_empty()
                || !word
                    .chars()
                    .all(|character| ThaiScalar::new(character).is_accepted())
            {
                return Err(Error::DictionaryEntry(word));
            }
            let mut node = 0;
            for character in word.chars() {
                let next = if let Some(next) = result.nodes[node].children.get(&character).copied()
                {
                    next
                } else {
                    let next = result.nodes.len();
                    result.nodes.push(Node::default());
                    result.nodes[node].children.insert(character, next);
                    next
                };
                node = next;
            }
            if !result.nodes[node].terminal {
                result.nodes[node].terminal = true;
                result.entries += 1;
            }
        }
        Ok(result)
    }

    /// Returns the number of unique accepted entries.
    ///
    /// # Examples
    ///
    /// ```
    /// use thai_word_segmenter::dictionary::ClusterDictionary;
    ///
    /// assert_eq!(ClusterDictionary::read("ปฐมกาล\nพระวาทะ\n".as_bytes())?.len(), 2);
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    #[must_use]
    pub const fn len(&self) -> usize {
        self.entries
    }

    /// Selects nonoverlapping leftmost-longest matches in syllable coordinates.
    ///
    /// # Examples
    ///
    /// ```
    /// use thai_word_segmenter::dictionary::ClusterDictionary;
    ///
    /// let dictionary = ClusterDictionary::read("ปฐมกาล\n".as_bytes())?;
    /// // Ranges are syllable indices.
    /// assert_eq!(dictionary.longest_matches(&["ใน", "ปฐม", "กาล"]), [1..3]);
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn longest_matches<S: AsRef<str>>(&self, syllables: &[S]) -> Vec<Range<usize>> {
        let mut matches = Vec::new();
        let mut start = 0;
        while start < syllables.len() {
            let mut node = 0;
            let mut longest = None;
            'search: for (end, syllable) in syllables.iter().enumerate().skip(start) {
                for character in syllable.as_ref().chars() {
                    let Some(next) = self.nodes[node].children.get(&character).copied() else {
                        break 'search;
                    };
                    node = next;
                }
                if self.nodes[node].terminal {
                    longest = Some(end + 1);
                }
            }
            match longest {
                Some(end) => {
                    matches.push(start..end);
                    start = end;
                }
                None => start += 1,
            }
        }
        matches
    }

    /// Marks syllable gaps that belong inside selected accepted clusters.
    ///
    /// # Examples
    ///
    /// ```
    /// use thai_word_segmenter::dictionary::ClusterDictionary;
    ///
    /// let dictionary = ClusterDictionary::read("ปฐมกาล\n".as_bytes())?;
    /// // Gap `i` follows syllable `i`; only the gap inside the match is protected.
    /// assert_eq!(dictionary.protected_gaps(&["ใน", "ปฐม", "กาล"]), [false, true, false]);
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn protected_gaps<S: AsRef<str>>(&self, syllables: &[S]) -> Vec<bool> {
        let mut protected = vec![false; syllables.len()];
        for span in self.longest_matches(syllables) {
            protected[span.start..span.end.saturating_sub(1)].fill(true);
        }
        protected
    }
}

#[derive(Default)]
struct Node {
    children: BTreeMap<char, usize>,
    terminal: bool,
}
