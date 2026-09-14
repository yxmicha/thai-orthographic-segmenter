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
