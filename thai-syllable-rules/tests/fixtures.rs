#![allow(
    clippy::too_many_lines,
    reason = "fixture tests list their expected segmentations inline"
)]
mod text_part;

use text_part::{TextPart, text_parts};
use thai_syllable_rules::split;

#[test]
fn covers_combined_vowel_and_silent_mark_fixture() {
    let source = "พระศาสตร์ศิษย์ศรีลึกซึ้งคลื่นพุทธศูนย์เป๊ะเพศแฉะแกล้งพจน์โจทย์เคราะห์ฟอร์มเปรอะเทิร์นเปี๊ยะเหรียญเจือะเหนื่อยผลัวะช่วงกำพร้าใหญ่ไหม้เศร้าฤทธิ์ฤๅษีรฦกฦๅชา";
    assert_eq!(
        text_parts(source, &split(source)),
        vec![
            TextPart::Syllable("พระ"),
            TextPart::Syllable("ศาสตร์"),
            TextPart::Syllable("ศิษย์"),
            TextPart::Syllable("ศรี"),
            TextPart::Syllable("ลึ"),
            TextPart::Undecided("ก"),
            TextPart::Syllable("ซึ้"),
            TextPart::Undecided("ง"),
            TextPart::Undecided("ค"),
            TextPart::Syllable("ลื่"),
            TextPart::Undecided("น"),
            TextPart::Syllable("พุ"),
            TextPart::Undecided("ท"),
            TextPart::Undecided("ธ"),
            TextPart::Syllable("ศูนย์"),
            TextPart::Syllable("เป๊ะ"),
            TextPart::Syllable("เพ"),
            TextPart::Undecided("ศ"),
            TextPart::Syllable("แฉะ"),
            TextPart::Syllable("แกล้"),
            TextPart::Undecided("ง"),
            TextPart::Undecided("พ"),
            TextPart::Syllable("จน์"),
            TextPart::Syllable("โจทย์"),
            TextPart::Syllable("เคราะห์"),
            TextPart::Syllable("ฟอร์ม"),
            TextPart::Syllable("เปรอะ"),
            TextPart::Syllable("เทิร์น"),
            TextPart::Syllable("เปี๊ยะ"),
            TextPart::Syllable("เหรีย"),
            TextPart::Undecided("ญ"),
            TextPart::Syllable("เจือะ"),
            TextPart::Syllable("เหนื่อย"),
            TextPart::Syllable("ผลัวะ"),
            TextPart::Syllable("ช่วง"),
            TextPart::Syllable("กำ"),
            TextPart::Syllable("พร้า"),
            TextPart::Syllable("ใหญ่"),
            TextPart::Syllable("ไหม้"),
            TextPart::Syllable("เศร้า"),
            TextPart::Syllable("ฤทธิ์"),
            TextPart::Syllable("ฤๅ"),
            TextPart::Syllable("ษี"),
            TextPart::Syllable("รฦ"),
            TextPart::Undecided("ก"),
            TextPart::Syllable("ฦๅ"),
            TextPart::Syllable("ชา"),
        ],
    );
}
