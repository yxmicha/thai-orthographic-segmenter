const BUNDLED_CHARACTERS: &str = include_str!("../model/characters.txt");
const BUNDLED_NGRAMS: &str = include_str!("../model/ngrams.txt");
const FNV_OFFSET_BASIS: u64 = 0xcbf2_9ce4_8422_2325;
const FNV_PRIME: u64 = 0x0100_0000_01b3;

/// Character and n-gram vocabulary with a stable unknown row and fingerprint.
///
/// Row zero is the unknown entry; entry order fixes every other embedding row.
#[derive(Clone, Debug)]
pub(crate) struct CharacterVocabulary {
    pub(crate) characters: Vec<char>,
    pub(crate) ngrams: Vec<&'static str>,
    fingerprint: u64,
    /// `(packed characters, row)` of every bigram and trigram, sorted by packed key.
    bigrams: Vec<(u64, usize)>,
    trigrams: Vec<(u64, usize)>,
}

impl CharacterVocabulary {

    /// Character embedding rows, including the unknown row.
    pub(crate) fn character_rows(&self) -> usize {
        self.characters.len() + 1
    }

    /// Stable hash of the exact character and n-gram row mapping.
    pub(crate) fn fingerprint(&self) -> u64 {
        self.fingerprint
    }
}
