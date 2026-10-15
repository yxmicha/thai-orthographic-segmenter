//! Longest accepted matches with source-syllable alignment.


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

#[derive(Default)]
struct Node {
    children: BTreeMap<char, usize>,
    terminal: bool,
}
