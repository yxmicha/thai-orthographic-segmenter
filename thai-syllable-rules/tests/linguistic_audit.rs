mod text_part;
use text_part::{TextPart, text_parts};
use thai_syllable_rules::{SyllablePattern, split};

#[test]
fn bare_carrier_does_not_license_an_onset_only_final() {
    for tail in ["ห", "ฮ", "ฉ", "ฌ", "ผ", "ฝ"] {
        let source = format!("อ{tail}");
        assert_eq!(
            text_parts(&source, &split(&source)),
            vec![TextPart::Undecided("อ"), TextPart::Undecided(tail)],
            "{source}"
        );
    }
    for source in ["อก", "อน", "ออ", "อย่า", "อยู่", "อย่าง", "อยาก"]
    {
        assert_eq!(
            text_parts(source, &split(source)),
            vec![TextPart::Syllable(source)],
            "{source}"
        );
    }
}

#[test]
fn retained_oe_carrier_is_one_vowel_pattern() {
    for source in ["เทอม", "เทอญ"] {
        assert_eq!(
            SyllablePattern::inspect(source).map(|pattern| pattern.vowel_pattern_count()),
            Some(1),
            "{source}"
        );
        assert_eq!(
            text_parts(source, &split(source)),
            vec![TextPart::Syllable(source)],
            "{source}"
        );
    }
}

#[test]
fn lakkhangyao_requires_an_independent_vowel() {
    assert_eq!(
        text_parts("กๅก", &split("กๅก")),
        vec![
            TextPart::Undecided("ก"),
            TextPart::Text("ๅ"),
            TextPart::Undecided("ก")
        ]
    );
    for source in ["ฤๅ", "ฦๅ"] {
        assert_eq!(
            text_parts(source, &split(source)),
            vec![TextPart::Syllable(source)]
        );
    }
}
