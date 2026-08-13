use crate::atom_sequence::AtomSequence;
use crate::character::ThaiScalar;
use crate::resolution::ResolutionState;

impl ResolutionState<'_> {
    // Rule: only the short ั, ิ, and ุ rimes enter the unmarked final-ร pass.
    // Reusing that same local evidence in resolution prevents a different hard
    // ending such as ำ from being treated as if its following `ร` could attach
    // backward. Examples: `มันรสเหมือน` exposes `ร` and `ส`, while `ทำรถไฟ`
    // keeps the certain `รถ` candidate intact.
    fn preceding_atom_allows_unmarked_final_r(&self, start: usize) -> bool {
        let Some(previous) = start.checked_sub(1) else {
            return false;
        };
        let range = self.ownership.atom_range(previous);
        self.characters().iter().any(|character| {
            if character.start < range.start || character.end > range.end {
                return false;
            }
            matches!(
                character.character,
                '\u{0e31}' // อั
                    | '\u{0e34}' // อิ
                    | '\u{0e38}' // อุ
            )
        })
    }

    /// Resolves the bare run before a following syllable when it ends in a deferred silent coda,
    /// repeats a carrier onset, forms an initial cluster, or is a singleton that cannot attach
    /// left. Returns whether it handled the run.
    fn resolve_leading_cases(&mut self, start: usize, end: usize) -> bool {
        let deferred_silent_coda = start
            .checked_sub(1)
            .is_some_and(|previous| self.atoms()[previous].deferred_silent_coda);
        let unresolved = &self.atoms()[start..end];
        if deferred_silent_coda
            && AtomSequence(unresolved)
                .implicit_syllable_candidates(self.source_offset())
                .is_some()
            && AtomSequence(&unresolved[1..])
                .implicit_syllable_candidates(self.source_offset())
                .is_some()
        {
            // Both complete coverings survive: the first base may close
            // the silent-mark spelling, or start an implicit syllable.
            // Preserve their different groupings instead of fixing the
            // first implicit pair and erasing the scanner's alternative.
            for atom_index in start..end {
                self.ownership
                    .push_undecided(self.ownership.atom_absolute_range(atom_index));
            }
            return true;
        }
        if self.expose_repeated_carrier_onset(start, end) {
            return true;
        }
        let is_two_atom_initial_cluster = end - start == 2
            && self.atoms()[start]
                .first_base()
                .zip(self.atoms()[start + 1].first_base())
                .is_some_and(|(first, second)| {
                    ThaiScalar(first).forms_initial_cluster_with(ThaiScalar(second))
                });
        let first_atom_can_be_final_r = self.atoms()[start]
            .first_base()
            .is_some_and(|character| ThaiScalar(character).is_final_r());
        if end - start == 2
            && first_atom_can_be_final_r
            && self.preceding_atom_allows_unmarked_final_r(start)
            && !is_two_atom_initial_cluster
            && self.atoms()[end].starts_with_leading_vowel()
        {
            // Rule: two bare consonants between a hard ending and a
            // following leading-vowel nucleus have two-sided ownership
            // ambiguity. The first may extend the closed spelling, the
            // second may start the following leading-vowel spelling, and
            // the pair may also be an implicit syllable. Without a
            // lexical decision, do not invent any of those boundaries.
            // Examples: `มันรสเหมือน` becomes `มัน | ?ร | ?ส | เหมือน`;
            // neither `มันร` nor `สเหมือน` is certain, and `รส` is not
            // promoted merely because two bare bases can form a candidate.
            // A listed onset such as `พร` is different: the cluster rule
            // gives the pair a structural interpretation, so it remains
            // eligible for the normal implicit-candidate resolution.
            self.ownership
                .push_undecided(self.ownership.atom_absolute_range(start));
            self.ownership
                .push_undecided(self.ownership.atom_absolute_range(start + 1));
            return true;
        }
        let unresolved_len = end - start;
        let singleton_cannot_attach_left =
            unresolved_len == 1 && self.singleton_cannot_attach_to_inherent_final_left(start);
        if singleton_cannot_attach_left {
            // Rule: once an inherent-final ใ-/ไ- atom is complete, a
            // singleton that cannot be its coda or its listed onset
            // continuation is not a real two-sided ambiguity. Move it
            // into the following candidate instead of exposing an
            // impossible `Undecided` part.
            // Example: `ในจมก.` becomes `ใน | จมก | .`; `จ` cannot
            // attach to `ใน`, so the following terminal candidate owns it.
            self.ownership.adopt_atom_prefix(end, start);
            return true;
        }
        false
    }

    /// Whether the lone base at `start` cannot close the inherent-final ใ-/ไ- atom before it and
    /// must instead be adopted by the following candidate.
    fn singleton_cannot_attach_to_inherent_final_left(&self, start: usize) -> bool {
        let Some(previous) = start.checked_sub(1) else {
            return false;
        };
        let previous_atom = &self.atoms()[previous];
        let previous_range = self.ownership.atom_range(previous);
        let starts_with_inherent_final = self
            .characters()
            .iter()
            .find(|character| character.start == previous_range.start)
            .is_some_and(|character| {
                ThaiScalar(character.character).is_inherent_final_leading_vowel()
            });
        if !starts_with_inherent_final {
            return false;
        }

        let Some(previous_base) = previous_atom.first_base() else {
            return false;
        };
        let Some(current_base) = self.atoms()[start].first_base() else {
            return false;
        };

        // Rule: a bare ย remains ambiguous after ใ-/ไ- because the inherent-final
        // spelling may own it; this is the one unresolved bare continuation that
        // the hard-ending closure deliberately preserves. A listed initial
        // cluster has the same positive evidence for the following syllable's
        // onset, so its first base remains ambiguous as well. A coda-capable
        // singleton also has a valid left attachment and may start the following
        // candidate, so it remains undecided. Every other singleton has no valid
        // left attachment to the already closed inherent-final atom and must be
        // adopted by the following candidate.
        // Examples: `ในจมก` moves `จ` into `จมก`, while `ไหลย้อน` keeps `ล`
        // undecided because ห+ล is a listed cluster, and `ไกยกะ` keeps `ย`
        // undecided because ย can still belong to the ไก spelling. `ไหนโอ` keeps
        // `น` undecided because it can close `ไห` or begin `โอ`.
        if current_base == '\u{0e22}' // ย
        || ThaiScalar(previous_base).forms_initial_cluster_with(ThaiScalar(current_base))
        || ThaiScalar(current_base).is_coda()
        {
            return false;
        }

        true
    }

    /// Measures the candidates a remaining bare run can offer before a following syllable.
    fn remaining_run_facts(&self, start: usize, end: usize) -> RemainingRunFacts {
        let unresolved = &self.atoms()[start..end];
        let first_length = AtomSequence(unresolved).implicit_syllable_length();
        let leaves_singleton_tail =
            unresolved.len() > first_length && unresolved.len() - first_length == 1;
        let final_two_suffix_start = unresolved.len().checked_sub(2).filter(|&suffix_start| {
            unresolved.len() >= 4
                && AtomSequence(&unresolved[suffix_start..]).implicit_syllable_length() == 2
        });
        // Rule: prefer the longest safe prefix that still leaves a complete
        // implicit candidate, while retaining any unresolved tail.
        // Examples: for a run shaped like `กเรณุกา`, a three-atom `กเรณุ` prefix
        // may be preferred when it leaves `กา` as a complete candidate. The rule
        // never consumes the final atom if doing so would leave a one-atom tail.
        let prefix_length = (2..=unresolved.len()).rev().find(|&length| {
            AtomSequence(&unresolved[..length])
                .implicit_syllable_sequence_start()
                .is_some()
        });
        // Rule: do not synthesize an implicit candidate when its final
        // bare atom is also the first base of a following initial cluster.
        // The preceding atoms may form the left candidate, while that
        // final atom may begin the right-hand cluster; the cluster alone
        // does not prove either ownership. Examples: `ไหนครับ` becomes
        // `ไห | ?น | ?ค | รับ`, and `ไหมครับ` becomes
        // `ไห | ?ม | ?ค | รับ`. Only a later pass may resolve those atoms.
        let first_candidate_ends_at_following_cluster = AtomSequence(unresolved)
            .following_initial_prefix_start(&self.atoms()[end])
            .is_some_and(|cluster_start| cluster_start + 1 == first_length);
        RemainingRunFacts {
            first_length,
            leaves_singleton_tail,
            final_two_suffix_start,
            prefix_length,
            first_candidate_ends_at_following_cluster,
        }
    }

    /// Prefers the final two-atom candidate when a leading candidate would leave a singleton.
    fn resolve_final_two_candidate(
        &mut self,
        start: usize,
        end: usize,
        facts: &RemainingRunFacts,
    ) -> bool {
        let (true, Some(suffix_start)) =
            (facts.leaves_singleton_tail, facts.final_two_suffix_start)
        else {
            return false;
        };
        // Rule: if a leading implicit interpretation would leave a
        // singleton tail, prefer the final two-atom candidate and
        // keep every earlier attachment choice explicit.
        // Example: `เพื่อตรวจสอบ` becomes `เพื่อ | ต? | ร? |
        // วจ | สอบ`; the parser does not choose whether either
        // leading consonant belongs to the adjacent spelling.
        for offset in 0..suffix_start {
            self.ownership
                .push_undecided(self.ownership.atom_absolute_range(start + offset));
        }
        self.ownership.push_syllable(
            self.ownership
                .atom_absolute_range(start + suffix_start)
                .start..self.ownership.atom_absolute_range(end - 1).end,
        );
        true
    }
}

/// What a remaining bare run can offer as candidates before a following syllable.
struct RemainingRunFacts {
    first_length: usize,
    leaves_singleton_tail: bool,
    final_two_suffix_start: Option<usize>,
    prefix_length: Option<usize>,
    first_candidate_ends_at_following_cluster: bool,
}
