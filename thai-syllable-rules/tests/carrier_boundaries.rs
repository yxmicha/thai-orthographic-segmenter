mod text_part;

use text_part::{TextPart, text_parts};
use thai_syllable_rules::split;

#[test]
fn leaves_a_garan_trailing_base_undecided_before_an_open_carrier() {
    let source = "หงษ์ทอง";

    assert_eq!(
        text_parts(source, &split(source)),
        vec![
            TextPart::Syllable("หงษ์"),
            TextPart::Undecided("ท"),
            TextPart::Syllable("อง"),
        ]
    );
}

#[test]
fn attaches_a_carrier_after_a_tone_marked_base_before_an_ambiguous_glide() {
    let source = "อร่อยที่";

    assert_eq!(
        text_parts(source, &split(source)),
        vec![
            TextPart::Syllable("อร่อ"),
            TextPart::Undecided("ย"),
            TextPart::Syllable("ที่"),
        ]
    );
}

#[test]
fn keeps_a_tone_marked_carrier_glide_ambiguous_before_a_following_syllable() {
    let source = "ค่อยไป";

    assert_eq!(
        text_parts(source, &split(source)),
        vec![
            TextPart::Syllable("ค่อ"),
            TextPart::Undecided("ย"),
            TextPart::Syllable("ไป"),
        ]
    );
}

#[test]
fn keeps_a_tone_marked_carrier_final_ambiguous_before_a_leading_vowel() {
    let source = "ห้องเด้อะ";

    assert_eq!(
        text_parts(source, &split(source)),
        vec![
            TextPart::Syllable("ห้อ"),
            TextPart::Undecided("ง"),
            TextPart::Syllable("เด้อะ"),
        ]
    );
}
