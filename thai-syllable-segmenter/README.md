# thai-syllable-segmenter

Source-preserving Thai syllable segmentation. It combines the candidates from
[`thai-syllable-rules`](https://docs.rs/thai-syllable-rules) with a small contextual Transformer
and picks the best complete segmentation with a dynamic-programming decoder. It runs on the
CPU with bundled weights: no model download, CUDA runtime or lexicon.

## Installation

```sh
cargo add thai-syllable-segmenter
```

