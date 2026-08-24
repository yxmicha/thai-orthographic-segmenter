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

#[test]
fn candidate_preparation_adds_a_terminal_implicit_anchor() {
    let parsed = parsed_atoms("กะตก");
    let prepared = prepared_atoms("กะตก", false);
    assert!(parsed.is_some(), "fixture must be one Thai run");
    assert!(prepared.is_some(), "fixture must be one Thai run");
    let (Some(parsed), Some(prepared)) = (parsed, prepared) else {
        return;
    };

    assert!(prepared.len() < parsed.len());
    assert_eq!(
        prepared.last(),
        Some(&AtomSnapshot {
            range: 6..12,
            has_hard_ending: false,
            has_nucleus: true,
            semantic: SemanticAtomSnapshot {
                onset: OnsetSnapshot::Base('ต'),
                nucleus: NucleusSnapshot::Implicit,
                ending: EndingSnapshot::Open,
                silent_mark: SilentMarkSnapshot::None,
            },
        })
    );
}

#[test]
fn prepared_context_and_planned_ranges_remain_source_relative_and_explicit() {
    let prepared = prepared_run("กะตก", 11, true);
    assert!(prepared.is_some(), "fixture must be one Thai run");
    let Some(prepared) = prepared else {
        return;
    };
    assert_eq!(prepared.source_offset, 11);
    assert!(prepared.followed_by_single_period);

    let planned = planned_parts("กะตกลมเตะ", 7, false);
    assert!(planned.is_some(), "fixture must be one Thai run");
    let Some(planned) = planned else {
        return;
    };
    assert_eq!(
        planned,
        vec![
            TracePart {
                kind: PartKind::Syllable,
                range: 7..13,
            },
            TracePart {
                kind: PartKind::Syllable,
                range: 13..19,
            },
            TracePart {
                kind: PartKind::Undecided,
                range: 19..22,
            },
            TracePart {
                kind: PartKind::Undecided,
                range: 22..25,
            },
            TracePart {
                kind: PartKind::Syllable,
                range: 25..34,
            },
        ]
    );
}

#[test]
fn public_parts_expose_source_preserving_kinds_and_ranges() {
    assert_eq!(
        projected_parts("นะก่กา. /"),
        vec![
            TracePart {
                kind: PartKind::Syllable,
                range: 0..6,
            },
            TracePart {
                kind: PartKind::Invalid,
                range: 6..12,
            },
            TracePart {
                kind: PartKind::Syllable,
                range: 12..18,
            },
            TracePart {
                kind: PartKind::Dot,
                range: 18..19,
            },
            TracePart {
                kind: PartKind::Text,
                range: 19..21,
            },
        ]
    );
}
