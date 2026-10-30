use thai_word_segmenter::{Error, WordSegmenter};

const SAMPLE_TEXTS: [&str; 3] = ["ในปฐมกาลพระวาทะทรงดำรงอยู่", "“การเรียนสำคัญ”", "ฉันกินข้าว สวัสดี!"];

#[test]
fn bundled_model_loads_and_segments_thai_text() -> Result<(), Error> {
    let segmenter = WordSegmenter::new()?;
    let input = "ในปฐมกาลพระวาทะทรงดำรงอยู่";
    let words = segmenter.split(input)?;
    assert_eq!(words.concat(), input);
    assert_eq!(words, vec!["ในปฐมกาล", "พระวาทะ", "ทรงดำรง", "อยู่"]);
    let extracted: Vec<&str> = segmenter
        .ranges(input)?
        .into_iter()
        .map(|range| &input[range])
        .collect();
    assert_eq!(extracted, words);
    Ok(())
}

#[test]
fn quotation_marks_attach_to_their_words() -> Result<(), Error> {
    let segmenter = WordSegmenter::new()?;
    let input = "“การเรียนสำคัญ”";
    let words = segmenter.split(input)?;
    assert_eq!(words.concat(), input);
    assert!(
        words
            .first()
            .is_some_and(|word| word.starts_with('“') && word.chars().count() > 1)
    );
    assert!(words.last().is_some_and(|word| word.ends_with('”')));
    Ok(())
}
