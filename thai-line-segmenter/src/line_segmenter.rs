use crate::{
    Error, NON_BREAKING_SPACE, ZERO_WIDTH_JOINER, ZERO_WIDTH_SPACE, icu_dictionary::IcuDictionary,
};

const DEFAULT_ICU_DICTIONARY: &str = include_str!("../data/icu/th-icu.txt");
const ZERO_WIDTH_NON_JOINER: char = '\u{200C}';
const ZERO_WIDTH_NO_BREAK_SPACE: char = '\u{FEFF}';
const ZERO_WIDTH_CHARACTERS: [char; 4] = [
    ZERO_WIDTH_SPACE,
    ZERO_WIDTH_NON_JOINER,
    ZERO_WIDTH_JOINER,
    ZERO_WIDTH_NO_BREAK_SPACE,
];
const SPECIAL_SPACES: [char; 5] = [
    NON_BREAKING_SPACE,
    '\u{202F}',
    '\u{2009}',
    '\u{200A}',
    '\u{205F}',
];
