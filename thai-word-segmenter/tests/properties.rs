//! Properties of the word segmenter and its post-processing on arbitrary input.

use std::sync::LazyLock;

use proptest::prelude::*;
use thai_word_segmenter::{
    WordSegmenter,
    decoding::{BoundaryScores, SourceText},
    dictionary::ClusterDictionary,
};

static SEGMENTER: LazyLock<WordSegmenter> =
    LazyLock::new(|| WordSegmenter::new().expect("the bundled model loads"));

fn text() -> impl Strategy<Value = String> {
    proptest::collection::vec(
        prop_oneof![
            8 => proptest::char::range('\u{0e00}', '\u{0e7f}'),
            2 => proptest::char::range('a', 'z'),
            2 => prop_oneof![Just(' '), Just('.'), Just(','), Just('"'), Just('('), Just(')'), Just('\n')],
            1 => any::<char>(),
        ],
        0..60,
    )
    .prop_map(|characters| characters.into_iter().collect())
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(96))]

    #[test]
    fn words_concatenate_to_the_input(source in text()) {
        prop_assert_eq!(SEGMENTER.split(&source)?.concat(), source);
    }

    #[test]
    fn ranges_partition_the_input_on_character_boundaries(source in text()) {
        let mut end = 0;
        for range in SEGMENTER.ranges(&source)? {
            prop_assert_eq!(range.start, end);
            prop_assert!(range.start < range.end);
            prop_assert!(source.is_char_boundary(range.start) && source.is_char_boundary(range.end));
            end = range.end;
        }
        prop_assert_eq!(end, source.len());
    }

    #[test]
    fn scoring_texts_together_equals_scoring_them_alone(
        sources in proptest::collection::vec(text(), 0..5)
    ) {
        let views = sources.iter().map(String::as_str).collect::<Vec<_>>();
        let together = SEGMENTER.split_many(&views)?;
        let alone = views
            .iter()
            .map(|source| SEGMENTER.split(source))
            .collect::<Result<Vec<_>, _>>()?;
        prop_assert_eq!(together, alone);
    }
}
