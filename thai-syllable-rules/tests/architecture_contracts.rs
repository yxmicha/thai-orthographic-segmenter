use thai_syllable_rules::test_support::{
    AtomSnapshot, EndingSnapshot, NucleusSnapshot, OnsetSnapshot, PartKind, SemanticAtomSnapshot,
    SilentMarkSnapshot, TracePart, parsed_atoms, planned_parts, prepared_atoms, prepared_run,
    projected_parts,
};

#[test]
fn atomization_records_a_hard_ending_without_resolving_ownership() {
    assert_eq!(
        parsed_atoms("กำ"),
        Some(vec![AtomSnapshot {
            range: 0..6,
            has_hard_ending: true,
            has_nucleus: true,
            semantic: SemanticAtomSnapshot {
                onset: OnsetSnapshot::Base('ก'),
                nucleus: NucleusSnapshot::Rime {
                    explicit: true,
                    carrier: false,
                    completed_ue_carrier: false,
                    glide: false,
                },
                ending: EndingSnapshot::Hard,
                silent_mark: SilentMarkSnapshot::None,
            },
        }])
    );
}

#[test]
fn atomization_preserves_an_orphan_dependent_rime_as_non_nucleus() {
    assert_eq!(
        parsed_atoms("กีา"),
        Some(vec![
            AtomSnapshot {
                range: 0..6,
                has_hard_ending: false,
                has_nucleus: true,
                semantic: SemanticAtomSnapshot {
                    onset: OnsetSnapshot::Base('ก'),
                    nucleus: NucleusSnapshot::Rime {
                        explicit: true,
                        carrier: false,
                        completed_ue_carrier: false,
                        glide: false,
                    },
                    ending: EndingSnapshot::Open,
                    silent_mark: SilentMarkSnapshot::None,
                },
            },
            AtomSnapshot {
                range: 6..9,
                has_hard_ending: false,
                has_nucleus: false,
                semantic: SemanticAtomSnapshot {
                    onset: OnsetSnapshot::DependentMark,
                    nucleus: NucleusSnapshot::OrphanRime,
                    ending: EndingSnapshot::Open,
                    silent_mark: SilentMarkSnapshot::None,
                },
            },
        ])
    );
}
