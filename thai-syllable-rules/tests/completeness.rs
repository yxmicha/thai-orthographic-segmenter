mod text_part;

use text_part::{TextPart, text_parts};
use thai_syllable_rules::split;

#[test]
fn required_rime_endings_are_not_supplied_by_the_end_of_input() {
    for source in ["กั", "เกิ", "เก็", "กื"] {
        let parts = split(source);
        assert_eq!(
            text_parts(source, &parts),
            vec![TextPart::Invalid(source)],
            "{source}"
        );
    }
}
