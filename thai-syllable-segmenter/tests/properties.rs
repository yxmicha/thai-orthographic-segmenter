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
