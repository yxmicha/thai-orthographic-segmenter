# thai-syllable-rules

[![crates.io](https://img.shields.io/crates/v/thai-syllable-rules.svg)](https://crates.io/crates/thai-syllable-rules)
[![docs.rs](https://img.shields.io/docsrs/thai-syllable-rules)](https://docs.rs/thai-syllable-rules)
[![downloads](https://img.shields.io/crates/d/thai-syllable-rules.svg)](https://crates.io/crates/thai-syllable-rules)
[![MSRV](https://img.shields.io/crates/msrv/thai-syllable-rules.svg)](https://crates.io/crates/thai-syllable-rules)
[![CI](https://github.com/yxmicha/thai-orthographic-segmenter/actions/workflows/ci.yml/badge.svg)](https://github.com/yxmicha/thai-orthographic-segmenter/actions/workflows/ci.yml)
[![License](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue.svg)](https://github.com/yxmicha/thai-orthographic-segmenter#license)

Deterministic Thai orthographic candidates with explicit attachment uncertainty. It uses no
model, dictionary, frequency data or training corpus; its only dependency is `winnow`.

## Installation

```sh
cargo add thai-syllable-rules
```

The minimum supported Rust version is 1.95.

## Usage

```rust
use thai_syllable_rules::{Part, split};

fn main() {
    let source = "นะ.";
    let parts = split(source);
    assert!(matches!(parts.as_slice(), [Part::Syllable(_), Part::Dot(_)]));
    assert_eq!(parts[0].text(source), "นะ");
}
```

`split(&str) -> Vec<Part>` returns byte ranges into the input. The ranges are contiguous,
non-empty, aligned to UTF-8 boundaries and together cover the input exactly. Use
`Part::byte_range()` or `Part::text(source)` to read a part.

| Part | Meaning |
| --- | --- |
| `Syllable` | A Thai orthographic candidate or anchor, always at least two Thai characters. It is not a guarantee of one spoken syllable or a dictionary word, and adjacent `Undecided` material may still belong to it. |
| `Undecided` | Thai material whose attachment to the preceding or following syllable orthography cannot decide, including a bare consonant right before a recognized period. |
| `Invalid` | A cell violating mark-order rules (`ก่้`, hostless `ธุ์`), or a construction still missing its required ending. This is not general spelling validation. |
| `Dot` | Exactly one ASCII period immediately after a `Syllable` or `Undecided`. Repeated or misplaced periods are `Text`. |
| `Text` | Non-Thai material (Latin, digits, whitespace, punctuation), orphan Thai marks, and misplaced periods. |

Example outputs (`S` syllable, `?` undecided, `!` invalid, `.` dot, `T` text):

| Input | Parts |
| --- | --- |
| `นะ.` | `S:นะ` `.:.` |
| `นะก.` | `S:นะ` `?:ก` `.:.` |
| `การออก` | `S:กา` `?:ร` `S:ออก` |
| `ฉันกินข้าว` | `S:ฉัน` `S:กิ` `?:น` `S:ข้าว` |
| `ศิลป์คน` | `S:ศิลป์` `S:คน` |
| `ก่้` | `!:ก่้` |
| `ไทย 123` | `S:ไทย` `T: 123` |

`การออก` shows the central rule: `ร` could end `กา` or begin `รออก`, so it stays `Undecided`
instead of being guessed. The rules eliminate illegal local combinations first, commit
attachments that have exactly one legal owner second, and leave every remaining choice for
a later resolver. `thai-syllable-segmenter` is that resolver.

## Other public API

- `RuleAnalysis::new(source)` finds readings the stable `split` projection cannot express
  (weak initials, shared consonants, รร, preposed-vowel scope). `reading_alternatives()`,
  `r_han_patterns()` and `permits_weak_initial(range)` expose them, and
  `recover_parts(&parts)` re-exposes those positions in an existing `split` result without
  changing byte ownership.
- `SyllablePattern::inspect(source)` counts vowel nuclei in one candidate (`None` for empty,
  non-Thai or unparseable text). `inspect_substantial` does the same without counting
  consonant-only silent-mark continuations such as `มพ์` in `พิมพ์`.
- `is_one_syllable_or_more` is a predicate over a single candidate string, and
  `SyllablePattern::is_candidate(source)` is the stricter check on a candidate whose pattern
  you already hold.
- `ThaiScalar::new(character)` classifies one scalar: `is_accepted` is the accepted alphabet, and
  `is_base`, `is_dependent`, `is_leading_vowel`, `is_independent_vowel` and
  `forms_initial_cluster_with` expose its orthographic role.

## Testing

`cargo test -p thai-syllable-rules --features test-support` also runs the architecture tests, which
use the hidden `test_support` module (stage snapshots) that only exists under that feature.

## Cargo features

| Feature | Effect |
| --- | --- |
| `serde` | Implements `Serialize` and `Deserialize` for `Part`, `ReadingAlternative`, `RHanPattern`, `RuleAnalysis`, `SyllablePattern` and `ThaiScalar`. |
| `test-support` | Exposes internal stage snapshots for this repository's integration tests. Not part of the stable API. |
