//! Properties of the syllable segmenter and the checkpoint reader on arbitrary input.

use std::sync::LazyLock;

use proptest::prelude::*;
use thai_syllable_segmenter::{Error, SyllableSegmenter, inference::Checkpoint};

static SEGMENTER: LazyLock<SyllableSegmenter> =
    LazyLock::new(|| SyllableSegmenter::new().expect("the bundled model loads"));

fn text() -> impl Strategy<Value = String> {
    proptest::collection::vec(
        prop_oneof![
            8 => proptest::char::range('\u{0e00}', '\u{0e7f}'),
            2 => proptest::char::range('a', 'z'),
            1 => prop_oneof![Just(' '), Just('.'), Just('\n'), Just('ก')],
            1 => any::<char>(),
        ],
        0..60,
    )
    .prop_map(|characters| characters.into_iter().collect())
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(128))]

    #[test]
    fn syllables_concatenate_to_the_input(source in text()) {
        let syllables = SEGMENTER.split(&source)?;
        prop_assert_eq!(syllables.concat(), source);
    }

    #[test]
    fn ranges_partition_the_input_on_character_boundaries(source in text()) {
        let ranges = SEGMENTER.ranges(&source)?;
        let mut end = 0;
        for range in ranges {
            prop_assert_eq!(range.start, end);
            prop_assert!(source.is_char_boundary(range.start) && source.is_char_boundary(range.end));
            end = range.end;
        }
        prop_assert_eq!(end, source.len());
    }
}
