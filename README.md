# Thai orthographic segmenter

Four layered Thai segmentation crates, from spelling rules up to line breaking.

| Crate | Role | Output |
| --- | --- | --- |
| [`thai-syllable-rules`](https://docs.rs/thai-syllable-rules) | Deterministic orthographic candidates; ambiguity stays explicit | `Vec<Part>` of byte ranges |
| [`thai-syllable-segmenter`](https://docs.rs/thai-syllable-segmenter) | Written syllables from the rules and a bundled contextual model | borrowed slices |
| [`thai-word-segmenter`](https://docs.rs/thai-word-segmenter) | Words (lexical clusters) from syllables, a bundled cluster model and a dictionary | borrowed slices or byte ranges |
| [`thai-line-segmenter`](https://docs.rs/thai-line-segmenter) | Punctuation attachment plus zero-width joiners and spaces for line breaking | a new `String` |

Each layer builds on the one above it in the table. The first three return slices or ranges
of the original input that reconstruct it exactly. The line segmenter inserts characters, so
it does not; see its readme for what it guarantees.

[`thai-segmenter-python`](thai-segmenter-python/README.md) wraps all four as a Python package
(`thai_orthographic_segmenter`, built with maturin). It is not published to crates.io.

