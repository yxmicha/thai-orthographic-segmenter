mod text_part;
use thai_syllable_rules::{Part, SyllablePattern, is_one_syllable_or_more, split};

#[test]
fn recognizes_complete_vowel_patterns_with_at_least_two_characters() {
    assert!(is_one_syllable_or_more("กา"));
    assert!(is_one_syllable_or_more("กิน"));
    assert!(!is_one_syllable_or_more("ก"));
    assert!(!is_one_syllable_or_more("ก้"));
}
