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

#[test]
fn does_not_consume_a_second_dependent_vowel_pattern_without_a_base() {
    let cases = [
        ("เคราะห์", vec![TextPart::Syllable("เคราะห์")]),
        ("กีา", vec![TextPart::Syllable("กี"), TextPart::Text("า")]),
        ("กุา", vec![TextPart::Syllable("กุ"), TextPart::Text("า")]),
        ("กะา", vec![TextPart::Syllable("กะ"), TextPart::Text("า")]),
    ];

    for (source, expected) in cases {
        assert_eq!(text_parts(source, &split(source)), expected, "{source}");
    }
}

#[test]
fn keeps_a_post_final_dependent_mark_with_a_hard_ending() {
    let source = "สมบัติมากกว่า";

    assert_eq!(
        text_parts(source, &split(source)),
        vec![
            TextPart::Syllable("สม"),
            TextPart::Syllable("บัติ"),
            TextPart::Syllable("มา"),
            TextPart::Undecided("ก"),
            TextPart::Undecided("ก"),
            TextPart::Syllable("ว่า"),
        ]
    );
}

#[test]
fn keeps_only_one_possible_attachment_with_the_adjacent_syllable() {
    let cases = [
        (
            "นะก",
            vec![TextPart::Syllable("นะ"), TextPart::Undecided("ก")],
        ),
        (
            "นะกม",
            vec![TextPart::Syllable("นะ"), TextPart::Syllable("กม")],
        ),
        (
            "อิสรา",
            vec![
                TextPart::Syllable("อิ"),
                TextPart::Undecided("ส"),
                TextPart::Syllable("รา"),
            ],
        ),
        (
            "อิสรอ",
            vec![
                TextPart::Syllable("อิ"),
                TextPart::Undecided("ส"),
                TextPart::Syllable("รอ"),
            ],
        ),
        (
            "เหรียญเจือะ",
            vec![
                TextPart::Syllable("เหรีย"),
                TextPart::Undecided("ญ"),
                TextPart::Syllable("เจือะ"),
            ],
        ),
        (
            "จมกนะ",
            vec![
                TextPart::Syllable("จม"),
                TextPart::Undecided("ก"),
                TextPart::Syllable("นะ"),
            ],
        ),
        (
            "เป็นอยู่",
            vec![TextPart::Syllable("เป็น"), TextPart::Syllable("อยู่")],
        ),
    ];

    for (source, expected) in cases {
        assert_eq!(text_parts(source, &split(source)), expected, "{source}");
    }
}

#[test]
fn never_emits_a_single_thai_base_as_a_syllable() {
    let cases = [
        ("ฮ", vec![TextPart::Undecided("ฮ")]),
        ("อ", vec![TextPart::Undecided("อ")]),
        (
            "นะก",
            vec![TextPart::Syllable("นะ"), TextPart::Undecided("ก")],
        ),
        (
            "กะฮ",
            vec![TextPart::Syllable("กะ"), TextPart::Undecided("ฮ")],
        ),
    ];

    for (source, expected) in cases {
        assert_eq!(text_parts(source, &split(source)), expected, "{source}");
    }
}

#[test]
fn does_not_promote_a_terminal_run_ending_in_an_onset_only_base() {
    let source = "กะนห";

    assert_eq!(
        text_parts(source, &split(source)),
        vec![
            TextPart::Syllable("กะ"),
            TextPart::Undecided("น"),
            TextPart::Undecided("ห"),
        ]
    );
}

#[test]
fn keeps_a_tone_marked_ue_final_undecided_before_a_following_syllable() {
    let source = "พื้นที่";

    assert_eq!(
        text_parts(source, &split(source)),
        vec![
            TextPart::Syllable("พื้"),
            TextPart::Undecided("น"),
            TextPart::Syllable("ที่"),
        ]
    );
}

#[test]
fn keeps_a_dot_tail_undecided_after_an_inherent_final_syllable() {
    let source = "ในจมก.";

    assert_eq!(
        text_parts(source, &split(source)),
        vec![
            TextPart::Syllable("ใน"),
            TextPart::Syllable("จม"),
            TextPart::Undecided("ก"),
            TextPart::Dot("."),
        ]
    );
}

#[test]
fn keeps_two_ambiguous_bases_between_syllables() {
    let source = "เจียรวนนท์";

    assert_eq!(
        text_parts(source, &split(source)),
        vec![
            TextPart::Syllable("เจีย"),
            TextPart::Undecided("ร"),
            TextPart::Undecided("ว"),
            TextPart::Syllable("นนท์"),
        ]
    );
}

#[test]
fn preserves_an_unpaired_leading_vowel_as_text() {
    let source = "เแทบขาด";

    assert_eq!(
        text_parts(source, &split(source)),
        vec![
            TextPart::Text("เ"),
            TextPart::Syllable("แท"),
            TextPart::Undecided("บ"),
            TextPart::Syllable("ขาด"),
        ]
    );
}
