//! Properties of the syllable segmenter and the checkpoint reader on arbitrary input.

use std::sync::LazyLock;

use proptest::prelude::*;
use thai_syllable_segmenter::{Error, SyllableSegmenter, inference::Checkpoint};

static SEGMENTER: LazyLock<SyllableSegmenter> =
    LazyLock::new(|| SyllableSegmenter::new().expect("the bundled model loads"));
