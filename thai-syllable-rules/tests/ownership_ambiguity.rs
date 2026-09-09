use thai_syllable_rules::{Part, split};

#[test]
fn leaves_coda_and_initial_cluster_ownership_undecided() {
    let source = "ทุกรุ่น";
    let parts = split(source);

    assert_eq!(
        parts,
        vec![
            Part::Syllable(0..6),
            Part::Undecided(6..9),
            Part::Syllable(9..21),
        ]
    );
}
