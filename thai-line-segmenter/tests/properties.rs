//! Properties of the line segmenter on arbitrary input.

use std::sync::LazyLock;

use proptest::prelude::*;
use thai_line_segmenter::{LineSegmenter, NON_BREAKING_SPACE, ZERO_WIDTH_JOINER, ZERO_WIDTH_SPACE};

static SEGMENTER: LazyLock<LineSegmenter> =
    LazyLock::new(|| LineSegmenter::new().expect("the bundled models load"));
