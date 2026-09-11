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

#[test]
fn r_han_nuclei_are_counted_without_inventing_a_pronunciation() {
    for (source, count) in [
        ("กรรม", 1),
        ("กรรมธรรม", 2),
        ("ภรรยา", 2),
        ("สรรเสริญ", 2),
        ("สรรพคุณ", 2),
    ] {
        let analysis = RuleAnalysis::new(source);
        assert!(!analysis.r_han_patterns().is_empty(), "{source}");
        assert_eq!(
            SyllablePattern::inspect(source).map(|pattern| pattern.vowel_pattern_count()),
            Some(count),
            "{source}"
        );
    }
}

#[test]
fn weak_initials_are_recoverable_while_whole_clusters_remain_available() {
    for source in ["ขนม", "ถนน", "สนุก", "ตลาด", "จมูก", "อร่อย"]
    {
        let analysis = RuleAnalysis::new(source);
        assert!(analysis.permits_weak_initial(&(0..3)), "{source}");
        let recovered = analysis.recover_parts(&split(source));
        assert!(recovered.iter().any(|part| part.byte_range() == (0..3)));
        assert_eq!(
            recovered
                .iter()
                .map(|part| part.text(source))
                .collect::<String>(),
            source
        );
    }
    let analysis = RuleAnalysis::new("ครา");
    assert_eq!(split("ครา"), [Part::Syllable(0..9)]);
    assert_eq!(
        analysis
            .recover_parts(&split("ครา"))
            .iter()
            .map(|part| part.text("ครา"))
            .collect::<String>(),
        "ครา"
    );
}
