mod text_part;

use text_part::{TextPart, text_parts};
use thai_syllable_rules::split;

#[test]
fn attaches_an_onset_only_singleton_to_a_following_cluster() {
    let cases = [
        (
            "แกหรอก",
            vec![TextPart::Syllable("แก"), TextPart::Syllable("หรอก")],
        ),
        (
            "อีกหลาย",
            vec![TextPart::Syllable("อีก"), TextPart::Syllable("หลาย")],
        ),
        (
            "อกหลาย",
            vec![TextPart::Syllable("อก"), TextPart::Syllable("หลาย")],
        ),
    ];

    for (source, expected) in cases {
        assert_eq!(text_parts(source, &split(source)), expected, "{source}");
    }
}

#[test]
fn preserves_r_attachment_before_a_following_carrier() {
    let cases = [
        (
            "กุรอก",
            vec![
                TextPart::Syllable("กุ"),
                TextPart::Undecided("ร"),
                TextPart::Syllable("อก"),
            ],
        ),
        (
            "กีรอก",
            vec![
                TextPart::Syllable("กี"),
                TextPart::Undecided("ร"),
                TextPart::Syllable("อก"),
            ],
        ),
        (
            "จมรรอก",
            vec![
                TextPart::Syllable("จม"),
                TextPart::Undecided("ร"),
                TextPart::Undecided("ร"),
                TextPart::Syllable("อก"),
            ],
        ),
        (
            "ตอนนอนสิ",
            vec![
                TextPart::Syllable("ตอ"),
                TextPart::Undecided("น"),
                TextPart::Syllable("นอ"),
                TextPart::Undecided("น"),
                TextPart::Syllable("สิ"),
            ],
        ),
    ];

    for (source, expected) in cases {
        assert_eq!(text_parts(source, &split(source)), expected, "{source}");
    }
}

#[test]
fn attaches_a_carrier_to_the_following_pattern_but_preserves_left_ambiguity() {
    let following = "ขออนุญาต";
    assert_eq!(
        text_parts(following, &split(following)),
        vec![
            TextPart::Syllable("ขอ"),
            TextPart::Syllable("อนุ"),
            TextPart::Syllable("ญาต"),
        ]
    );

    let leading = "เธอก็ไม่";
    assert_eq!(
        text_parts(leading, &split(leading)),
        vec![
            TextPart::Syllable("เธ"),
            TextPart::Undecided("อ"),
            TextPart::Syllable("ก็"),
            TextPart::Syllable("ไม่"),
        ]
    );
}

#[test]
fn treats_standalone_ko_taikhu_as_a_syllable_before_a_carrier() {
    let source = "ก็รอ";

    assert_eq!(
        text_parts(source, &split(source)),
        vec![TextPart::Syllable("ก็"), TextPart::Syllable("รอ")],
    );

    let started_before = "เก็บกา";
    assert_eq!(
        text_parts(started_before, &split(started_before)),
        vec![TextPart::Syllable("เก็บ"), TextPart::Syllable("กา")],
    );
}

#[test]
fn keeps_a_bare_carrier_and_final_base_as_one_implicit_syllable() {
    let cases = [
        ("อน", vec![TextPart::Syllable("อน")]),
        (
            "ขออน",
            vec![TextPart::Syllable("ขอ"), TextPart::Syllable("อน")],
        ),
        (
            "อนา",
            vec![TextPart::Undecided("อ"), TextPart::Syllable("นา")],
        ),
        (
            "อนุ",
            vec![TextPart::Undecided("อ"), TextPart::Syllable("นุ")],
        ),
    ];

    for (source, expected) in cases {
        assert_eq!(text_parts(source, &split(source)), expected, "{source}");
    }
}

#[test]
fn keeps_a_complete_bare_carrier_candidate_before_an_ordinary_syllable() {
    let source = "มาลองใช้";

    assert_eq!(
        text_parts(source, &split(source)),
        vec![
            TextPart::Syllable("มา"),
            TextPart::Undecided("ล"),
            TextPart::Syllable("อง"),
            TextPart::Syllable("ใช้"),
        ]
    );
}

#[test]
fn leaves_cluster_attachment_explicit_without_invalidating_the_syllables() {
    let source = "ทุกรุ่น";
    let parts = split(source);

    assert_eq!(
        text_parts(source, &parts),
        vec![
            TextPart::Syllable("ทุ"),
            TextPart::Undecided("ก"),
            TextPart::Syllable("รุ่น"),
        ]
    );
}
#[test]
fn keeps_ue_carrier_with_the_vowel_syllable() {
    let source = "ชื่อโลทา";
    let parts = split(source);

    assert_eq!(
        text_parts(source, &parts),
        vec![
            TextPart::Syllable("ชื่อ"),
            TextPart::Syllable("โล"),
            TextPart::Syllable("ทา"),
        ]
    );
}

#[test]
fn does_not_consume_a_carrier_that_starts_the_next_rime() {
    let source = "ซื้ออิน";

    assert_eq!(
        text_parts(source, &split(source)),
        vec![TextPart::Syllable("ซื้อ"), TextPart::Syllable("อิน")]
    );
}

#[test]
fn recognizes_common_vowels_and_initial_clusters() {
    let cases = [
        (
            "เกรด",
            vec![TextPart::Syllable("เก"), TextPart::Syllable("รด")],
        ),
        ("กล้า", vec![TextPart::Syllable("กล้า")]),
        ("ขวัญ", vec![TextPart::Syllable("ขวัญ")]),
        ("ไทย", vec![TextPart::Syllable("ไทย")]),
        ("เครื่อง", vec![TextPart::Syllable("เครื่อง")]),
        (
            "เซรั่ม",
            vec![TextPart::Syllable("เซ"), TextPart::Syllable("รั่ม")],
        ),
        ("ชื่อ", vec![TextPart::Syllable("ชื่อ")]),
    ];

    for (source, expected) in cases {
        assert_eq!(text_parts(source, &split(source)), expected, "{source}");
    }
}
#[test]
fn recognizes_the_extended_initial_cluster_inventory() {
    let cases = [
        (
            "กะซรา",
            vec![TextPart::Syllable("กะ"), TextPart::Syllable("ซรา")],
        ),
        (
            "กะฟรา",
            vec![TextPart::Syllable("กะ"), TextPart::Syllable("ฟรา")],
        ),
        (
            "กะฟลา",
            vec![TextPart::Syllable("กะ"), TextPart::Syllable("ฟลา")],
        ),
        (
            "กะอยา",
            vec![TextPart::Syllable("กะ"), TextPart::Syllable("อยา")],
        ),
        (
            "กะกฤ",
            vec![TextPart::Syllable("กะ"), TextPart::Syllable("กฤ")],
        ),
        (
            "กะตฤ",
            vec![TextPart::Syllable("กะ"), TextPart::Syllable("ตฤ")],
        ),
        (
            "กะปฤ",
            vec![TextPart::Syllable("กะ"), TextPart::Syllable("ปฤ")],
        ),
        (
            "กะพฤ",
            vec![TextPart::Syllable("กะ"), TextPart::Syllable("พฤ")],
        ),
        (
            "กะทฤ",
            vec![TextPart::Syllable("กะ"), TextPart::Syllable("ทฤ")],
        ),
    ];

    for (source, expected) in cases {
        assert_eq!(text_parts(source, &split(source)), expected, "{source}");
    }
}
