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

## Why this is hard

Thai is written without spaces between words. Readers find the boundaries by knowing the
vocabulary, so software has no marker to look for, and three things keep a simple rule from
working:

- **Spelling is ambiguous before meaning enters.** A consonant can end one syllable or begin the
  next. In `การออก`, `ร` could end `กา` or begin `รออก`, so even syllable boundaries cannot be
  read off the characters.
- **A word is not a settled unit.** Whether a compound such as `นายกรัฐมนตรี` is one word or two
  depends on the annotation convention, and the common benchmark sets disagree. This crate's
  word model scores about 70 to 76 F1 across three of them, while the trained segmenters in its
  [comparison](thai-word-segmenter/README.md#comparison-with-other-word-segmenters) score 81 to
  86 on the same sets.
- **Context decides.** The same characters split differently in different sentences, and new
  names, loanwords and slang keep arriving, so a dictionary alone fails.

These crates are small models that run on a CPU, offline, with the same output every time and no
per-call cost. A 2-layer model segments syllables, and a 6-layer model of about 5.5 million
parameters groups them into words. They were trained to under-segment on purpose, because line
breaking is safer when compounds stay together.

Large language models do this better. A current frontier model brings far more vocabulary and
context to a sentence than these models hold, and it will usually segment more accurately than
anything here. The trade is accuracy for running locally and cheaply.

The accuracy of the small models also reflects how much was spent teaching them. They were
distilled from larger models, and their labels are only as good as the model that reviewed them
and the tokens spent on that review. Labelling and reviewing more text with a stronger model
would give better training data, and distilling it into the same architecture should give much
better results.

