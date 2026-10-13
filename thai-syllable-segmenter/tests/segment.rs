use thai_syllable_rules::ThaiScalar;

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
