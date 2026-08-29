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

#[test]
fn applies_the_carrier_rule_to_every_non_coda_base() {
    // These are all accepted Thai bases that cannot be Thai codas. The test
    // Each spelling is written out so every member of the structural class is
    // visible in the test and cannot be omitted by a future inventory change.
    assert_eq!(
        text_parts("ว่าฃอนด้า", &split("ว่าฃอนด้า")),
        vec![
            TextPart::Syllable("ว่า"),
            TextPart::Syllable("ฃอ"),
            TextPart::Undecided("น"),
            TextPart::Syllable("ด้า"),
        ]
    );
    assert_eq!(
        text_parts("ว่าฅอนด้า", &split("ว่าฅอนด้า")),
        vec![
            TextPart::Syllable("ว่า"),
            TextPart::Syllable("ฅอ"),
            TextPart::Undecided("น"),
            TextPart::Syllable("ด้า"),
        ]
    );
    assert_eq!(
        text_parts("ว่าฉอนด้า", &split("ว่าฉอนด้า")),
        vec![
            TextPart::Syllable("ว่า"),
            TextPart::Syllable("ฉอ"),
            TextPart::Undecided("น"),
            TextPart::Syllable("ด้า"),
        ]
    );
    assert_eq!(
        text_parts("ว่าฌอนด้า", &split("ว่าฌอนด้า")),
        vec![
            TextPart::Syllable("ว่า"),
            TextPart::Syllable("ฌอ"),
            TextPart::Undecided("น"),
            TextPart::Syllable("ด้า"),
        ]
    );
    assert_eq!(
        text_parts("ว่าผอนด้า", &split("ว่าผอนด้า")),
        vec![
            TextPart::Syllable("ว่า"),
            TextPart::Syllable("ผอ"),
            TextPart::Undecided("น"),
            TextPart::Syllable("ด้า"),
        ]
    );
    assert_eq!(
        text_parts("ว่าฝอนด้า", &split("ว่าฝอนด้า")),
        vec![
            TextPart::Syllable("ว่า"),
            TextPart::Syllable("ฝอ"),
            TextPart::Undecided("น"),
            TextPart::Syllable("ด้า"),
        ]
    );
    assert_eq!(
        text_parts("ว่าหอนด้า", &split("ว่าหอนด้า")),
        vec![
            TextPart::Syllable("ว่า"),
            TextPart::Syllable("หอ"),
            TextPart::Undecided("น"),
            TextPart::Syllable("ด้า"),
        ]
    );
    assert_eq!(
        text_parts("ว่าออนด้า", &split("ว่าออนด้า")),
        vec![
            TextPart::Syllable("ว่า"),
            TextPart::Syllable("ออ"),
            TextPart::Undecided("น"),
            TextPart::Syllable("ด้า"),
        ]
    );
    assert_eq!(
        text_parts("ว่าฮอนด้า", &split("ว่าฮอนด้า")),
        vec![
            TextPart::Syllable("ว่า"),
            TextPart::Syllable("ฮอ"),
            TextPart::Undecided("น"),
            TextPart::Syllable("ด้า"),
        ]
    );
}

#[test]
fn keeps_a_bare_carrier_prefix_after_short_i_ambiguous() {
    let source = "กิจอสัง";

    assert_eq!(
        text_parts(source, &split(source)),
        vec![
            TextPart::Syllable("กิ"),
            TextPart::Undecided("จ"),
            TextPart::Undecided("อ"),
            TextPart::Syllable("สัง"),
        ]
    );
}

#[test]
fn keeps_a_single_carrier_ambiguous_between_leading_vowel_candidates() {
    let source = "เจอแบ";

    assert_eq!(
        text_parts(source, &split(source)),
        vec![
            TextPart::Syllable("เจ"),
            TextPart::Undecided("อ"),
            TextPart::Syllable("แบ"),
        ]
    );
}

#[test]
fn keeps_a_tone_marked_glide_with_its_following_syllable() {
    assert_eq!(
        text_parts("รู้ว่า", &split("รู้ว่า")),
        vec![TextPart::Syllable("รู้"), TextPart::Syllable("ว่า")]
    );
}

#[test]
fn keeps_a_glide_ambiguous_before_a_following_syllable() {
    let source = "คราวนี้";

    assert_eq!(
        text_parts(source, &split(source)),
        vec![
            TextPart::Syllable("ครา"),
            TextPart::Undecided("ว"),
            TextPart::Syllable("นี้"),
        ]
    );
}

#[test]
fn keeps_a_y_glide_ambiguous_before_a_following_syllable() {
    let source = "ตายที่";

    assert_eq!(
        text_parts(source, &split(source)),
        vec![
            TextPart::Syllable("ตา"),
            TextPart::Undecided("ย"),
            TextPart::Syllable("ที่"),
        ]
    );
}

#[test]
fn keeps_a_glide_ambiguous_before_a_following_implicit_candidate() {
    let source = "คราวกมรา";

    assert_eq!(
        text_parts(source, &split(source)),
        vec![
            TextPart::Syllable("ครา"),
            TextPart::Undecided("ว"),
            TextPart::Undecided("ก"),
            TextPart::Undecided("ม"),
            TextPart::Syllable("รา"),
        ]
    );
}

#[test]
fn keeps_an_inherent_final_coda_ambiguous_before_a_following_carrier() {
    let source = "ไหนโอ";

    assert_eq!(
        text_parts(source, &split(source)),
        vec![
            TextPart::Syllable("ไห"),
            TextPart::Undecided("น"),
            TextPart::Syllable("โอ"),
        ]
    );
}

#[test]
fn keeps_a_ue_carrier_final_ambiguous_before_a_leading_vowel() {
    let source = "เหมือนแส";

    assert_eq!(
        text_parts(source, &split(source)),
        vec![
            TextPart::Syllable("เหมือ"),
            TextPart::Undecided("น"),
            TextPart::Syllable("แส"),
        ]
    );
}

#[test]
fn keeps_a_ue_carrier_final_ambiguous_before_another_leading_vowel() {
    let source = "เทือนแร";

    assert_eq!(
        text_parts(source, &split(source)),
        vec![
            TextPart::Syllable("เทือ"),
            TextPart::Undecided("น"),
            TextPart::Syllable("แร"),
        ]
    );
}

#[test]
fn keeps_a_ue_carrier_final_ambiguous_before_an_open_carrier_syllable() {
    let source = "เหมือนตอ";

    assert_eq!(
        text_parts(source, &split(source)),
        vec![
            TextPart::Syllable("เหมือ"),
            TextPart::Undecided("น"),
            TextPart::Syllable("ตอ"),
        ]
    );
}

#[test]
fn keeps_an_unmarked_carrier_final_ambiguous_before_a_cluster() {
    let source = "ลอกหนัง";

    assert_eq!(
        text_parts(source, &split(source)),
        vec![
            TextPart::Syllable("ลอ"),
            TextPart::Undecided("ก"),
            TextPart::Syllable("หนัง"),
        ]
    );
}
