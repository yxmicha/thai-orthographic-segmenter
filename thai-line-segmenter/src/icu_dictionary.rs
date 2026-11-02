//! Longest-match dictionary for ICU Thai break patterns.


/// A following dictionary word must end beyond this byte index, which means it spans more than one
/// Thai character, for the preceding match to be accepted as a word boundary.
const MINIMUM_FOLLOWING_WORD_END_INDEX: usize = 3;
