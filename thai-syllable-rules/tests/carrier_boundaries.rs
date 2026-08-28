mod text_part;

use text_part::{TextPart, text_parts};
use thai_syllable_rules::split;

#[test]
fn leaves_a_garan_trailing_base_undecided_before_an_open_carrier() {
    let source = "หงษ์ทอง";

    assert_eq!(
        text_parts(source, &split(source)),
        vec![
            TextPart::Syllable("หงษ์"),
            TextPart::Undecided("ท"),
            TextPart::Syllable("อง"),
        ]
    );
}
