//! Longest accepted matches with source-syllable alignment.

use std::ops::Range;
use std::{collections::BTreeMap, io::BufRead};

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
}

#[derive(Default)]
struct Node {
    children: BTreeMap<char, usize>,
    terminal: bool,
}
