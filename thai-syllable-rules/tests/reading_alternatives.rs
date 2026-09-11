use thai_syllable_rules::{Part, ReadingAlternative, RuleAnalysis, SyllablePattern, split};

#[test]
fn r_han_shared_references_are_emitted_once() {
    for (source, shared_range) in [("สรรเสริญ", 6..9), ("สรรพคุณ", 9..12)]
    {
        let analysis = RuleAnalysis::new(source);
        let count = analysis
            .reading_alternatives()
            .iter()
            .filter(|alternative| {
                matches!(alternative,
                    ReadingAlternative::SharedConsonant { source_range, .. }
                        if *source_range == shared_range
                )
            })
            .count();
        assert_eq!(count, 1, "{source}");
    }
}
