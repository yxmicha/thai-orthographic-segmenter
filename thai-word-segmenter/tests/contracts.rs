use std::num::NonZeroUsize;

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

#[test]
fn custom_dictionary_protects_specified_word() -> Result<(), Error> {
    let segmenter = WordSegmenter::with_dictionary("การเรียนรู้\n")?;
    assert_eq!(segmenter.split("การเรียนรู้")?, vec!["การเรียนรู้"]);
    Ok(())
}

#[test]
fn splits_as_soon_as_non_thai_excluding_dots_begins() -> Result<(), Error> {
    let segmenter = WordSegmenter::new()?;
    assert_eq!(segmenter.split("ปฐก.28:12")?, vec!["ปฐก.", "28:12"]);
    assert_eq!(segmenter.split("ลง’1:51")?, vec!["ลง’", "1:51"]);
    Ok(())
}

#[test]
fn one_segmenter_gives_identical_results_on_every_thread() -> Result<(), Error> {
    let segmenter = WordSegmenter::new()?;
    let split_all = || {
        SAMPLE_TEXTS
            .iter()
            .map(|text| segmenter.split(text).ok())
            .collect::<Vec<_>>()
    };
    let expected = split_all();
    assert!(expected.iter().all(Option::is_some));
    std::thread::scope(|scope| {
        let workers = (0..4).map(|_| scope.spawn(split_all)).collect::<Vec<_>>();
        for worker in workers {
            assert_eq!(worker.join().expect("worker thread panicked"), expected);
        }
    });
    Ok(())
}

#[test]
fn split_many_matches_splitting_each_text_alone() -> Result<(), Error> {
    let segmenter = WordSegmenter::new()?;
    // More than one inference window of syllables, plus empty, non-Thai, and mixed texts.
    let long = SAMPLE_TEXTS[0].repeat(10);
    let texts = [
        SAMPLE_TEXTS[0],
        "",
        "English only 123",
        SAMPLE_TEXTS[1],
        long.as_str(),
        SAMPLE_TEXTS[2],
        "ปฐก.28:12",
    ];
    let batched = segmenter.split_many(&texts)?;
    assert_eq!(batched.len(), texts.len());
    for (text, words) in texts.iter().zip(&batched) {
        assert_eq!(words, &segmenter.split(text)?);
        assert_eq!(words.concat(), *text);
    }
    assert_eq!(segmenter.split_many(&[])?, [] as [std::vec::Vec<&str>; 0]);
    let ranges = segmenter.ranges_many(&texts)?;
    for (text, ranges) in texts.iter().zip(ranges) {
        assert_eq!(ranges, segmenter.ranges(text)?);
    }
    Ok(())
}

#[test]
fn worker_threads_give_the_same_result_as_one_thread() -> Result<(), Error> {
    let segmenter = WordSegmenter::new()?;
    // Enough text for several groups, so workers must claim more than one.
    let long = SAMPLE_TEXTS[0].repeat(4);
    let texts = std::iter::repeat_n(long.as_str(), 8)
        .chain(SAMPLE_TEXTS)
        .chain(["", "English"])
        .collect::<Vec<_>>();
    let expected = segmenter.split_many(&texts)?;
    for threads in [1, 3, 100] {
        let threads = NonZeroUsize::new(threads).expect("nonzero thread count");
        assert_eq!(segmenter.split_many_on(&texts, threads)?, expected);
    }
    let one = NonZeroUsize::MIN;
    assert_eq!(
        segmenter.split_many_on(&[], one)?,
        [] as [std::vec::Vec<&str>; 0]
    );
    Ok(())
}
