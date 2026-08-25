mod text_part;

use text_part::{TextPart, text_parts};
use thai_syllable_rules::split;

#[test]
fn required_coda_takes_a_following_onset_when_its_remainder_is_complete() {
    for (source, completed) in [
        ("ก้บอก", "ก้บ"),
        ("ก่บอก", "ก่บ"),
        ("ก๊บอก", "ก๊บ"),
        ("ก๋บอก", "ก๋บ"),
    ] {
        assert_eq!(
            text_parts(source, &split(source)),
            vec![TextPart::Syllable(completed), TextPart::Syllable("อก"),],
            "{source}"
        );
    }
}
