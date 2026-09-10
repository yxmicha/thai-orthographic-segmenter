use std::ops::Range;

mod text_part;

use text_part::{TextPart, text_parts};
use thai_syllable_rules::{Part, split};

fn ranges(parts: &[Part]) -> Vec<Range<usize>> {
    parts.iter().map(Part::byte_range).collect()
}

#[test]
fn preserves_text_and_ranges_exactly() {
    let source = "นะ. / คะ!";
    let parts = split(source);

    assert_eq!(
        text_parts(source, &parts),
        vec![
            TextPart::Syllable("นะ"),
            TextPart::Dot("."),
            TextPart::Text(" / "),
            TextPart::Syllable("คะ"),
            TextPart::Text("!"),
        ]
    );
    assert_eq!(ranges(&parts).first().map(|range| range.start), Some(0));
    assert_eq!(
        ranges(&parts).last().map(|range| range.end),
        Some(source.len())
    );
    assert!(
        ranges(&parts)
            .windows(2)
            .all(|window| window[0].end == window[1].start)
    );
}
