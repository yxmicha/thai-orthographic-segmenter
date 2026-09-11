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

#[test]
fn shared_consonants_and_preposed_scope_refer_to_original_bytes() {
    let shared = RuleAnalysis::new("กิจการ");
    assert!(shared.reading_alternatives().iter().any(|alternative| matches!(alternative,
        ReadingAlternative::SharedConsonant { source_range, boundary } if *source_range == (6..9) && *boundary == 9
    )));
    let scoped = RuleAnalysis::new("เกษม");
    assert!(scoped.reading_alternatives().iter().any(|alternative| matches!(alternative,
        ReadingAlternative::LeadingVowelScope { vowel_range, consonant_range, following_onset_range, boundary }
        if *vowel_range == (0..3) && *consonant_range == (3..6) && *following_onset_range == (6..9) && *boundary == 6
    )));
    let parts = scoped.recover_parts(&split("เกษม"));
    assert!(!parts.iter().any(|part| part.byte_range().end == 3));
}

#[test]
fn r_han_recovery_keeps_the_nucleus_together_and_exposes_context() {
    for source in ["กรรไกร", "ภรรยา", "สรรเสริญ", "สรรพคุณ"]
    {
        let analysis = RuleAnalysis::new(source);
        let parts = analysis.recover_parts(&split(source));
        assert_eq!(
            parts
                .iter()
                .map(|part| part.text(source))
                .collect::<String>(),
            source
        );
        assert!(
            parts.iter().any(|part| part.byte_range() == (0..9)),
            "{source}"
        );
        assert!(
            analysis
                .reading_alternatives()
                .iter()
                .any(|alternative| matches!(
                    alternative,
                    ReadingAlternative::SharedConsonant { .. }
                )),
            "{source}"
        );
    }
}

#[test]
fn retained_oe_has_one_nucleus_in_context() {
    for source in ["เทอมนี้", "เทอมหน้า", "เทอญนะ"] {
        assert_eq!(
            SyllablePattern::inspect(source).map(|pattern| pattern.vowel_pattern_count()),
            Some(2),
            "{source}"
        );
    }
    assert_eq!(split("โมอับ"), [Part::Syllable(0..6), Part::Syllable(6..15)]);
}

#[test]
fn recovery_preserves_mixed_text_barriers_and_utf8_ranges() {
    let source = "🦀ขนม ก่้ ภรรยา";
    let analysis = RuleAnalysis::new(source);
    let parts = analysis.recover_parts(&split(source));
    let mut end = 0;
    for part in &parts {
        let range = part.byte_range();
        assert_eq!(range.start, end);
        assert!(range.start < range.end);
        assert!(source.is_char_boundary(range.start) && source.is_char_boundary(range.end));
        end = range.end;
    }
    assert_eq!(end, source.len());
    assert_eq!(
        parts
            .iter()
            .map(|part| part.text(source))
            .collect::<String>(),
        source
    );
    assert!(
        parts
            .iter()
            .any(|part| matches!(part, Part::Invalid(_)) && part.text(source) == "ก่้")
    );
}
