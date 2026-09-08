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

#[test]
fn rejecting_a_cell_does_not_strand_its_preposed_vowel() {
    let source = "เกิี";
    let parts = split(source);
    assert_eq!(text_parts(source, &parts), vec![TextPart::Invalid(source)]);
}

#[test]
fn valid_stacks_and_decomposed_sara_am_keep_the_original_spelling() {
    for source in ["กี่", "กุ้ง", "กู้", "น้ำ", "นํ้า", "พันธุ์", "ค่าา"]
    {
        let parts = split(source);
        assert_eq!(
            text_parts(source, &parts),
            vec![TextPart::Syllable(source)],
            "{source}"
        );
    }
}

#[test]
fn illegal_cells_are_local_barriers_with_exact_utf8_ranges() {
    let source = "🦀กา กี้่ลิตร แสดง ่ก้บอก.";
    let parts = split(source);
    assert_eq!(
        text_parts(source, &parts),
        vec![
            TextPart::Text("🦀"),
            TextPart::Syllable("กา"),
            TextPart::Text(" "),
            TextPart::Invalid("กี้่"),
            TextPart::Syllable("ลิ"),
            TextPart::Syllable("ตร"),
            TextPart::Text(" "),
            TextPart::Syllable("แส"),
            TextPart::Syllable("ดง"),
            TextPart::Text(" ่"),
            TextPart::Syllable("ก้บ"),
            TextPart::Syllable("อก"),
            TextPart::Dot("."),
        ]
    );
    let mut end = 0;
    for part in &parts {
        let range = part.byte_range();
        assert_eq!(range.start, end);
        assert!(range.start < range.end);
        assert!(source.is_char_boundary(range.start) && source.is_char_boundary(range.end));
        end = range.end;
    }
    assert_eq!(end, source.len());
}
