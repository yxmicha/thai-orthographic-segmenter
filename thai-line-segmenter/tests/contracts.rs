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
