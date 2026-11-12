"""Thai syllable, word and line segmentation with bundled models.

The functions run on the default models:

>>> from thai_orthographic_segmenter import word_tokenize
>>> word_tokenize("ในปฐมกาลพระวาทะทรงดำรงอยู่")
['ในปฐมกาล', 'พระวาทะ', 'ทรงดำรง', 'อยู่']

Syllables and words are slices of the input, so joining them gives the input back. For
repeated work with a custom dictionary or model, build a tokenizer class once and pass it in.
"""

from __future__ import annotations

from collections.abc import Iterable
from functools import cache
from typing import NamedTuple

from . import _native
from ._native import LineBreaker, SegmenterError, SyllableTokenizer, WordTokenizer

__all__ = [
    "LineBreaker",
    "OrthographicPart",
    "SegmenterError",
    "SyllableTokenizer",
    "WordTokenizer",
    "mark_line_breaks",
    "orthographic_parts",
    "syllable_spans",
    "syllable_tokenize",
    "word_spans",
    "word_tokenize",
    "word_tokenize_many",
]


class OrthographicPart(NamedTuple):
    """One part of a rule-based split of text; see `orthographic_parts`."""

    kind: str
    """`"syllable"`, `"undecided"`, `"invalid"`, `"dot"` or `"text"`."""
    text: str
    start: int
    """Code point offset of the first character."""
    end: int
    """Code point offset after the last character."""


@cache
def _default_syllables() -> SyllableTokenizer:
    return SyllableTokenizer()


@cache
def _default_words() -> WordTokenizer:
    return WordTokenizer()


@cache
def _default_breaker() -> LineBreaker:
    return LineBreaker()


def _without_whitespace(words: list[str]) -> list[str]:
    stripped_words = []
    for word in words:
        stripped = word.strip()
        if stripped:
            stripped_words.append(stripped)
    return stripped_words


def syllable_tokenize(text: str, tokenizer: SyllableTokenizer | None = None) -> list[str]:
    """Split `text` into written syllables.

    Non-Thai text comes back as pieces between the syllables. `tokenizer` defaults to a shared
    instance built on first use.
    """
    return (tokenizer or _default_syllables()).tokenize(text)


def syllable_spans(text: str, tokenizer: SyllableTokenizer | None = None) -> list[tuple[int, int]]:
    """Return the `(start, end)` code point offsets of each syllable of `text`."""
    return (tokenizer or _default_syllables()).spans(text)


def word_tokenize(
    text: str,
    keep_whitespace: bool = True,
    tokenizer: WordTokenizer | None = None,
) -> list[str]:
    """Split `text` into words (lexical clusters).

    The model keeps compounds and collocations together, so it returns fewer, longer words than
    annotation-style word segmenters. Whitespace stays attached to a neighbouring word, so the
    words join to the input. With `keep_whitespace=False`, whitespace is stripped from the ends
    of each word and words left empty are dropped, so the words no longer join to the input.
    `tokenizer` defaults to a shared instance built on first use; pass
    `WordTokenizer.with_dictionary(...)` for a custom dictionary.
    """
    words = (tokenizer or _default_words()).tokenize(text)
    return words if keep_whitespace else _without_whitespace(words)


def word_tokenize_many(
    texts: Iterable[str],
    jobs: int | None = None,
    keep_whitespace: bool = True,
    tokenizer: WordTokenizer | None = None,
) -> list[list[str]]:
    """Split every text into words, faster than calling `word_tokenize` on each.

    The texts are scored together on up to `jobs` threads (default: every available core), and the
    result for each text equals `word_tokenize` on it.
    """
    results = (tokenizer or _default_words()).tokenize_many(list(texts), jobs)
    if keep_whitespace:
        return results
    return [_without_whitespace(words) for words in results]


def word_spans(text: str, tokenizer: WordTokenizer | None = None) -> list[tuple[int, int]]:
    """Return the `(start, end)` code point offsets of each word of `text`."""
    return (tokenizer or _default_words()).spans(text)


def mark_line_breaks(text: str, breaker: LineBreaker | None = None) -> str:
    """Insert zero-width spaces and joiners so that text wraps only between words.

    A zero-width space (U+200B) marks each break opportunity between words, a zero-width joiner
    (U+200D) forbids breaks inside words that a dictionary-based line breaker would split, and
    punctuation is attached to its neighbouring word. `breaker` defaults to a shared instance
    built on first use.
    """
    return (breaker or _default_breaker()).mark(text)


def orthographic_parts(text: str) -> list[OrthographicPart]:
    """Split `text` by spelling rules alone, keeping every ambiguity explicit.

    The parts cover the input in order. `"undecided"` parts can belong to the syllable before or
    after them; `syllable_tokenize` decides them in context.
    """
    return [OrthographicPart(*part) for part in _native.orthographic_parts(text)]
