//! Properties of the line segmenter on arbitrary input.

use std::sync::LazyLock;

use proptest::prelude::*;
use thai_line_segmenter::{LineSegmenter, NON_BREAKING_SPACE, ZERO_WIDTH_JOINER, ZERO_WIDTH_SPACE};

static SEGMENTER: LazyLock<LineSegmenter> =
    LazyLock::new(|| LineSegmenter::new().expect("the bundled models load"));

/// Thai, ASCII words and the punctuation the segmenter attaches, without zero-width or special
/// spaces, which it rewrites.
fn text() -> impl Strategy<Value = String> {
    proptest::collection::vec(
        prop_oneof![
            8 => proptest::char::range('\u{0e00}', '\u{0e7f}'),
            3 => proptest::char::range('a', 'z'),
            2 => prop_oneof![Just(' '), Just('.'), Just(','), Just('"'), Just('('), Just(')'), Just('\n')],
        ],
        0..60,
    )
    .prop_map(|characters| characters.into_iter().collect())
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(96))]

    #[test]
    fn marking_only_adds_zero_width_characters_and_makes_attached_spaces_non_breaking(
        source in text()
    ) {
        let marked = SEGMENTER.normalize_line_boundaries(&source)?;
        let restored: String = marked
            .chars()
            .filter(|character| *character != ZERO_WIDTH_SPACE && *character != ZERO_WIDTH_JOINER)
            .map(|character| if character == NON_BREAKING_SPACE { ' ' } else { character })
            .collect();
        prop_assert_eq!(restored, source);
    }
}
