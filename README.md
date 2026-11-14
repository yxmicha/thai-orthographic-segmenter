# Thai orthographic segmenter

[![CI](https://github.com/yxmicha/thai-orthographic-segmenter/actions/workflows/ci.yml/badge.svg)](https://github.com/yxmicha/thai-orthographic-segmenter/actions/workflows/ci.yml)
[![PyPI](https://img.shields.io/pypi/v/thai-orthographic-segmenter.svg)](https://pypi.org/project/thai-orthographic-segmenter/)
[![Python versions](https://img.shields.io/pypi/pyversions/thai-orthographic-segmenter.svg)](https://pypi.org/project/thai-orthographic-segmenter/)
[![License](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue.svg)](https://github.com/yxmicha/thai-orthographic-segmenter#license)
[![MSRV](https://img.shields.io/badge/rustc-1.95%2B-orange.svg)](https://www.rust-lang.org)

Four layered Thai segmentation crates, from spelling rules up to line breaking.

| Crate | Version | Role | Output |
| --- | --- | --- | --- |
| [`thai-syllable-rules`](https://docs.rs/thai-syllable-rules) | [![crates.io](https://img.shields.io/crates/v/thai-syllable-rules.svg)](https://crates.io/crates/thai-syllable-rules) | Deterministic orthographic candidates; ambiguity stays explicit | `Vec<Part>` of byte ranges |
| [`thai-syllable-segmenter`](https://docs.rs/thai-syllable-segmenter) | [![crates.io](https://img.shields.io/crates/v/thai-syllable-segmenter.svg)](https://crates.io/crates/thai-syllable-segmenter) | Written syllables from the rules and a bundled contextual model | borrowed slices |
| [`thai-word-segmenter`](https://docs.rs/thai-word-segmenter) | [![crates.io](https://img.shields.io/crates/v/thai-word-segmenter.svg)](https://crates.io/crates/thai-word-segmenter) | Words (lexical clusters) from syllables, a bundled cluster model and a dictionary | borrowed slices or byte ranges |
| [`thai-line-segmenter`](https://docs.rs/thai-line-segmenter) | [![crates.io](https://img.shields.io/crates/v/thai-line-segmenter.svg)](https://crates.io/crates/thai-line-segmenter) | Punctuation attachment plus zero-width joiners and spaces for line breaking | a new `String` |

Each layer builds on the one above it in the table. The first three return slices or ranges
of the original input that reconstruct it exactly. The line segmenter inserts characters, so
it does not; see its readme for what it guarantees.

[`thai-segmenter-python`](thai-segmenter-python/README.md) wraps all four as a Python package
(`thai_orthographic_segmenter`, built with maturin) and is published on [PyPI](https://pypi.org/project/thai-orthographic-segmenter/), not on crates.io.

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

## Development

The repository ships a Nix flake with the pinned Rust toolchain (from `rust-toolchain.toml`), a
C linker, `cargo-deny`, `cargo-hack` and `gitleaks`. With [direnv](https://direnv.net/) run
`direnv allow`; otherwise run `nix develop`. The checks CI runs:

```sh
cargo fmt --all --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all-targets --all-features
cargo test --no-default-features
cargo deny check
cargo hack check --feature-powerset
```

The Python package has its own checks, run in `thai-segmenter-python/` with
[maturin](https://www.maturin.rs/) and [pytest](https://pytest.org/):

```sh
maturin develop --release
pytest
```

## License

Licensed under either of [Apache License, Version 2.0](LICENSE-APACHE) or
[MIT license](LICENSE-MIT) at your option.
