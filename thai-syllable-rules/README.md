# thai-syllable-rules

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

