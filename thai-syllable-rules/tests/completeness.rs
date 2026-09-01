mod text_part;

use text_part::{TextPart, text_parts};
use thai_syllable_rules::split;

#[test]
fn required_rime_endings_are_not_supplied_by_the_end_of_input() {
    for source in ["กั", "เกิ", "เก็", "กื"] {
        let parts = split(source);
        assert_eq!(
            text_parts(source, &parts),
            vec![TextPart::Invalid(source)],
            "{source}"
        );
    }
}

#[test]
fn hostless_silent_cells_are_not_standalone_syllables() {
    for source in ["ธุ์", "กิ์", "ก์"] {
        assert_eq!(
            text_parts(source, &split(source)),
            vec![TextPart::Invalid(source)],
            "{source}"
        );
    }
}

#[test]
fn compound_vowel_frames_need_all_written_constituents() {
    for source in ["เกี", "เกืก"] {
        assert_eq!(
            text_parts(source, &split(source)),
            vec![TextPart::Invalid(source)],
            "{source}"
        );
    }
    for source in ["เกีย", "เกือ"] {
        assert_eq!(
            text_parts(source, &split(source)),
            vec![TextPart::Syllable(source)],
            "{source}"
        );
    }
}

#[test]
fn required_coda_cannot_steal_an_onset_that_has_its_own_spacing_vowel() {
    let source = "กักา";
    let parts = split(source);
    assert_eq!(
        text_parts(source, &parts),
        vec![TextPart::Invalid("กั"), TextPart::Syllable("กา"),]
    );
}

#[test]
fn adopting_a_prefix_does_not_supply_a_missing_rime_ending() {
    let source = "ขกั";
    let parts = split(source);
    assert_eq!(text_parts(source, &parts), vec![TextPart::Invalid("ขกั")]);
}

#[test]
fn completing_material_remains_available_before_invalidity_is_decided() {
    for source in ["กัด", "เกิน", "เด็ก", "คืน", "มือ", "ตัว", "ก็", "บัติ"]
    {
        let parts = split(source);
        assert_eq!(
            text_parts(source, &parts),
            vec![TextPart::Syllable(source)],
            "{source}"
        );
    }
}

#[test]
fn required_rimes_take_the_adjacent_coda_before_a_carrier_remainder() {
    for (source, first, second) in [
        ("ส่งออก", "ส่ง", "ออก"),
        ("เงินออม", "เงิน", "ออม"),
        ("เปิดออก", "เปิด", "ออก"),
    ] {
        assert_eq!(
            text_parts(source, &split(source)),
            vec![TextPart::Syllable(first), TextPart::Syllable(second)],
            "{source}"
        );
    }
}

#[test]
fn complete_leading_vowels_do_not_swallow_the_next_carrier_rime() {
    for (source, first, second) in [
        ("เสรี", "เส", "รี"),
        ("เหลืออีก", "เหลือ", "อีก"),
        ("โมอับ", "โม", "อับ"),
    ] {
        assert_eq!(
            text_parts(source, &split(source)),
            vec![TextPart::Syllable(first), TextPart::Syllable(second)],
            "{source}"
        );
    }
}
