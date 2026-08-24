mod text_part;

use text_part::{TextPart, text_parts};
use thai_syllable_rules::{Part, split};

#[test]
fn recognizes_vowel_bearing_thai_units_as_syllables() {
    let source = "นะคะ";
    let parts = split(source);

    assert!(matches!(
        parts.as_slice(),
        [Part::Syllable(_), Part::Syllable(_)]
    ));
    assert_eq!(
        text_parts(source, &parts),
        vec![TextPart::Syllable("นะ"), TextPart::Syllable("คะ")]
    );
}
