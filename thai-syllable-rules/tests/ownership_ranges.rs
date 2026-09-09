mod text_part;

use text_part::{TextPart, text_parts};
use thai_syllable_rules::split;

#[test]
fn terminal_onset_only_consonants_do_not_become_codas() {
    for (source, tail) in [("นาห", "ห"), ("นาฮ", "ฮ"), ("นาผ", "ผ")] {
        assert_eq!(
            text_parts(source, &split(source)),
            vec![TextPart::Syllable("นา"), TextPart::Undecided(tail)],
            "{source}",
        );
    }
}
