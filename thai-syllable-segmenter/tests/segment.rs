use thai_syllable_rules::ThaiScalar;
use thai_syllable_segmenter::{Error, SyllableSegmenter};

#[test]
fn bundled_vocabulary_contains_only_thai_and_dot() {
    let characters = include_str!("../model/characters.txt");
    assert!(characters.lines().any(|line| line == "."));
    for entry in characters
        .lines()
        .chain(include_str!("../model/ngrams.txt").lines())
    {
        assert!(
            entry
                .chars()
                .all(|character| character == '.' || ThaiScalar::new(character).is_accepted()),
            "unexpected vocabulary entry: {entry:?}"
        );
    }
}

#[test]
fn bundled_student_splits_thai_and_preserves_other_text() -> Result<(), Error> {
    let segmenter = SyllableSegmenter::new()?;
    assert_eq!(
        segmenter.split("ฉันกินข้าว สวัสดี!")?,
        ["ฉัน", "กิน", "ข้าว", " ", "สวัส", "ดี", "!"]
    );
    assert_eq!(segmenter.split("")?, [] as [&str; 0]);
    for source in ["  English 🙂 ", "่", "ไทย🙂text", "เกี๊ยว\n"] {
        let pieces = segmenter.split(source)?;
        assert!(!pieces.iter().any(|piece| piece.is_empty()), "{source:?}");
        assert_eq!(pieces.concat(), source);
    }
    let over_limit = "ก".repeat(4097);
    assert!(matches!(
        segmenter.split(&over_limit),
        Err(Error::ThaiRunTooLong {
            scalars: 4097,
            limit: 4096
        })
    ));
    Ok(())
}

#[test]
fn non_thai_spans_are_preserved_and_do_not_limit_or_contextualize_thai_runs() -> Result<(), Error> {
    let segmenter = SyllableSegmenter::new()?;
    let latin = "a".repeat(4_100);
    let non_thai = format!("{latin}🙂!");
    assert_eq!(segmenter.split(&non_thai)?, [non_thai.as_str()]);

    let thai_run = "สวัสดี";
    let expected = segmenter.split(thai_run)?;
    let mixed = format!("{non_thai}{thai_run} / English");
    let mut pieces = vec![non_thai.as_str()];
    pieces.extend(expected);
    pieces.push(" / English");
    assert_eq!(segmenter.split(&mixed)?, pieces);

    let mut separate_runs = segmenter.split("วู้วววว")?;
    separate_runs.push(" ");
    separate_runs.extend(segmenter.split("ลดทุกวันนนนน")?);
    assert_eq!(segmenter.split("วู้วววว ลดทุกวันนนนน")?, separate_runs);

    // The frozen student without lexical evidence merges this input; a lexicon
    // override in the original model splits it into พี่ | มด | คะ.
    assert_eq!(segmenter.split("!!!พี่มดคะ  ")?, ["!!!", "พี่มด", "คะ", "  "]);
    Ok(())
}
