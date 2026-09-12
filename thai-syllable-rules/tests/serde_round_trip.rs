//! Rule results serialize and deserialize losslessly behind the `serde` feature.

use thai_syllable_rules::{Part, RuleAnalysis, SyllablePattern, split};

#[test]
fn parts_round_trip_through_json() -> Result<(), serde_json::Error> {
    let parts = split("กะตกลม.");
    let json = serde_json::to_string(&parts)?;
    assert_eq!(serde_json::from_str::<Vec<Part>>(&json)?, parts);
    Ok(())
}
