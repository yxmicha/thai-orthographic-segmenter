mod text_part;

use text_part::{TextPart, text_parts};
use thai_syllable_rules::{Part, split};

#[test]
fn keeps_a_repeated_y_tail_available_to_the_common_base_word() {
    let source = "อร่อยยย";

    assert_eq!(
        text_parts(source, &split(source)),
        vec![
            TextPart::Syllable("อร่อ"),
            TextPart::Undecided("ย"),
            TextPart::Syllable("ยย"),
        ]
    );
}

#[test]
fn keeps_a_short_u_final_ambiguous_before_a_following_nucleus() {
    let cases = [
        (
            "ปทุมธานี",
            vec![
                TextPart::Syllable("ปทุ"),
                TextPart::Undecided("ม"),
                TextPart::Syllable("ธา"),
                TextPart::Syllable("นี"),
            ],
        ),
        (
            "สุขภาพ",
            vec![
                TextPart::Syllable("สุ"),
                TextPart::Undecided("ข"),
                TextPart::Syllable("ภาพ"),
            ],
        ),
        (
            "หยุดการ",
            vec![
                TextPart::Syllable("หยุ"),
                TextPart::Undecided("ด"),
                TextPart::Syllable("การ"),
            ],
        ),
    ];

    for (source, expected) in cases {
        assert_eq!(text_parts(source, &split(source)), expected, "{source}");
    }
}

#[test]
fn does_not_merge_independent_vowel_nuclei_into_one_syllable() {
    let source = "นาวัฒ";

    assert_eq!(
        text_parts(source, &split(source)),
        vec![TextPart::Syllable("นา"), TextPart::Syllable("วัฒ")]
    );
}

#[test]
fn keeps_the_cross_atom_ao_pattern_together() {
    let cases = [
        ("เฉพาะ", vec![TextPart::Syllable("เฉพาะ")]),
        ("เรพาะ", vec![TextPart::Syllable("เรพาะ")]),
        ("เสนาะ", vec![TextPart::Syllable("เสนาะ")]),
    ];

    for (source, expected) in cases {
        assert_eq!(text_parts(source, &split(source)), expected, "{source}");
    }
}

#[test]
fn keeps_a_coda_capable_base_in_the_ambiguous_previous_boundary_class() {
    let source = "ว่ากอนด้า";

    assert_eq!(
        text_parts(source, &split(source)),
        vec![
            TextPart::Syllable("ว่า"),
            TextPart::Undecided("ก"),
            TextPart::Undecided("อ"),
            TextPart::Undecided("น"),
            TextPart::Syllable("ด้า"),
        ]
    );
}

#[test]
fn keeps_both_bases_between_leading_and_following_syllables_ambiguous() {
    let source = "เยอรมัน";

    assert_eq!(
        text_parts(source, &split(source)),
        vec![
            TextPart::Syllable("เย"),
            TextPart::Undecided("อ"),
            TextPart::Undecided("ร"),
            TextPart::Syllable("มัน"),
        ]
    );
}

#[test]
fn keeps_ordinary_finals_ambiguous_before_following_leading_vowels() {
    let cases = [
        (
            "เปลี่ยนยัง",
            vec![
                TextPart::Syllable("เปลี่ย"),
                TextPart::Undecided("น"),
                TextPart::Syllable("ยัง"),
            ],
        ),
        (
            "ขายใน",
            vec![
                TextPart::Syllable("ขา"),
                TextPart::Undecided("ย"),
                TextPart::Syllable("ใน"),
            ],
        ),
        (
            "เสียงให้",
            vec![
                TextPart::Syllable("เสีย"),
                TextPart::Undecided("ง"),
                TextPart::Syllable("ให้"),
            ],
        ),
        (
            "ท้องเสีย",
            vec![
                TextPart::Syllable("ท้อ"),
                TextPart::Undecided("ง"),
                TextPart::Syllable("เสีย"),
            ],
        ),
        (
            "ก้อนเจ้า",
            vec![
                TextPart::Syllable("ก้อ"),
                TextPart::Undecided("น"),
                TextPart::Syllable("เจ้า"),
            ],
        ),
    ];

    for (source, expected) in cases {
        assert_eq!(text_parts(source, &split(source)), expected, "{source}");
    }
}

#[test]
fn keeps_a_short_i_final_ambiguous_before_a_following_rime() {
    let source = "กินซัก";

    assert_eq!(
        text_parts(source, &split(source)),
        vec![
            TextPart::Syllable("กิ"),
            TextPart::Undecided("น"),
            TextPart::Syllable("ซัก"),
        ]
    );
}

#[test]
fn keeps_the_short_i_final_in_vit_undecided_before_a_following_syllable() {
    let source = "วิตที่";

    assert_eq!(
        text_parts(source, &split(source)),
        vec![
            TextPart::Syllable("วิ"),
            TextPart::Undecided("ต"),
            TextPart::Syllable("ที่"),
        ]
    );
}

#[test]
fn keeps_the_short_i_final_in_nis_undecided_before_a_following_syllable() {
    let source = "นิสสัน";

    assert_eq!(
        text_parts(source, &split(source)),
        vec![
            TextPart::Syllable("นิ"),
            TextPart::Undecided("ส"),
            TextPart::Syllable("สัน"),
        ]
    );
}

#[test]
fn keeps_the_short_i_final_in_phalit_undecided_before_a_following_syllable() {
    let source = "ผลิตภัณฑ์";

    assert_eq!(
        text_parts(source, &split(source)),
        vec![
            TextPart::Syllable("ผลิ"),
            TextPart::Undecided("ต"),
            TextPart::Syllable("ภัณฑ์"),
        ]
    );
}

#[test]
fn keeps_two_ordinary_consonants_ambiguous_before_a_following_leading_vowel() {
    let source = "มันรสเหมือน";

    assert_eq!(
        text_parts(source, &split(source)),
        vec![
            TextPart::Syllable("มัน"),
            TextPart::Undecided("ร"),
            TextPart::Undecided("ส"),
            TextPart::Syllable("เหมือน"),
        ]
    );
}
