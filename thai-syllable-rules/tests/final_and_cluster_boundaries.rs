mod text_part;

use text_part::{TextPart, text_parts};
use thai_syllable_rules::split;

#[test]
fn leaves_the_final_r_cluster_ambiguous_before_the_following_syllable() {
    assert_eq!(
        text_parts("จักรยาน", &split("จักรยาน")),
        vec![
            TextPart::Syllable("จัก"),
            TextPart::Undecided("ร"),
            TextPart::Syllable("ยาน"),
        ]
    );
}

#[test]
fn leaves_a_leading_cluster_coda_ambiguous_before_a_following_rime() {
    let source = "เครดิต";

    assert_eq!(
        text_parts(source, &split(source)),
        vec![
            TextPart::Syllable("เค"),
            TextPart::Undecided("ร"),
            TextPart::Syllable("ดิต"),
        ]
    );
}

#[test]
fn leaves_an_ordinary_leading_cluster_coda_ambiguous_before_a_leading_vowel() {
    let source = "โพรเท็ค";

    assert_eq!(
        text_parts(source, &split(source)),
        vec![
            TextPart::Syllable("โพ"),
            TextPart::Undecided("ร"),
            TextPart::Syllable("เท็ค"),
        ]
    );
}

#[test]
fn keeps_both_final_r_boundaries_ambiguous_in_a_chained_spelling() {
    let source = "บัตรเครดิต";

    assert_eq!(
        text_parts(source, &split(source)),
        vec![
            TextPart::Syllable("บัต"),
            TextPart::Undecided("ร"),
            TextPart::Syllable("เค"),
            TextPart::Undecided("ร"),
            TextPart::Syllable("ดิต"),
        ]
    );
}

#[test]
fn completes_a_terminal_leading_cluster_as_two_candidates() {
    assert_eq!(
        text_parts("เบรก", &split("เบรก")),
        vec![TextPart::Syllable("เบ"), TextPart::Syllable("รก")]
    );
}
