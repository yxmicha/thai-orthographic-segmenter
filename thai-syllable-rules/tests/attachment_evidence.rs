mod text_part;

use text_part::{TextPart, text_parts};
use thai_syllable_rules::split;

#[test]
fn required_coda_takes_a_following_onset_when_its_remainder_is_complete() {
    for (source, completed) in [
        ("ก้บอก", "ก้บ"),
        ("ก่บอก", "ก่บ"),
        ("ก๊บอก", "ก๊บ"),
        ("ก๋บอก", "ก๋บ"),
    ] {
        assert_eq!(
            text_parts(source, &split(source)),
            vec![TextPart::Syllable(completed), TextPart::Syllable("อก"),],
            "{source}"
        );
    }
}

#[test]
fn required_coda_completion_is_independent_of_a_preceding_syllable() {
    assert_eq!(
        text_parts("นะก้บอก", &split("นะก้บอก")),
        vec![
            TextPart::Syllable("นะ"),
            TextPart::Syllable("ก้บ"),
            TextPart::Syllable("อก"),
        ]
    );
}

#[test]
fn short_i_keeps_a_final_r_cluster_onset_undecided() {
    for (source, following) in [("อิสรา", "รา"), ("อิสรอ", "รอ")] {
        let parts = split(source);

        assert_eq!(
            text_parts(source, &parts),
            vec![
                TextPart::Syllable("อิ"),
                TextPart::Undecided("ส"),
                TextPart::Syllable(following),
            ],
            "{source}",
        );
    }
}

#[test]
fn silent_mark_leaves_a_forced_following_implicit_pair_intact() {
    let source = "ศิลป์คน";
    let parts = split(source);

    assert_eq!(
        text_parts(source, &parts),
        vec![TextPart::Syllable("ศิลป์"), TextPart::Syllable("คน"),],
    );
}

#[test]
fn internal_silent_mark_preserves_a_competing_coda_and_onset() {
    let source = "ฟิล์มคน";
    let parts = split(source);

    assert_eq!(
        text_parts(source, &parts),
        vec![
            TextPart::Syllable("ฟิล์"),
            TextPart::Undecided("ม"),
            TextPart::Syllable("คน"),
        ],
    );
}

#[test]
fn terminal_internal_silence_retains_its_pronounced_final() {
    let source = "ฟิล์ม ฟอร์ม";
    let parts = split(source);

    assert_eq!(
        text_parts(source, &parts),
        vec![
            TextPart::Syllable("ฟิล์ม"),
            TextPart::Text(" "),
            TextPart::Syllable("ฟอร์ม"),
        ],
    );
}

#[test]
fn deferred_silent_coda_keeps_competing_implicit_groupings_visible() {
    for source in ["ฟิล์มตกลมกะ", "ฟอร์มตกลมกะ"] {
        let parts = split(source);
        let prefix = if source.starts_with("ฟิล์") {
            "ฟิล์"
        } else {
            "ฟอร์"
        };

        assert_eq!(
            text_parts(source, &parts),
            vec![
                TextPart::Syllable(prefix),
                TextPart::Undecided("ม"),
                TextPart::Undecided("ต"),
                TextPart::Undecided("ก"),
                TextPart::Undecided("ล"),
                TextPart::Undecided("ม"),
                TextPart::Syllable("กะ"),
            ],
            "{source}",
        );
    }
}

#[test]
fn an_existing_coda_is_not_reopened_by_a_silent_suffix() {
    let source = "สัตว์ตกลมกะ";
    let parts = split(source);

    assert_eq!(
        text_parts(source, &parts),
        vec![
            TextPart::Syllable("สัตว์"),
            TextPart::Syllable("ตก"),
            TextPart::Undecided("ล"),
            TextPart::Undecided("ม"),
            TextPart::Syllable("กะ"),
        ],
    );
}
