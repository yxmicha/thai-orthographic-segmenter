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

#[test]
fn does_not_merge_distinct_vowel_nuclei_into_one_syllable() {
    let cases = [
        (
            "พรรคสา",
            vec![
                TextPart::Syllable("พร"),
                TextPart::Undecided("ร"),
                TextPart::Undecided("ค"),
                TextPart::Syllable("สา"),
            ],
        ),
        (
            "และยัง",
            vec![TextPart::Syllable("และ"), TextPart::Syllable("ยัง")],
        ),
        (
            "เบียร์ลดลง",
            vec![
                TextPart::Syllable("เบียร์"),
                TextPart::Syllable("ลด"),
                TextPart::Syllable("ลง"),
            ],
        ),
        (
            "ใหญ่อา",
            vec![TextPart::Syllable("ใหญ่"), TextPart::Syllable("อา")],
        ),
        (
            "นาวา",
            vec![TextPart::Syllable("นา"), TextPart::Syllable("วา")],
        ),
        (
            "ตะวัน",
            vec![TextPart::Syllable("ตะ"), TextPart::Syllable("วัน")],
        ),
        (
            "ราวัณ",
            vec![TextPart::Syllable("รา"), TextPart::Syllable("วัณ")],
        ),
        (
            "สุวรร",
            vec![
                TextPart::Syllable("สุ"),
                TextPart::Undecided("ว"),
                TextPart::Syllable("รร"),
            ],
        ),
        (
            "อ้วนนน",
            vec![TextPart::Syllable("อ้วน"), TextPart::Syllable("นน")],
        ),
        (
            "ม้วนได้",
            vec![
                TextPart::Syllable("ม้ว"),
                TextPart::Undecided("น"),
                TextPart::Syllable("ได้"),
            ],
        ),
        (
            "ติพร",
            vec![TextPart::Syllable("ติ"), TextPart::Syllable("พร")],
        ),
        (
            "น้องหงส์",
            vec![TextPart::Syllable("น้อง"), TextPart::Syllable("หงส์")],
        ),
        (
            "ล่องหน",
            vec![
                TextPart::Syllable("ล่อ"),
                TextPart::Undecided("ง"),
                TextPart::Syllable("หน"),
            ],
        ),
        (
            "ขอนหงส์",
            vec![
                TextPart::Syllable("ขอ"),
                TextPart::Undecided("น"),
                TextPart::Syllable("หงส์"),
            ],
        ),
        (
            "กอนกรา",
            vec![
                TextPart::Syllable("กอ"),
                TextPart::Undecided("น"),
                TextPart::Undecided("ก"),
                TextPart::Syllable("รา"),
            ],
        ),
    ];

    for (source, expected) in cases {
        assert_eq!(text_parts(source, &split(source)), expected, "{source}");
    }
}
#[test]
fn attaches_tone_marked_onsets_to_the_following_vowel() {
    assert_eq!(
        text_parts("ช่วยป้องกัน", &split("ช่วยป้องกัน")),
        vec![
            TextPart::Syllable("ช่วย"),
            TextPart::Syllable("ป้อง"),
            TextPart::Syllable("กัน"),
        ]
    );
}
