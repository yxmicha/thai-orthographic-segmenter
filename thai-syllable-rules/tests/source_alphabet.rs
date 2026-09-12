mod text_part;

use text_part::{TextPart, text_parts};
use thai_syllable_rules::ThaiScalar;
use thai_syllable_rules::split;

#[test]
fn excludes_thai_digits_baht_sign_and_punctuation_from_thai_runs() {
    assert!(ThaiScalar::new('ก').is_accepted()); // ก
    assert!(ThaiScalar::new('์').is_accepted()); // อ์
    assert!(!ThaiScalar::new('ฯ').is_accepted()); // ฯ
    assert!(!ThaiScalar::new('฿').is_accepted()); // ฿
    assert!(!ThaiScalar::new('ๆ').is_accepted()); // ๆ
    assert!(!ThaiScalar::new('๏').is_accepted()); // ๏
    assert!(!ThaiScalar::new('๐').is_accepted()); // ๐
    assert!(!ThaiScalar::new('๙').is_accepted()); // ๙
    assert!(!ThaiScalar::new('๚').is_accepted()); // ๚
    assert!(!ThaiScalar::new('๛').is_accepted()); // ๛

    assert_eq!(
        text_parts("ก๑฿ข", &split("ก๑฿ข")),
        vec![
            TextPart::Undecided("ก"),
            TextPart::Text("๑฿"),
            TextPart::Undecided("ข"),
        ]
    );
}

// This keeps the regression corpus available without making the test depend on a data path.
