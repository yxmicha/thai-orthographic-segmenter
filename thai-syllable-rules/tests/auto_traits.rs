//! Public types are `Send + Sync`, `Debug` and, where they are plain data, hashable and ordered.

use thai_syllable_rules::{
    Part, RHanPattern, ReadingAlternative, RuleAnalysis, SyllablePattern, ThaiScalar,
};

fn assert_send_sync_debug<T: Send + Sync + std::fmt::Debug>() {}
fn assert_data<T: Clone + Eq + std::hash::Hash + Send + Sync + std::fmt::Debug>() {}

#[test]
fn public_types_are_thread_safe_and_inspectable() {
    assert_data::<Part>();
    assert_data::<ReadingAlternative>();
    assert_data::<RHanPattern>();
    assert_data::<RuleAnalysis>();
    assert_data::<SyllablePattern>();
    assert_data::<ThaiScalar>();
    assert_send_sync_debug::<ThaiScalar>();
}

#[test]
fn scalars_convert_from_and_to_characters() {
    let scalar = ThaiScalar::from('ก');
    assert_eq!(char::from(scalar), 'ก');
    assert_eq!(scalar, ThaiScalar::new('ก'));
    assert!(ThaiScalar::new('ก') < ThaiScalar::new('ข'));
}
