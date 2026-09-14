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
