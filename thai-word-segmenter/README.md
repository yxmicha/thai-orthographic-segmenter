# thai-word-segmenter

Source-preserving Thai word segmentation. Orthographic syllables from
[`thai-syllable-segmenter`](https://docs.rs/thai-syllable-segmenter) are grouped into lexical
clusters (words, compounds, names, productive bound forms) by a contextual Transformer and a
dictionary of protected multi-syllable words. It runs on the CPU with bundled weights
(epoch 6 of the training run).

## Installation

```sh
cargo add thai-word-segmenter
```

The minimum supported Rust version is 1.95.

## Usage

```rust
use thai_word_segmenter::WordSegmenter;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let segmenter = WordSegmenter::new()?;
    let words = segmenter.split("ในปฐมกาลพระวาทะทรงดำรงอยู่")?;
    assert_eq!(words, ["ในปฐมกาล", "พระวาทะ", "ทรงดำรง", "อยู่"]);
    Ok(())
}
```

`split` returns borrowed slices whose concatenation is the input; `ranges` returns the same
segments as byte ranges.

- `split_many(&texts)` and `ranges_many(&texts)` give the same result as calling `split` on each
  text, but score all the texts' Thai runs together, which is roughly twice as fast for short
  texts. Pass bounded batches, since memory grows with their total length.
- `split_many_on(&texts, threads)` and `ranges_many_on` do the same on up to `threads` worker
  threads that share the segmenter, giving the identical result.
- `WordSegmenter::with_dictionary(text)` swaps in a newline-delimited dictionary of Thai
  words but keeps the bundled model.
- `WordSegmenter::load(directory)` loads a trained artifact directory (`config.json`,
  `dictionary.txt`, `model.safetensors` as 32-bit floats or in the compact 16-bit or 8-bit forms).
- Failures are `thai_word_segmenter::Error`. A Thai run over 4,096 scalars fails with
  `Error::Syllables(ThaiRunTooLong)` because the syllable segmenter rejects it.

