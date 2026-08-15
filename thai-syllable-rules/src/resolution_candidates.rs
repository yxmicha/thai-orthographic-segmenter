use crate::atom_sequence::AtomSequence;
use crate::character::ThaiScalar;
use crate::resolution::ResolutionState;

impl ResolutionState<'_> {
    pub(crate) fn expose_repeated_carrier_onset(&mut self, start: usize, end: usize) -> bool {
        // Rule: when an unresolved coda-capable base is immediately followed by a
        // carrier atom beginning with the same base, the repeated base may instead
        // continue the preceding implicit-vowel construction. This rule applies
        // only after an implicit candidate; an explicit carrier such as `ตอ`
        // already supplies its vowel, so the following `นอ` keeps its onset.
        // Keep both bases undecided and strip only the carrier's onset from the
        // following atom when the implicit context proves that choice remains
        // ambiguous.
        // Examples: `จมรรอก` becomes `จม | ?ร | ?ร | อก`; the two ร bases may
        // form the implicit-a material after จม, or the second one may begin รอก.
        // `ตอนนอนสิ` becomes `ตอ | ?น | นอ | ?น | สิ`; the explicit carrier in
        // ตอ does not make the onset of นอ ambiguous. `กุรอก` uses ordinary
        // singleton attachment instead: ร remains a possible coda of กุ.
        let Some(previous) = start
            .checked_sub(1)
            .and_then(|index| self.atoms().get(index))
        else {
            return false;
        };
        if previous.has_explicit_rime()
            || previous.has_carrier()
            || previous.starts_with_leading_vowel()
            || previous.has_tone_mark
            || previous.has_hard_ending()
        {
            return false;
        }
        if end - start != 1
            || !self.can_attach_singleton_as_coda(start)
            || !self.atoms()[end].has_carrier()
            || self.atoms()[start].first_base() != self.atoms()[end].first_base()
        {
            return false;
        }

        let repeated_base_start = self.ownership.atom_starts[end];
        let Some(repeated_base) = self.atoms()[end].first_base() else {
            return false;
        };
        for atom_index in start..end {
            self.ownership
                .push_undecided(self.ownership.atom_absolute_range(atom_index));
        }
        let repeated_base_end = repeated_base_start + repeated_base.len_utf8();
        {
            let (index, end) = (end, repeated_base_end);
            let plan = &mut self.ownership;
            let start = plan.atom_starts[index];
            debug_assert!(end > start && end < plan.atom_ranges[index].end);
            plan.push_undecided((plan.source_offset + start)..(plan.source_offset + end));
            plan.atom_starts[index] = end;
        };
        true
    }

    pub(crate) fn move_onset_only_cluster_suffix_forward(
        &mut self,
        start: usize,
        end: usize,
    ) -> bool {
        // Rule: if the tail of a longer unresolved run is an onset-only base and
        // forms a listed initial cluster with the following atom, move only that
        // suffix forward. Earlier bases remain independently attachable instead
        // of being swallowed into the same following syllable. A single earlier
        // coda-capable base is different: once the following atom already owns
        // the listed cluster onset, that base cannot be a second onset there, so
        // it belongs to the preceding syllable when that syllable can accept it.
        // Examples: `เดือนหน้า` is `เดือ | ?น | หน้า`, not `เดือน | หน้า`,
        // because the parser cannot prove that น belongs to เดือ rather than
        // being boundary material before the following หน onset. The same
        // certainty changes `อีกหลาย` to `อีก | หลาย`; it does not resolve a longer ambiguous tail such as
        // `กะตกลม`, where more than one possible attachment remains. `เพื่อตรวจ`
        // does not use this rule because ร + ว is not a listed pair, while
        // `กะกเรณุ` retains its existing singleton-cluster rule for ก + ร.
        let relative_cluster_start = AtomSequence(&self.atoms()[start..end])
            .following_initial_prefix_start(&self.atoms()[end]);
        let Some(relative_cluster_start) = relative_cluster_start else {
            return false;
        };
        let cluster_start = start + relative_cluster_start;
        let can_attach_coda_before_cluster = cluster_start == start + 1
            && 'can_attach_coda_before_onset_cluster: {
                let (start, cluster_start) = (start, cluster_start);
                // Rule: this certainty path applies only to one unresolved coda before
                // a following listed onset cluster. The preceding public part must be a
                // real open syllable; a hard ending or a missing nucleus cannot accept
                // another base. A plain carrier and a completed ือ carrier are both
                // deliberately excluded because their final remains ambiguous before
                // a following candidate. The earlier tail must also not form a valid
                // implicit candidate with the cluster-start base; otherwise there are
                // two legal ownership readings and the tail must remain `Undecided`.
                // The cluster-start argument is kept explicit so this guard cannot
                // accidentally consume a longer unresolved run.
                // Examples: `เดือนหน้า` keeps น undecided between เดือ and หน้า, while
                // `อีกหลาย` accepts ก into อีก before หล. `กะนหน้า` keeps น undecided because the
                // short ะ already hard-closes กะ, and `กะตกลม` keeps its multi-base tail
                // ambiguous because this helper only handles one base immediately before
                // the onset-only suffix. `ซึ้งคลื่น` remains `ซึ้ | ?ง | ?ค | ลื่น`
                // because `งค` is itself a valid implicit pair. `วิฟอยู่` remains
                // `วิ | ?ฟ | ?อ | ยู่` because the plain อ carrier is also ambiguous.
                // `บล็อกหยุด` and `เดือนหน้า` follow the same conservative rule: neither
                // commits an optional final before a following cluster.
                if cluster_start != start + 1
                    || !self.ownership.last_is_syllable()
                    || self.atoms()[cluster_start]
                        .first_base()
                        .is_some_and(|character| ThaiScalar(character).is_vowel_carrier())
                    || AtomSequence(&self.atoms()[start..=cluster_start]).implicit_syllable_length()
                        >= 2
                {
                    break 'can_attach_coda_before_onset_cluster false;
                }
                let Some(previous) = start
                    .checked_sub(1)
                    .and_then(|index| self.atoms().get(index))
                else {
                    break 'can_attach_coda_before_onset_cluster false;
                };
                previous.has_nucleus()
                    && !previous.has_hard_ending()
                    && !previous.has_carrier()
                    && self.can_attach_singleton_as_coda(start)
            };
        if can_attach_coda_before_cluster {
            self.ownership
                .extend_last_syllable(self.ownership.atom_absolute_range(start).end);
            self.ownership.adopt_atom_prefix(end, cluster_start);
            return true;
        }
        if cluster_start <= start
            || self.can_attach_singleton_as_coda(cluster_start)
            || self.atoms()[cluster_start]
                .first_base()
                .is_some_and(|character| ThaiScalar(character).is_vowel_carrier())
        {
            // Rule: a carrier at the front of a listed cluster cannot be moved
            // forward merely because its second base is a recognized onset.
            // When an earlier bare atom is still unresolved, the carrier can
            // itself be the next attachment choice, so moving it would hide a
            // real boundary without certainty. Examples: `วิฟอยู่` remains
            // `วิ | ?ฟ | ?อ | ยู่`; the `อ` is not silently folded into `อยู`.
            // This does not weaken ordinary cluster ownership: `อีกหลาย` still
            // moves onset-only `ห` into `หลาย`, and a carrier that was already
            // atomized with its own rime remains a normal following syllable.
            return false;
        }

        for atom_index in start..cluster_start {
            self.ownership
                .push_undecided(self.ownership.atom_absolute_range(atom_index));
        }
        self.ownership.adopt_atom_prefix(end, cluster_start);
        true
    }
}

impl ResolutionState<'_> {
    pub(crate) fn can_attach_singleton_as_coda(&self, start: usize) -> bool {
        // Ordinary ร is a /n/ coda, including after an explicit vowel (การ).
        // Its silent-extension role must not remove this competing attachment.
        self.atoms()
            .get(start)
            .and_then(super::atom::ThaiAtom::first_base)
            .is_some_and(|character| ThaiScalar(character).is_coda())
    }

    pub(crate) fn absorb_leading_bases(&mut self) -> usize {
        // Rule: leading bases have no preceding syllable. Absorb them only when
        // their following shape proves the attachment, using explicit cluster
        // structure or a leading-vowel spelling. Otherwise keep onset-only bases
        // visible. Examples: `มเหสี` absorbs `ม` into `เห`, while `มหา` keeps
        // `ม` separate from the base-led `หา` candidate.
        // The helper never invents a left-hand part that the source does not have.
        let Some(nucleus_index) = self
            .atoms()
            .iter()
            .position(super::atom::ThaiAtom::has_nucleus)
        else {
            return 0;
        };
        let prefix = &self.atoms()[..nucleus_index];
        if prefix.is_empty() || !prefix.iter().all(|atom| atom.first_base().is_some()) {
            return 0;
        }

        self.absorb_leading_pair(nucleus_index)
            .or_else(|| self.absorb_leading_single(nucleus_index))
            .unwrap_or(0)
    }

    /// Resolves a leading run of two or more bases before the nucleus at `nucleus_index`; `None`
    /// when the run does not start with a pair this handles.
    fn absorb_leading_pair(&mut self, nucleus_index: usize) -> Option<usize> {
        let prefix = &self.atoms()[..nucleus_index];
        if prefix.len() >= 2
            && prefix[0]
                .first_base()
                .zip(prefix[1].first_base())
                .is_some_and(|(first, second)| {
                    !ThaiScalar(first).forms_initial_cluster_with(ThaiScalar(second))
                })
        {
            if prefix[1]
                .first_base()
                .is_some_and(|character| ThaiScalar(character).is_coda())
            {
                // Rule: two non-cluster leading bases form one implicit candidate
                // only when the second base can be its coda. Example: `สม` is a
                // valid structural candidate, while `สห` cannot be one because
                // ห is onset-only and must remain available to the next spelling.
                let start = self.source_offset() + prefix[0].relative_range().start;
                let end = self.source_offset() + prefix[1].relative_range().end;
                self.ownership.push_syllable(start..end);
                return Some(2);
            }

            // Rule: a non-cluster leading run whose second base is onset-only
            // cannot be collapsed into an implicit syllable. Keep each base
            // visible so `มหา`, `สหภาพ`, and `คหบดี` do not hide separate spoken
            // units inside a broad first candidate.
            let prefix_len = prefix.len();
            for atom_index in 0..prefix_len {
                self.ownership
                    .push_undecided(self.ownership.atom_absolute_range(atom_index));
            }
            return Some(nucleus_index);
        }

        if prefix.len() >= 2
            && prefix[0]
                .first_base()
                .zip(prefix[1].first_base())
                .is_some_and(|(first, second)| {
                    ThaiScalar(first).forms_initial_cluster_with(ThaiScalar(second))
                })
        {
            // Rule: a recognized initial cluster is retained as one leading
            // candidate before the later nucleus. Examples: a leading `กร` before
            // a later nucleus stays one candidate; an unlisted pair uses the rule
            // above instead.
            let start = self.source_offset() + prefix[0].relative_range().start;
            let end = self.source_offset() + prefix[1].relative_range().end;
            self.ownership.push_syllable(start..end);
            return Some(2);
        }

        None
    }

    /// Resolves a single leading base before the nucleus at `nucleus_index`; `None` when the run
    /// has another length.
    fn absorb_leading_single(&mut self, nucleus_index: usize) -> Option<usize> {
        let prefix = &self.atoms()[..nucleus_index];
        if prefix.len() == 1 && prefix[0].has_tone_mark {
            // Rule: a tone-marked bare consonant is already a complete implicit
            // onset candidate. Do not merge its implicit vowel with the following
            // explicit vowel pattern, as in `ก้` before `นา`.
            self.ownership
                .push_syllable(self.ownership.atom_absolute_range(0));
            return Some(nucleus_index);
        }

        if prefix.len() == 1
            && prefix[0]
                .first_base()
                .is_some_and(|character| ThaiScalar(character).is_vowel_carrier())
            && self.atoms()[nucleus_index].has_explicit_rime()
        {
            // Rule: a standalone vowel carrier already supplies its own implicit
            // vowel. Keep it separate before a following explicit-rime candidate,
            // as in a leading `อ` before `นา`.
            self.ownership
                .push_syllable(self.ownership.atom_absolute_range(0));
            return Some(nucleus_index);
        }

        if prefix.len() == 1 {
            let joins_following_cluster = prefix[0]
                .first_base()
                .zip(self.atoms()[nucleus_index].first_base())
                .is_some_and(|(first, second)| {
                    ThaiScalar(first).forms_initial_cluster_with(ThaiScalar(second))
                });
            let following_base_is_onset_only = self.atoms()[nucleus_index]
                .first_base()
                .is_some_and(|base| !ThaiScalar(base).is_coda());
            if self.atoms()[nucleus_index].starts_with_leading_vowel()
                || joins_following_cluster
                || !following_base_is_onset_only
            {
                // Rule: one leading base may join a following leading-vowel
                // spelling, recognized initial cluster, or coda-capable base
                // because the source order proves that it supplies the
                // following onset. Examples: `มเหสี` keeps `ม` with `เห`,
                // `กล้า` keeps `ก` with `ล้า`, and `ขนาด` keeps `ข` with `นา`.
                self.ownership.adopt_atom_prefix(nucleus_index, 0);
                return Some(nucleus_index);
            }

            // Rule: one leading base before an onset-only nucleus is not known
            // to belong to that nucleus. Keep it unresolved instead of
            // creating a multi-vowel candidate such as `มหา` or `พหุ`.
            self.ownership
                .push_undecided(self.ownership.atom_absolute_range(0));
            return Some(nucleus_index);
        }

        None
    }
}
