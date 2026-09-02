mod text_part;

use text_part::{TextPart, text_parts};
use thai_syllable_rules::split;

#[test]
fn closes_hard_syllable_endings_before_following_syllables() {
    let cases = [
        (
            "เป็นคน",
            vec![TextPart::Syllable("เป็น"), TextPart::Syllable("คน")],
        ),
        (
            "น็อคเอาท์",
            vec![TextPart::Syllable("น็อค"), TextPart::Syllable("เอาท์")],
        ),
        (
            "คืนดี",
            vec![TextPart::Syllable("คืน"), TextPart::Syllable("ดี")],
        ),
        (
            "เกินไป",
            vec![TextPart::Syllable("เกิน"), TextPart::Syllable("ไป")],
        ),
        (
            "กำหนด",
            vec![TextPart::Syllable("กำ"), TextPart::Syllable("หนด")],
        ),
        (
            "อำพราง",
            vec![TextPart::Syllable("อำ"), TextPart::Syllable("พราง")],
        ),
        (
            "ตัวการ",
            vec![TextPart::Syllable("ตัว"), TextPart::Syllable("การ")],
        ),
        (
            "เขาไป",
            vec![TextPart::Syllable("เขา"), TextPart::Syllable("ไป")],
        ),
        (
            "บัตรไทย",
            vec![
                TextPart::Syllable("บัต"),
                TextPart::Undecided("ร"),
                TextPart::Syllable("ไทย"),
            ],
        ),
        (
            "จักรยาน",
            vec![
                TextPart::Syllable("จัก"),
                TextPart::Undecided("ร"),
                TextPart::Syllable("ยาน"),
            ],
        ),
        (
            "มิตรภาพ",
            vec![
                TextPart::Syllable("มิ"),
                TextPart::Undecided("ต"),
                TextPart::Undecided("ร"),
                TextPart::Syllable("ภาพ"),
            ],
        ),
        (
            "สัตว์ป่า",
            vec![TextPart::Syllable("สัตว์"), TextPart::Syllable("ป่า")],
        ),
        (
            "จันทร์นี้",
            vec![TextPart::Syllable("จันทร์"), TextPart::Syllable("นี้")],
        ),
        (
            "ศุกร์นี้",
            vec![TextPart::Syllable("ศุกร์"), TextPart::Syllable("นี้")],
        ),
        (
            "ลักษณ์ดี",
            vec![TextPart::Syllable("ลักษณ์"), TextPart::Syllable("ดี")],
        ),
        (
            "เต็มหรอ",
            vec![TextPart::Syllable("เต็ม"), TextPart::Syllable("หรอ")],
        ),
    ];

    for (source, expected) in cases {
        assert_eq!(text_parts(source, &split(source)), expected, "{source}");
    }
}

#[test]
fn keeps_taikhu_final_ambiguous_before_a_following_syllable() {
    let source = "บล็อกหยุด";

    assert_eq!(
        text_parts(source, &split(source)),
        vec![
            TextPart::Syllable("บล็อ"),
            TextPart::Undecided("ก"),
            TextPart::Syllable("หยุด"),
        ]
    );
}

#[test]
fn keeps_garan_extension_after_an_inherent_final_vowel() {
    let source = "กำน์กะ";

    assert_eq!(
        text_parts(source, &split(source)),
        vec![TextPart::Syllable("กำน์"), TextPart::Syllable("กะ")]
    );
}
#[test]
fn closes_inherent_final_vowels_and_preserves_cluster_tail_ambiguity() {
    let cases = [
        (
            "กำกะ",
            vec![TextPart::Syllable("กำ"), TextPart::Syllable("กะ")],
        ),
        (
            "ไกกะ",
            vec![TextPart::Syllable("ไก"), TextPart::Syllable("กะ")],
        ),
        (
            "ใจกะ",
            vec![TextPart::Syllable("ใจ"), TextPart::Syllable("กะ")],
        ),
        (
            "เกากะ",
            vec![TextPart::Syllable("เกา"), TextPart::Syllable("กะ")],
        ),
        (
            "กรำกะ",
            vec![TextPart::Syllable("กรำ"), TextPart::Syllable("กะ")],
        ),
        (
            "ไตรกะ",
            vec![
                TextPart::Syllable("ไต"),
                TextPart::Undecided("ร"),
                TextPart::Syllable("กะ"),
            ],
        ),
        (
            "ใครกะ",
            vec![
                TextPart::Syllable("ใค"),
                TextPart::Undecided("ร"),
                TextPart::Syllable("กะ"),
            ],
        ),
        (
            "เกลากะ",
            vec![TextPart::Syllable("เกลา"), TextPart::Syllable("กะ")],
        ),
        (
            "ไกยกะ",
            vec![
                TextPart::Syllable("ไก"),
                TextPart::Undecided("ย"),
                TextPart::Syllable("กะ"),
            ],
        ),
        (
            "ไชยกะ",
            vec![
                TextPart::Syllable("ไช"),
                TextPart::Undecided("ย"),
                TextPart::Syllable("กะ"),
            ],
        ),
        ("ไกย", vec![TextPart::Syllable("ไกย")]),
        (
            "ไตรย์กะ",
            vec![TextPart::Syllable("ไตรย์"), TextPart::Syllable("กะ")],
        ),
        (
            "เสาร์กะ",
            vec![TextPart::Syllable("เสาร์"), TextPart::Syllable("กะ")],
        ),
    ];

    for (source, expected) in cases {
        assert_eq!(text_parts(source, &split(source)), expected, "{source}");
    }
}
#[test]
fn does_not_emit_a_single_consonant_after_a_hard_ending() {
    let source = "กะตกลมเตะ";

    assert_eq!(
        text_parts(source, &split(source)),
        vec![
            TextPart::Syllable("กะ"),
            TextPart::Syllable("ตก"),
            TextPart::Undecided("ล"),
            TextPart::Undecided("ม"),
            TextPart::Syllable("เตะ"),
        ]
    );
}
