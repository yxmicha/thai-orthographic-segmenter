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

#[test]
fn keeps_short_i_coda_and_following_carrier_attachments_explicit() {
    let source = "วิฟอยู่";

    assert_eq!(
        text_parts(source, &split(source)),
        vec![
            TextPart::Syllable("วิ"),
            TextPart::Undecided("ฟ"),
            TextPart::Undecided("อ"),
            TextPart::Syllable("ยู่"),
        ]
    );
}

#[test]
fn keeps_a_coda_ambiguous_after_an_onset_only_carrier_syllable() {
    let source = "ว่าฮอนด้า";

    assert_eq!(
        text_parts(source, &split(source)),
        vec![
            TextPart::Syllable("ว่า"),
            TextPart::Syllable("ฮอ"),
            TextPart::Undecided("น"),
            TextPart::Syllable("ด้า"),
        ]
    );
}
