use std::ops::Range;

mod text_part;
use thai_syllable_rules::{Part, split};

fn ranges(parts: &[Part]) -> Vec<Range<usize>> {
    parts.iter().map(Part::byte_range).collect()
}
