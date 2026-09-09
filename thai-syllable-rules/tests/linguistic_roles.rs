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
