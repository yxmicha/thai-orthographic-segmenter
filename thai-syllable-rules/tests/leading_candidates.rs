#![allow(
    clippy::too_many_lines,
    reason = "fixture tests list their expected segmentations inline"
)]
mod text_part;

use text_part::{TextPart, text_parts};
use thai_syllable_rules::split;

#[test]
fn keeps_leading_vowel_words_together_at_the_start_of_a_run() {
    assert_eq!(
        text_parts("มเหสี", &split("มเหสี")),
        vec![TextPart::Syllable("มเห"), TextPart::Syllable("สี"),]
    );
    assert_eq!(
        text_parts("มเหศ", &split("มเหศ")),
        vec![TextPart::Syllable("มเหศ")]
    );
    assert_eq!(
        text_parts("มเหศวร", &split("มเหศวร")),
        vec![
            TextPart::Syllable("มเห"),
            TextPart::Undecided("ศ"),
            TextPart::Syllable("วร"),
        ]
    );
    assert_eq!(
        text_parts("มเหศวรี", &split("มเหศวรี")),
        vec![
            TextPart::Syllable("มเห"),
            TextPart::Undecided("ศ"),
            TextPart::Undecided("ว"),
            TextPart::Syllable("รี"),
        ]
    );
    assert_eq!(
        text_parts("มเหศักดิ์", &split("มเหศักดิ์")),
        vec![TextPart::Syllable("มเห"), TextPart::Syllable("ศักดิ์"),]
    );
    assert_eq!(
        text_parts("มเหสักข์", &split("มเหสักข์")),
        vec![TextPart::Syllable("มเห"), TextPart::Syllable("สักข์"),]
    );
    assert_eq!(
        text_parts("มเหนทร์", &split("มเหนทร์")),
        vec![TextPart::Syllable("มเห"), TextPart::Syllable("นทร์")]
    );
    assert_eq!(
        text_parts("มเหยงค์", &split("มเหยงค์")),
        vec![TextPart::Syllable("มเห"), TextPart::Syllable("ยงค์"),]
    );
    assert_eq!(
        text_parts("กเฬวร", &split("กเฬวร")),
        vec![TextPart::Syllable("กเฬ"), TextPart::Syllable("วร")]
    );
    assert_eq!(
        text_parts("กเฬวราก", &split("กเฬวราก")),
        vec![
            TextPart::Syllable("กเฬ"),
            TextPart::Undecided("ว"),
            TextPart::Syllable("ราก"),
        ]
    );
    assert_eq!(
        text_parts("กเรณุ", &split("กเรณุ")),
        vec![TextPart::Syllable("กเร"), TextPart::Syllable("ณุ"),]
    );
    assert_eq!(
        text_parts("กเรณุกา", &split("กเรณุกา")),
        vec![
            TextPart::Syllable("กเร"),
            TextPart::Syllable("ณุ"),
            TextPart::Syllable("กา"),
        ]
    );
    assert_eq!(
        text_parts("กเรนทร์", &split("กเรนทร์")),
        vec![TextPart::Syllable("กเรนทร์")]
    );
    assert_eq!(
        text_parts("กเบนทร์", &split("กเบนทร์")),
        vec![TextPart::Syllable("กเบนทร์")]
    );
}

#[test]
fn keeps_noncluster_h_onsets_separate_from_leading_bases() {
    let cases = [
        (
            "มหา",
            vec![TextPart::Undecided("ม"), TextPart::Syllable("หา")],
        ),
        (
            "ทหาร",
            vec![TextPart::Undecided("ท"), TextPart::Syllable("หาร")],
        ),
        (
            "รหัส",
            vec![TextPart::Undecided("ร"), TextPart::Syllable("หัส")],
        ),
        (
            "สหกรณ์",
            vec![
                TextPart::Undecided("ส"),
                TextPart::Undecided("ห"),
                TextPart::Syllable("กรณ์"),
            ],
        ),
        (
            "สหภาพ",
            vec![
                TextPart::Undecided("ส"),
                TextPart::Undecided("ห"),
                TextPart::Syllable("ภาพ"),
            ],
        ),
        (
            "สหาย",
            vec![TextPart::Undecided("ส"), TextPart::Syllable("หาย")],
        ),
        (
            "คหบดี",
            vec![
                TextPart::Undecided("ค"),
                TextPart::Undecided("ห"),
                TextPart::Undecided("บ"),
                TextPart::Syllable("ดี"),
            ],
        ),
        (
            "พหุ",
            vec![TextPart::Undecided("พ"), TextPart::Syllable("หุ")],
        ),
        (
            "พหูพจน์",
            vec![
                TextPart::Undecided("พ"),
                TextPart::Syllable("หู"),
                TextPart::Syllable("พจน์"),
            ],
        ),
        (
            "อหังการ",
            vec![
                TextPart::Undecided("อ"),
                TextPart::Syllable("หัง"),
                TextPart::Syllable("การ"),
            ],
        ),
    ];

    for (source, expected) in cases {
        assert_eq!(text_parts(source, &split(source)), expected, "{source}");
    }
}
