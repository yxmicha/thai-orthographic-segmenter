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

#[test]
fn ownership_transfers_preserve_mixed_source_offsets() {
    for thai in [
        "มเหสี",
        "จมรรอก",
        "เจียรวนนท์",
        "ไหนครับ",
        "กะกเรณุ",
        "นาห",
        "บัติ",
    ] {
        let source = format!("🙂 /{thai}. café");
        let parts = split(&source);
        let mut end = 0;
        for part in &parts {
            let range = part.byte_range();
            assert_eq!(range.start, end, "{source}");
            assert!(range.end > range.start, "{source}");
            assert!(source.is_char_boundary(range.start), "{source}");
            assert!(source.is_char_boundary(range.end), "{source}");
            end = range.end;
        }
        assert_eq!(end, source.len(), "{source}");
        assert_eq!(
            parts
                .iter()
                .map(|part| part.text(&source))
                .collect::<String>(),
            source
        );
    }
}
