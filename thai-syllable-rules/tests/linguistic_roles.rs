mod text_part;

use text_part::{TextPart, text_parts};
use thai_syllable_rules::split;

#[test]
fn orphan_marks_are_not_repaired_by_assigning_them_to_a_later_base() {
    for (source, mark) in [("่กา", "่"), ("้กา", "้"), ("ฺกา", "ฺ")] {
        assert_eq!(
            text_parts(source, &split(source)),
            vec![TextPart::Text(mark), TextPart::Syllable("กา")]
        );
    }
}

#[test]
fn unsupported_phinthu_run_is_not_an_ordinary_implicit_candidate() {
    assert_eq!(
        text_parts("กฺก", &split("กฺก")),
        vec![TextPart::Undecided("กฺ"), TextPart::Undecided("ก")]
    );
    assert_eq!(
        text_parts("กะกฺก", &split("กะกฺก")),
        vec![
            TextPart::Syllable("กะ"),
            TextPart::Undecided("กฺ"),
            TextPart::Undecided("ก")
        ]
    );
}

#[test]
fn terminal_carrier_completes_a_leading_vowel_without_becoming_a_coda() {
    for source in ["เหรอ", "เจอ"] {
        assert_eq!(
            text_parts(source, &split(source)),
            vec![TextPart::Syllable(source)]
        );
    }
}

#[test]
fn incomplete_tone_only_unit_is_not_made_complete_by_run_boundaries() {
    for source in ["ก่", "ก้", "ก๊", "ก๋"] {
        assert_eq!(
            text_parts(source, &split(source)),
            vec![TextPart::Invalid(source)]
        );
    }
}

#[test]
fn phinthu_does_not_supply_a_vowel_nucleus() {
    assert_eq!(text_parts("กฺ", &split("กฺ")), vec![TextPart::Undecided("กฺ")]);
}

#[test]
fn implicit_cluster_requires_a_coda_capable_final() {
    for (source, final_base) in [("กรห", "ห"), ("กรฮ", "ฮ"), ("กรผ", "ผ")] {
        assert_eq!(
            text_parts(source, &split(source)),
            vec![
                TextPart::Undecided("ก"),
                TextPart::Undecided("ร"),
                TextPart::Undecided(final_base),
            ],
            "{source}"
        );
    }
}
