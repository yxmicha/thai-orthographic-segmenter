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
