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
