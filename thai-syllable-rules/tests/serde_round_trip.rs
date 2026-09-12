//! Rule results serialize and deserialize losslessly behind the `serde` feature.

use thai_syllable_rules::{Part, RuleAnalysis, SyllablePattern, split};

#[test]
fn parts_round_trip_through_json() -> Result<(), serde_json::Error> {
    let parts = split("กะตกลม.");
    let json = serde_json::to_string(&parts)?;
    assert_eq!(serde_json::from_str::<Vec<Part>>(&json)?, parts);
    Ok(())
}

#[test]
fn analyses_and_patterns_round_trip_through_json() -> Result<(), serde_json::Error> {
    let analysis = RuleAnalysis::new("จมรรอก");
    let json = serde_json::to_string(&analysis)?;
    assert_eq!(serde_json::from_str::<RuleAnalysis>(&json)?, analysis);
    let pattern = SyllablePattern::inspect("เกรียน");
    let json = serde_json::to_string(&pattern)?;
    assert_eq!(
        serde_json::from_str::<Option<SyllablePattern>>(&json)?,
        pattern
    );
    Ok(())
}
