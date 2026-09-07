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

#[test]
fn incompatible_mark_roles_and_reversed_vowels_are_invalid() {
    // Each source spells its marks in the order the legality check must reject, so the escapes
    // keep them from being normalized by an editor or formatter.
    for source in [
        "\u{0e01}\u{0e48}\u{0e4c}",
        "\u{0e01}\u{0e47}\u{0e49}",
        "\u{0e01}\u{0e48}\u{0e34}",
        "\u{0e01}\u{0e38}\u{0e3a}",
        "\u{0e01}\u{0e48}\u{0e38}",
        "\u{0e01}\u{0e49}\u{0e39}",
        "\u{0e01}\u{0e4c}\u{0e38}",
    ] {
        let parts = split(source);
        assert_eq!(
            text_parts(source, &parts),
            vec![TextPart::Invalid(source)],
            "{source}"
        );
    }
}

#[test]
fn an_unpaired_preposed_vowel_is_preserved_as_text() {
    for source in ["เ", "แ", "โ", "ใ", "ไ", "เ่"] {
        let parts = split(source);
        assert_eq!(
            text_parts(source, &parts),
            vec![TextPart::Text(source)],
            "{source}"
        );
    }
}
