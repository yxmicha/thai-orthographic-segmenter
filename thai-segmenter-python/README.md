# thai-orthographic-segmenter (Python)

Python bindings for the [Thai orthographic segmenter](../README.md)
crates, as plain tokenizer functions (`word_tokenize`, `syllable_tokenize`, ...). The models are
bundled in the wheel, so there is nothing to download, and every call releases the interpreter
lock.

```sh
pip install thai-orthographic-segmenter
```

Python 3.10 or later is required.

```python
from thai_orthographic_segmenter import syllable_tokenize, word_tokenize

word_tokenize("ในปฐมกาลพระวาทะทรงดำรงอยู่")
# ['ในปฐมกาล', 'พระวาทะ', 'ทรงดำรง', 'อยู่']
syllable_tokenize("พระวาทะ")
# ['พระ', 'วา', 'ทะ']
```

## Functions

| Function | Backed by | Returns |
| --- | --- | --- |
| `syllable_tokenize(text)` | `thai-syllable-segmenter` | written syllables, a `list[str]` |
| `word_tokenize(text, keep_whitespace=True)` | `thai-word-segmenter` | words (lexical clusters), a `list[str]` |
| `word_tokenize_many(texts, jobs=None)` | `thai-word-segmenter` | the words of each text, scored together on up to `jobs` threads |
| `syllable_spans(text)`, `word_spans(text)` | the same | `(start, end)` code point offsets, so `text[start:end]` is the token |
| `mark_line_breaks(text)` | `thai-line-segmenter` | text with zero-width spaces at break opportunities and joiners inside words |
| `orthographic_parts(text)` | `thai-syllable-rules` | `OrthographicPart(kind, text, start, end)` from spelling rules alone, with ambiguity left as `"undecided"` |

Syllables and words are slices of the input, so `"".join(tokens) == text`. The functions use
shared default models built on first use (about 5 ms for syllables, 40 ms for words and line
breaks). For anything else, build a tokenizer once and pass it in:

```python
from thai_orthographic_segmenter import WordTokenizer, word_tokenize

tokenizer = WordTokenizer.with_dictionary(["ปฐมกาลพระวาทะ"])  # replaces the bundled dictionary
word_tokenize("ในปฐมกาลพระวาทะทรงดำรงอยู่", tokenizer=tokenizer)
WordTokenizer.load("path/to/artifact")  # config.json, dictionary.txt, model.safetensors
```

`SyllableTokenizer`, `WordTokenizer` and `LineBreaker(icu_dictionary=None)` take the same text
and return the same results as the functions through `tokenize`, `spans` and `mark`.

## Behaviour

- **Granularity**: the word model is trained to under-segment, keeping compounds and
  collocations together, so it returns fewer, longer words than annotation-style word
  segmenters. See the [word segmenter readme](../thai-word-segmenter/README.md) for the
  benchmark scores.
- **One model per level**: there is no choice of engine. A custom dictionary replaces the
  bundled one rather than extending it.
- **Failures**: they raise `SegmenterError`, a `ValueError`. A Thai run over 4,096 scalars is
  rejected by the syllable segmenter.

## Build and test

```sh
uvx maturin develop --manifest-path thai-segmenter-python/Cargo.toml --uv
uv run --with pytest pytest thai-segmenter-python/tests
```

`maturin build --release` produces a wheel. The wheel needs nothing beyond the Python
interpreter that imports it.
