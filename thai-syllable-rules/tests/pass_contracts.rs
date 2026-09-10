mod text_part;

use text_part::{TextPart, text_parts};
use thai_syllable_rules::split;

#[test]
fn source_scan_pass_preserves_non_thai_runs_as_hard_boundaries() {
    let source = "กะ English มเหสี";

    assert_eq!(
        text_parts(source, &split(source)),
        vec![
            TextPart::Syllable("กะ"),
            TextPart::Text(" English "),
            TextPart::Syllable("มเห"),
            TextPart::Syllable("สี"),
        ]
    );
}

#[test]
fn thai_parse_pass_keeps_dependent_marks_inside_the_local_unit() {
    let source = "เครื่อง";

    assert_eq!(
        text_parts(source, &split(source)),
        vec![TextPart::Syllable("เครื่อง")]
    );
}
