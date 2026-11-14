# thai-line-segmenter

[![crates.io](https://img.shields.io/crates/v/thai-line-segmenter.svg)](https://crates.io/crates/thai-line-segmenter)
[![docs.rs](https://img.shields.io/docsrs/thai-line-segmenter)](https://docs.rs/thai-line-segmenter)
[![downloads](https://img.shields.io/crates/d/thai-line-segmenter.svg)](https://crates.io/crates/thai-line-segmenter)
[![MSRV](https://img.shields.io/crates/msrv/thai-line-segmenter.svg)](https://crates.io/crates/thai-line-segmenter)
[![CI](https://github.com/yxmicha/thai-orthographic-segmenter/actions/workflows/ci.yml/badge.svg)](https://github.com/yxmicha/thai-orthographic-segmenter/actions/workflows/ci.yml)
[![License](https://img.shields.io/badge/license-(MIT%20OR%20Apache--2.0)%20AND%20Unicode--3.0-blue.svg)](https://github.com/yxmicha/thai-orthographic-segmenter/blob/master/thai-line-segmenter/data/icu/LICENSE)

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

## What it does

The input is cut into Thai runs, punctuation prefixes and suffixes, and other text.

1. **Other text** (Latin and so on) is copied unchanged. Attached punctuation is
   concatenated.
2. **Thai runs** are first cleaned: existing zero-width characters (U+200B, U+200C, U+200D,
   U+FEFF) are removed and special spaces (U+00A0, U+202F, U+2009, U+200A, U+205F) become
   ordinary spaces. The run is then split into words by
   [`thai-word-segmenter`](https://docs.rs/thai-word-segmenter).
3. Words are joined with **U+200B**, the break opportunities.
4. Inside each word, the longest ICU dictionary words are found and joined with **U+200D**,
   so a layout engine's own dictionary breaker cannot split the word there. A part of the word
   the dictionary does not cover gets U+200D between each pair of neighbouring characters
   that it could otherwise break apart.
5. **Punctuation** prefixes (opening quotes and brackets, `$`, digits, dashes, ellipsis and
   similar) and suffixes (closing quotes and brackets, `, . ! ? : ; % ๆ ฯ` and similar) are
   joined to their Thai run with U+200D. Inside the attached punctuation, spaces become
   non-breaking spaces (U+00A0), and neighbouring characters are joined with U+200D except
   digit pairs and pairs touching an ordinary space.

## What it preserves

Output differs from the input only by inserted U+200B and U+200D, spaces inside attached
punctuation becoming U+00A0, and the cleaning of Thai runs described above (their existing
zero-width characters are removed and special spaces become ordinary spaces). So for input
without zero-width or special-space characters, deleting U+200B and U+200D and turning U+00A0
back into U+0020 recovers the input.

## Licence

The code is licensed under MIT or Apache-2.0, at your option. The bundled word list
`data/icu/th-icu.txt` is ICU's Thai dictionary (`thaidict.txt`), which is distributed under the
[Unicode License v3](data/icu/LICENSE); that file holds the copyright and permission notice that
the licence requires to accompany every copy. The crate's licence expression is therefore
`(MIT OR Apache-2.0) AND Unicode-3.0`.
