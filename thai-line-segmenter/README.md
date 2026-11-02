# thai-line-segmenter

Prepares Thai and mixed text for line breaking. It attaches punctuation to the text next to
it and inserts zero-width joiners (U+200D) and zero-width spaces (U+200B) so that layout
engines break lines only at word boundaries.

## Installation

```sh
cargo add thai-line-segmenter
```

The minimum supported Rust version is 1.95.

## Usage

```rust
use thai_line_segmenter::LineSegmenter;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let segmenter = LineSegmenter::new()?;
    let input = "“ในปฐมกาลพระวาทะทรงดำรงอยู่”";
    let normalized = segmenter.normalize_line_boundaries(input)?;

    // Deleting U+200B and U+200D recovers the input.
    assert_eq!(normalized.replace(['\u{200b}', '\u{200d}'], ""), input);
    Ok(())
}
```

`normalize_line_boundaries` returns a new `String`. `LineSegmenter::with_icu_dictionary(text)`
replaces the bundled ICU word list, a newline-delimited list of 26,390 entries. The
constants `NON_BREAKING_SPACE`, `ZERO_WIDTH_JOINER` and `ZERO_WIDTH_SPACE` are public. Errors
are `thai_line_segmenter::Error` (`Words` wraps a word-segmentation failure).

