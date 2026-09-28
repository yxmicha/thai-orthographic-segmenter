use crate::Error;

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
    /// Loads the bundled inventories and rejects empty, unsorted, or malformed entries.
    pub(crate) fn bundled() -> Result<Self, Error> {
        let characters = BUNDLED_CHARACTERS
            .lines()
            .map(|line| {
                let mut characters = line.chars();
                match (characters.next(), characters.next()) {
                    (Some(character), None) => Ok(character),
                    _ => Err(Error::Invariant(
                        "character vocabulary lines must hold one character",
                    )),
                }
            })
            .collect::<Result<Vec<_>, Error>>()?;
        if characters.is_empty() || characters.windows(2).any(|pair| pair[0] >= pair[1]) {
            return Err(Error::Invariant(
                "character vocabulary must be nonempty, unique, and sorted",
            ));
        }
        let ngrams = BUNDLED_NGRAMS.lines().collect::<Vec<_>>();
        if ngrams.windows(2).any(|pair| pair[0] >= pair[1])
            || ngrams
                .iter()
                .any(|gram| !(2..=3).contains(&gram.chars().count()))
        {
            return Err(Error::Invariant(
                "n-gram vocabulary must be sorted, unique bigrams/trigrams",
            ));
        }
        let mut hash = FNV_OFFSET_BASIS;
        let character_bytes = characters
            .iter()
            .flat_map(|character| u32::from(*character).to_le_bytes());
        let ngram_bytes = ngrams
            .iter()
            .flat_map(|gram| gram.bytes().chain(std::iter::once(0)));
        for byte in character_bytes.chain(ngram_bytes) {
            hash = (hash ^ u64::from(byte)).wrapping_mul(FNV_PRIME);
        }
        let mut bigrams = Vec::new();
        let mut trigrams = Vec::new();
        for (index, gram) in ngrams.iter().enumerate() {
            let window = gram.chars().collect::<Vec<_>>();
            let entry = (Self::pack(&window), index + 1);
            if window.len() == 2 {
                bigrams.push(entry);
            } else {
                trigrams.push(entry);
            }
        }
        bigrams.sort_unstable();
        trigrams.sort_unstable();
        Ok(Self {
            characters,
            ngrams,
            fingerprint: hash.max(1),
            bigrams,
            trigrams,
        })
    }

    /// Packs up to three characters, 21 bits each, so equal windows have equal keys.
    fn pack(window: &[char]) -> u64 {
        window.iter().fold(0, |key, character| {
            (key << 21) | u64::from(u32::from(*character))
        })
    }

    /// Character embedding rows, including the unknown row.
    pub(crate) fn character_rows(&self) -> usize {
        self.characters.len() + 1
    }

    /// Stable hash of the exact character and n-gram row mapping.
    pub(crate) fn fingerprint(&self) -> u64 {
        self.fingerprint
    }
}
