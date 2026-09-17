# thai-syllable-segmenter

Source-preserving Thai syllable segmentation. It combines the candidates from
[`thai-syllable-rules`](https://docs.rs/thai-syllable-rules) with a small contextual Transformer
and picks the best complete segmentation with a dynamic-programming decoder. It runs on the
CPU with bundled weights: no model download, CUDA runtime or lexicon.

## Installation

```sh
cargo add thai-syllable-segmenter
```

## Usage

```rust
use thai_syllable_segmenter::SyllableSegmenter;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let segmenter = SyllableSegmenter::new()?;
    let pieces = segmenter.split("ฉันกินข้าว สวัสดี!")?;
    assert_eq!(pieces, ["ฉัน", "กิน", "ข้าว", " ", "สวัส", "ดี", "!"]);
    Ok(())
}
```

Construct the segmenter once and reuse it. `split` returns borrowed slices whose
concatenation is the input. Empty input gives an empty vector.

