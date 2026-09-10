//! Properties of the rule parser on arbitrary text: it never panics, and its parts partition the
//! input on character boundaries.

use proptest::prelude::*;
use thai_syllable_rules::{Part, RuleAnalysis, SyllablePattern, split};

/// Thai letters, vowels and marks mixed with ASCII, spaces, dots and other scripts.
fn text() -> impl Strategy<Value = String> {
    proptest::collection::vec(
        prop_oneof![
            8 => proptest::char::range('\u{0e00}', '\u{0e7f}'),
            2 => proptest::char::range('a', 'z'),
            1 => prop_oneof![Just(' '), Just('.'), Just('\n'), Just('\u{200b}'), Just('ก')],
            1 => any::<char>(),
        ],
        0..40,
    )
    .prop_map(|characters| characters.into_iter().collect())
}

fn assert_partition(source: &str, parts: &[Part]) {
    let mut end = 0;
    for part in parts {
        let range = part.byte_range();
        assert_eq!(
            range.start, end,
            "gap or overlap before {part:?} in {source:?}"
        );
        assert!(range.start < range.end, "empty part {part:?} in {source:?}");
        assert!(source.is_char_boundary(range.start) && source.is_char_boundary(range.end));
        end = range.end;
    }
    assert_eq!(
        end,
        source.len(),
        "parts do not reach the end of {source:?}"
    );
}

proptest! {
    #[test]
    fn parts_partition_the_source(source in text()) {
        let parts = split(&source);
        assert_partition(&source, &parts);
        let joined: String = parts.iter().map(|part| part.text(&source)).collect();
        prop_assert_eq!(joined, source);
    }

    #[test]
    fn analysis_recovers_a_partition_of_the_same_source(source in text()) {
        let parts = split(&source);
        let recovered = RuleAnalysis::new(&source).recover_parts(&parts);
        assert_partition(&source, &recovered);
    }

    #[test]
    fn patterns_are_inspectable_for_any_text(source in text()) {
        if let Some(pattern) = SyllablePattern::inspect(&source) {
            // Whatever the count, asking whether it is a candidate must not panic.
            let _ = pattern.is_candidate(&source);
        }
    }

    #[test]
    fn splitting_is_deterministic(source in text()) {
        prop_assert_eq!(split(&source), split(&source));
    }
}
