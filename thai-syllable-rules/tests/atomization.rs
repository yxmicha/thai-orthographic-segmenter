mod text_part;

use text_part::{TextPart, text_parts};
use thai_syllable_rules::{Part, split};

#[test]
fn recognizes_vowel_bearing_thai_units_as_syllables() {
    let source = "นะคะ";
    let parts = split(source);

    assert!(matches!(
        parts.as_slice(),
        [Part::Syllable(_), Part::Syllable(_)]
    ));
    assert_eq!(
        text_parts(source, &parts),
        vec![TextPart::Syllable("นะ"), TextPart::Syllable("คะ")]
    );
}

#[test]
fn separates_adjacent_complete_spoken_syllables() {
    let cases = [
        (
            "และอุณ",
            vec![TextPart::Syllable("และ"), TextPart::Syllable("อุณ")],
        ),
        (
            "ติของฮอร์",
            vec![
                TextPart::Syllable("ติ"),
                TextPart::Undecided("ข"),
                TextPart::Undecided("อ"),
                TextPart::Undecided("ง"),
                TextPart::Syllable("ฮอร์"),
            ],
        ),
    ];

    for (source, expected) in cases {
        assert_eq!(text_parts(source, &split(source)), expected, "{source}");
    }
}
