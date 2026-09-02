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
