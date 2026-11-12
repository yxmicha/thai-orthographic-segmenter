"""Behaviour of the `thai_orthographic_segmenter` package against its bundled models."""

from concurrent.futures import ThreadPoolExecutor

import pytest

import thai_orthographic_segmenter as ts

JOHN = "ในปฐมกาลพระวาทะทรงดำรงอยู่"
MIXED = "😀 ในปฐมกาล, พระวาทะ (Word) ทรงดำรงอยู่ 28:12"
ZERO_WIDTH = {"​", "‍"}


def test_word_tokenize_matches_the_documented_example() -> None:
    assert ts.word_tokenize(JOHN) == ["ในปฐมกาล", "พระวาทะ", "ทรงดำรง", "อยู่"]


@pytest.mark.parametrize("text", [JOHN, MIXED, "", "abc", "   "])
def test_syllables_and_words_reconstruct_the_input(text: str) -> None:
    assert "".join(ts.syllable_tokenize(text)) == text
    assert "".join(ts.word_tokenize(text)) == text


@pytest.mark.parametrize("spans_of", [ts.syllable_spans, ts.word_spans])
def test_spans_are_code_point_offsets_of_the_tokens(spans_of) -> None:
    tokens_of = {
        ts.syllable_spans: ts.syllable_tokenize,
        ts.word_spans: ts.word_tokenize,
    }
    spans = spans_of(MIXED)
    assert [MIXED[start:end] for start, end in spans] == tokens_of[spans_of](MIXED)
    assert spans[0][0] == 0
    assert spans[-1][1] == len(MIXED)


def test_keep_whitespace_false_strips_whitespace_from_the_words() -> None:
    words = ts.word_tokenize(MIXED, keep_whitespace=False)
    assert words == [word.strip() for word in ts.word_tokenize(MIXED)]
    assert all(word and word == word.strip() for word in words)
    assert ts.word_tokenize("   ", keep_whitespace=False) == []
    assert ts.word_tokenize_many([MIXED], keep_whitespace=False) == [words]


def test_tokenize_many_equals_tokenizing_each_text() -> None:
    texts = [JOHN, MIXED, "", "abc", JOHN * 3]
    expected = [ts.word_tokenize(text) for text in texts]
    assert ts.word_tokenize_many(texts) == expected
    assert ts.word_tokenize_many(texts, jobs=1) == expected
    assert ts.word_tokenize_many(iter(texts), jobs=3) == expected


def test_tokenize_many_batches_long_inputs() -> None:
    texts = [JOHN * 40] * 200
    assert ts.word_tokenize_many(texts, jobs=2) == [ts.word_tokenize(texts[0])] * 200


def test_tokenize_many_rejects_zero_jobs() -> None:
    with pytest.raises(ValueError, match="jobs"):
        ts.word_tokenize_many([JOHN], jobs=0)


def test_custom_dictionary_never_splits_its_words() -> None:
    tokenizer = ts.WordTokenizer.with_dictionary(["ปฐมกาลพระวาทะ"])
    words = ts.word_tokenize(JOHN, tokenizer=tokenizer)
    assert any("ปฐมกาลพระวาทะ" in word for word in words)


def test_dictionary_with_non_thai_entry_is_rejected() -> None:
    with pytest.raises(ts.SegmenterError, match="non-Thai"):
        ts.WordTokenizer.with_dictionary(["abc"])


def test_missing_model_directory_is_a_segmenter_error() -> None:
    with pytest.raises(ts.SegmenterError):
        ts.WordTokenizer.load("/nonexistent/thai-model")


def test_segmenter_error_is_a_value_error() -> None:
    assert issubclass(ts.SegmenterError, ValueError)


def test_too_long_thai_run_is_a_segmenter_error() -> None:
    with pytest.raises(ts.SegmenterError, match="4096|long"):
        ts.syllable_tokenize("ก" * 5000)


def test_line_breaks_only_add_zero_width_characters() -> None:
    marked = ts.mark_line_breaks(JOHN)
    assert "".join(char for char in marked if char not in ZERO_WIDTH) == JOHN
    assert "​" in marked


def test_line_breaker_accepts_an_icu_dictionary() -> None:
    breaker = ts.LineBreaker("ปฐมกาล\nพระวาทะ\n")
    marked = ts.mark_line_breaks(JOHN, breaker=breaker)
    assert "".join(char for char in marked if char not in ZERO_WIDTH) == JOHN


def test_orthographic_parts_cover_the_input_in_order() -> None:
    parts = ts.orthographic_parts(MIXED)
    assert "".join(part.text for part in parts) == MIXED
    assert [part.start for part in parts[1:]] == [part.end for part in parts[:-1]]
    assert all(MIXED[part.start : part.end] == part.text for part in parts)
    assert {part.kind for part in parts} <= {
        "syllable",
        "undecided",
        "invalid",
        "dot",
        "text",
    }
    assert any(part.kind == "syllable" for part in parts)


def test_orthographic_parts_keep_ambiguity_explicit() -> None:
    assert any(part.kind == "undecided" for part in ts.orthographic_parts("กะตกลม"))


def test_non_string_input_is_a_type_error() -> None:
    with pytest.raises(TypeError):
        ts.word_tokenize(b"bytes")  # type: ignore[arg-type]


def test_calls_from_several_threads_agree() -> None:
    with ThreadPoolExecutor(max_workers=4) as pool:
        results = list(pool.map(ts.word_tokenize, [MIXED] * 8))
    assert results == [ts.word_tokenize(MIXED)] * 8
