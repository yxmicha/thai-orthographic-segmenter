mod text_part;

use text_part::{TextPart, text_parts};
use thai_syllable_rules::split;

#[test]
fn decomposed_sara_am_closes_before_an_extra_consonant() {
    for syllable in ["น้ำ", "นํ้า", "น้ํา", "กำ", "กํา"] {
        let source = format!("{syllable}ก");
        let parts = split(&source);
        assert_eq!(
            text_parts(&source, &parts),
            vec![TextPart::Syllable(syllable), TextPart::Undecided("ก")],
            "{source}"
        );
    }
}
