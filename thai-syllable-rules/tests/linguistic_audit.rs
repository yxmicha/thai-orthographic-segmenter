mod text_part;
use text_part::{TextPart, text_parts};
use thai_syllable_rules::{SyllablePattern, split};

#[test]
fn bare_carrier_does_not_license_an_onset_only_final() {
    for tail in ["ห", "ฮ", "ฉ", "ฌ", "ผ", "ฝ"] {
        let source = format!("อ{tail}");
        assert_eq!(
            text_parts(&source, &split(&source)),
            vec![TextPart::Undecided("อ"), TextPart::Undecided(tail)],
            "{source}"
        );
    }
    for source in ["อก", "อน", "ออ", "อย่า", "อยู่", "อย่าง", "อยาก"]
    {
        assert_eq!(
            text_parts(source, &split(source)),
            vec![TextPart::Syllable(source)],
            "{source}"
        );
    }
}

#[test]
fn retained_oe_carrier_is_one_vowel_pattern() {
    for source in ["เทอม", "เทอญ"] {
        assert_eq!(
            SyllablePattern::inspect(source).map(|pattern| pattern.vowel_pattern_count()),
            Some(1),
            "{source}"
        );
        assert_eq!(
            text_parts(source, &split(source)),
            vec![TextPart::Syllable(source)],
            "{source}"
        );
    }
}

#[test]
fn lakkhangyao_requires_an_independent_vowel() {
    assert_eq!(
        text_parts("กๅก", &split("กๅก")),
        vec![
            TextPart::Undecided("ก"),
            TextPart::Text("ๅ"),
            TextPart::Undecided("ก")
        ]
    );
    for source in ["ฤๅ", "ฦๅ"] {
        assert_eq!(
            text_parts(source, &split(source)),
            vec![TextPart::Syllable(source)]
        );
    }
}

#[test]
fn conventional_bo_negator_is_not_an_invalid_tone_cell() {
    assert_eq!(text_parts("บ่", &split("บ่")), vec![TextPart::Syllable("บ่")]);
    assert_eq!(
        text_parts("บ่น", &split("บ่น")),
        vec![TextPart::Syllable("บ่น")]
    );
    assert_eq!(text_parts("ก่", &split("ก่")), vec![TextPart::Invalid("ก่")]);
}

#[test]
fn decomposed_am_does_not_accept_a_second_vowel() {
    for (source, syllable, orphan) in [
        ("นํ้าะ", "นํ้า", "ะ"),
        ("กําะ", "กํา", "ะ"),
        ("นํ้าา", "นํ้า", "า"),
        ("กําา", "กํา", "า"),
    ] {
        assert_eq!(
            text_parts(source, &split(source)),
            vec![TextPart::Syllable(syllable), TextPart::Text(orphan)],
            "{source}"
        );
    }
}

#[test]
fn tone_mark_does_not_complete_terminal_ue() {
    for source in ["กื", "กื้", "กื๊", "กื๋", "กื้ก์", "กื้กิ์"]
    {
        assert_eq!(
            text_parts(source, &split(source)),
            vec![TextPart::Invalid(source)],
            "{source}"
        );
    }
    assert_eq!(
        text_parts("พื้นที่", &split("พื้นที่")),
        vec![
            TextPart::Syllable("พื้"),
            TextPart::Undecided("น"),
            TextPart::Syllable("ที่")
        ]
    );
    for source in ["มือ", "ซื้อ", "คืน", "พื้น"] {
        assert_eq!(
            text_parts(source, &split(source)),
            vec![TextPart::Syllable(source)],
            "{source}"
        );
    }
}

#[test]
fn complex_silent_endings_keep_hosts_without_swallowing_neighbors() {
    let source = "ศักดิ์สิทธิ์ พันธุ์ศาสตร์ คอนเสิร์ต";
    let parts = split(source);
    assert_eq!(
        text_parts(source, &parts),
        vec![
            TextPart::Syllable("ศักดิ์"),
            TextPart::Syllable("สิทธิ์"),
            TextPart::Text(" "),
            TextPart::Syllable("พันธุ์"),
            TextPart::Syllable("ศาสตร์"),
            TextPart::Text(" "),
            TextPart::Syllable("คอ"),
            TextPart::Undecided("น"),
            TextPart::Syllable("เสิร์ต"),
        ]
    );
    assert_eq!(
        parts
            .iter()
            .map(|part| part.text(source))
            .collect::<String>(),
        source
    );
}

#[test]
fn carrier_correction_preserves_other_vowel_boundaries() {
    for (source, expected) in [
        (
            "โมอับ",
            vec![TextPart::Syllable("โม"), TextPart::Syllable("อับ")],
        ),
        (
            "เหลืออีก",
            vec![TextPart::Syllable("เหลือ"), TextPart::Syllable("อีก")],
        ),
        (
            "เธอเอง",
            vec![
                TextPart::Syllable("เธ"),
                TextPart::Undecided("อ"),
                TextPart::Syllable("เอง"),
            ],
        ),
        (
            "จมรรอห",
            vec![
                TextPart::Syllable("จม"),
                TextPart::Undecided("ร"),
                TextPart::Undecided("ร"),
                TextPart::Undecided("อ"),
                TextPart::Undecided("ห"),
            ],
        ),
        ("บ่มี", vec![TextPart::Syllable("บ่"), TextPart::Syllable("มี")]),
        (
            "บ่ได้",
            vec![TextPart::Syllable("บ่"), TextPart::Syllable("ได้")],
        ),
    ] {
        assert_eq!(text_parts(source, &split(source)), expected, "{source}");
    }
}

#[test]
fn reduced_ua_spellings_remain_terminal_candidates() {
    for source in ["สวน", "รวม", "บวก", "ดวง", "สวย", "กล้วย", "ช่วย", "ถ้วย"]
    {
        assert_eq!(
            text_parts(source, &split(source)),
            vec![TextPart::Syllable(source)],
            "{source}"
        );
    }
}
