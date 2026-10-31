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
