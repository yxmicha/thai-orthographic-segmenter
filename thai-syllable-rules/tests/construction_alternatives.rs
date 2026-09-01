mod text_part;

use text_part::{TextPart, text_parts};
use thai_syllable_rules::split;

#[test]
fn completes_terminal_pairs_after_leading_vowels() {
    let cases = [
        (
            "แสดง",
            vec![TextPart::Syllable("แส"), TextPart::Syllable("ดง")],
        ),
        (
            "เสมอ",
            vec![TextPart::Syllable("เส"), TextPart::Syllable("มอ")],
        ),
        (
            "เฉลย",
            vec![TextPart::Syllable("เฉ"), TextPart::Syllable("ลย")],
        ),
        (
            "แถลง",
            vec![TextPart::Syllable("แถ"), TextPart::Syllable("ลง")],
        ),
    ];

    for (source, expected) in cases {
        assert_eq!(text_parts(source, &split(source)), expected, "{source}");
    }
}

#[test]
fn keeps_compound_e_ao_rimes_as_positive_constructions() {
    for source in ["เฉพาะ", "เสนาะ", "เรพาะ"] {
        assert_eq!(
            text_parts(source, &split(source)),
            vec![TextPart::Syllable(source)],
            "{source}"
        );
    }
}

#[test]
fn completes_a_terminal_bare_pair_after_short_i() {
    assert_eq!(
        text_parts("ลิตร", &split("ลิตร")),
        vec![TextPart::Syllable("ลิ"), TextPart::Syllable("ตร"),],
    );
}
