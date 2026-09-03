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

#[test]
fn keeps_a_coda_undecided_before_a_following_h_nucleus_cluster() {
    let source = "เดือนหน้า";

    assert_eq!(
        text_parts(source, &split(source)),
        vec![
            TextPart::Syllable("เดือ"),
            TextPart::Undecided("น"),
            TextPart::Syllable("หน้า"),
        ]
    );
}

#[test]
fn keeps_an_inherent_final_cluster_tail_ambiguous_before_a_marked_syllable() {
    let source = "ไหลย้อน";

    assert_eq!(
        text_parts(source, &split(source)),
        vec![
            TextPart::Syllable("ไห"),
            TextPart::Undecided("ล"),
            TextPart::Syllable("ย้อน"),
        ]
    );
}

#[test]
fn keeps_cluster_tail_and_following_onset_ambiguous() {
    let source = "ไหนครับ";

    assert_eq!(
        text_parts(source, &split(source)),
        vec![
            TextPart::Syllable("ไห"),
            TextPart::Undecided("น"),
            TextPart::Undecided("ค"),
            TextPart::Syllable("รับ"),
        ],
    );
}

#[test]
fn keeps_tone_marked_garan_boundary_ambiguous() {
    let source = "แกล้งพจน์";

    assert_eq!(
        text_parts(source, &split(source)),
        vec![
            TextPart::Syllable("แกล้"),
            TextPart::Undecided("ง"),
            TextPart::Undecided("พ"),
            TextPart::Syllable("จน์"),
        ]
    );
}

#[test]
fn keeps_requested_compound_examples_structurally_conservative() {
    let cases = [
        (
            "มนสิการ",
            vec![
                TextPart::Syllable("มน"),
                TextPart::Syllable("สิ"),
                TextPart::Syllable("การ"),
            ],
        ),
        (
            "ธรรมสวนะ",
            vec![
                TextPart::Syllable("ธร"),
                TextPart::Undecided("ร"),
                TextPart::Undecided("ม"),
                TextPart::Undecided("ส"),
                TextPart::Undecided("ว"),
                TextPart::Syllable("นะ"),
            ],
        ),
        (
            "กรกฎาคม",
            vec![
                TextPart::Syllable("กร"),
                TextPart::Undecided("ก"),
                TextPart::Syllable("ฎา"),
                TextPart::Syllable("คม"),
            ],
        ),
        ("กรณ", vec![TextPart::Syllable("กรณ")]),
    ];

    for (source, expected) in cases {
        assert_eq!(text_parts(source, &split(source)), expected, "{source}");
    }
}
