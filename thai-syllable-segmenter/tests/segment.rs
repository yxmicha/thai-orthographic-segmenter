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
