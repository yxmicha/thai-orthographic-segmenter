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
#[test]
fn keeps_following_onsets_and_dependent_marks_structurally_attached() {
    assert_eq!(
        text_parts("สาวคนล่าสุด", &split("สาวคนล่าสุด")),
        vec![
            TextPart::Syllable("สา"),
            TextPart::Undecided("ว"),
            TextPart::Undecided("ค"),
            TextPart::Undecided("น"),
            TextPart::Syllable("ล่า"),
            TextPart::Syllable("สุด"),
        ]
    );
    assert_eq!(
        text_parts("เสนอข่าวที่บริษัท", &split("เสนอข่าวที่บริษัท")),
        vec![
            TextPart::Syllable("เส"),
            TextPart::Undecided("น"),
            TextPart::Undecided("อ"),
            TextPart::Syllable("ข่า"),
            TextPart::Undecided("ว"),
            TextPart::Syllable("ที่"),
            TextPart::Undecided("บ"),
            TextPart::Syllable("ริ"),
            TextPart::Syllable("ษัท"),
        ]
    );
}
#[test]
fn keeps_short_vowel_codas_and_separates_independent_vowel_patterns() {
    let cases = [
        (
            "อสัง",
            vec![TextPart::Undecided("อ"), TextPart::Syllable("สัง")],
        ),
        (
            "ตะวันระยะราวัณ",
            vec![
                TextPart::Syllable("ตะ"),
                TextPart::Syllable("วัน"),
                TextPart::Syllable("ระ"),
                TextPart::Syllable("ยะ"),
                TextPart::Syllable("รา"),
                TextPart::Syllable("วัณ"),
            ],
        ),
    ];

    for (source, expected) in cases {
        assert_eq!(text_parts(source, &split(source)), expected, "{source}");
    }
}
#[test]
fn leaves_ambiguous_consonant_attachment_explicit() {
    let source = "อนุญาตนะคะ";
    let parts = split(source);

    assert_eq!(
        text_parts(source, &parts),
        vec![
            TextPart::Undecided("อ"),
            TextPart::Syllable("นุ"),
            TextPart::Syllable("ญา"),
            TextPart::Undecided("ต"),
            TextPart::Syllable("นะ"),
            TextPart::Syllable("คะ"),
        ]
    );
}

#[test]
fn does_not_leave_a_leading_consonant_without_a_preceding_candidate() {
    let source = "อนญาตนะคะ";
    let parts = split(source);

    assert_eq!(
        text_parts(source, &parts),
        vec![
            TextPart::Syllable("อน"),
            TextPart::Syllable("ญา"),
            TextPart::Undecided("ต"),
            TextPart::Syllable("นะ"),
            TextPart::Syllable("คะ"),
        ]
    );
}
#[test]
fn keeps_silent_mark_runs_with_the_preceding_syllable() {
    let cases = [
        (
            "ฟิล์มและ",
            vec![TextPart::Syllable("ฟิล์ม"), TextPart::Syllable("และ")],
        ),
        (
            "โฟล์คและ",
            vec![TextPart::Syllable("โฟล์ค"), TextPart::Syllable("และ")],
        ),
        (
            "เสน่ห์และ",
            vec![TextPart::Syllable("เสน่ห์"), TextPart::Syllable("และ")],
        ),
        (
            "การ์ดา",
            vec![TextPart::Syllable("การ์"), TextPart::Syllable("ดา")],
        ),
        (
            "อาร์ตี้",
            vec![TextPart::Syllable("อาร์"), TextPart::Syllable("ตี้")],
        ),
        (
            "ซอฟต์แวร์",
            vec![TextPart::Syllable("ซอฟต์"), TextPart::Syllable("แวร์")],
        ),
        (
            "ซอฟท์แอนด์เซฟ",
            vec![
                TextPart::Syllable("ซอฟท์"),
                TextPart::Syllable("แอนด์"),
                TextPart::Syllable("เซฟ"),
            ],
        ),
    ];

    for (source, expected) in cases {
        assert_eq!(text_parts(source, &split(source)), expected, "{source}");
    }
}
