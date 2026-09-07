mod text_part;

use text_part::{TextPart, text_parts};
use thai_syllable_rules::split;

#[test]
fn conflicting_combining_marks_cannot_become_a_syllable() {
    for source in ["ก่้", "กี้่", "กิี", "กุิ", "ก์์"] {
        let parts = split(source);
        assert_eq!(text_parts(source, &parts), vec![TextPart::Invalid(source)]);
    }
}

#[test]
fn a_mark_without_an_adjacent_base_does_not_claim_a_later_consonant() {
    let source = "กะ่ก";
    let parts = split(source);
    assert_eq!(
        text_parts(source, &parts),
        vec![
            TextPart::Syllable("กะ"),
            TextPart::Text("่"),
            TextPart::Undecided("ก"),
        ]
    );
}

#[test]
fn an_orphan_spacing_vowel_does_not_hide_stranded_consonants() {
    let source = "กีาก";
    let parts = split(source);
    assert_eq!(
        text_parts(source, &parts),
        vec![
            TextPart::Syllable("กี"),
            TextPart::Text("า"),
            TextPart::Undecided("ก"),
        ]
    );
}
