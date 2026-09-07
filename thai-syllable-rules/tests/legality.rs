mod text_part;

use text_part::{TextPart, text_parts};
use thai_syllable_rules::split;

#[test]
fn conflicting_combining_marks_cannot_become_a_syllable() {
    for source in ["ก่้", "กี้่", "กิี", "กุิ", "ก์์"] {
        let parts = split(source);
        assert_eq!(text_parts(source, &parts), vec![TextPart::Invalid(source)]);
    }
}
