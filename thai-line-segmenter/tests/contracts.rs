use thai_line_segmenter::{Error, LineSegmenter, ZERO_WIDTH_JOINER, ZERO_WIDTH_SPACE};

#[test]
fn line_segmenter_initializes_and_normalizes() -> Result<(), Error> {
    let segmenter = LineSegmenter::new()?;
    let input = "“การเรียนสำคัญ”";
    let normalized = segmenter.normalize_line_boundaries(input)?;
    let clean = normalized
        .replace([ZERO_WIDTH_SPACE, ZERO_WIDTH_JOINER], "")
        .replace('\u{00a0}', " ");
    assert_eq!(clean, input);
    Ok(())
}

#[test]
fn line_segmenter_attaches_punctuation_and_inserts_zwj() -> Result<(), Error> {
    let segmenter = LineSegmenter::new()?;
    let normalized = segmenter.normalize_line_boundaries("“ในปฐมกาล”")?;
    assert!(normalized.contains(ZERO_WIDTH_JOINER));
    Ok(())
}

#[test]
fn line_segmenter_inserts_zwsp_at_word_break_opportunities() -> Result<(), Error> {
    let segmenter = LineSegmenter::new()?;
    let normalized = segmenter.normalize_line_boundaries("ในปฐมกาลพระวาทะทรงดำรงอยู่")?;
    assert!(normalized.contains(ZERO_WIDTH_SPACE));
    Ok(())
}
