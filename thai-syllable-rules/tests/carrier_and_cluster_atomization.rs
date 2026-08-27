mod text_part;

use text_part::{TextPart, text_parts};
use thai_syllable_rules::split;

#[test]
fn attaches_an_onset_only_singleton_to_a_following_cluster() {
    let cases = [
        (
            "แกหรอก",
            vec![TextPart::Syllable("แก"), TextPart::Syllable("หรอก")],
        ),
        (
            "อีกหลาย",
            vec![TextPart::Syllable("อีก"), TextPart::Syllable("หลาย")],
        ),
        (
            "อกหลาย",
            vec![TextPart::Syllable("อก"), TextPart::Syllable("หลาย")],
        ),
    ];

    for (source, expected) in cases {
        assert_eq!(text_parts(source, &split(source)), expected, "{source}");
    }
}
