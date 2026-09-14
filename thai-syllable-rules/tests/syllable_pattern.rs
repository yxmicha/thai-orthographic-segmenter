mod text_part;

use text_part::{TextPart, text_parts};
use thai_syllable_rules::{Part, SyllablePattern, is_one_syllable_or_more, split};

#[test]
fn recognizes_complete_vowel_patterns_with_at_least_two_characters() {
    assert!(is_one_syllable_or_more("กา"));
    assert!(is_one_syllable_or_more("กิน"));
    assert!(!is_one_syllable_or_more("ก"));
    assert!(!is_one_syllable_or_more("ก้"));
}

#[test]
fn recognizes_two_consonants_as_an_implicit_vowel_pattern() {
    assert!(is_one_syllable_or_more("ตร"));
    assert!(is_one_syllable_or_more("กร"));
    assert_eq!(
        SyllablePattern::inspect("ตร").map(|pattern| pattern.vowel_pattern_count()),
        Some(0)
    );
}

#[test]
fn permits_a_leading_consonant_that_is_silent_before_the_following_pattern() {
    assert!(is_one_syllable_or_more("ขนาด"));
    assert!(is_one_syllable_or_more("กล้า"));
    assert!(is_one_syllable_or_more("มเห"));
}

#[test]
fn rejects_non_thai_and_orphan_vowel_material() {
    assert!(!is_one_syllable_or_more("abc"));
    assert!(!is_one_syllable_or_more("า"));
}

#[test]
fn does_not_emit_single_character_independent_vowels_as_syllables() {
    for source in ["ฤ", "ฦ"] {
        assert_eq!(
            text_parts(source, &split(source)),
            vec![TextPart::Undecided(source)]
        );
    }
}

#[test]
fn does_not_emit_contextual_parts_that_fail_the_minimum_check() {
    for source in ["กกตต้องติดคุก", "กระทรวงท่องเที่ยวและการกีฬาออกก่อนเลือกตั้ง"]
    {
        assert!(
            split(source).iter().all(|part| {
                !matches!(part, Part::Syllable(_)) || is_one_syllable_or_more(part.text(source))
            }),
            "source emitted a syllable that fails the minimum check: {source}"
        );
    }
}
