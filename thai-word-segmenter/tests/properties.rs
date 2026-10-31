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
