use crate::character::ThaiScalar;
use crate::character_sequence::CharacterSequence;

impl CharacterSequence<'_> {
    pub(crate) fn starts_following_initial_cluster(self, index: usize) -> bool {
        let characters = self.0;
        // Rule: a listed onset pair is evidence that the next base run begins a
        // following syllable rather than extending the current final.
        // Examples: after a closed candidate, `กรา` is recognized as a following
        // onset construction, while an unlisted `กม` remains an attachment
        // question for implicit resolution.
        characters.get(index..index + 2).is_some_and(|values| {
            ThaiScalar(values[0].character)
                .forms_initial_cluster_with(ThaiScalar(values[1].character))
        })
    }

    pub(crate) fn starts_initial_cluster_with_rime(self, index: usize) -> bool {
        let characters = self.0;
        // Rule: if the second base of a listed onset cluster already has its own
        // dependent rime, the first base may still be the preceding final. This
        // preserves `อิส | รา` in `อิสรา`; the cluster-shaped lookahead does not
        // force ส into the following syllable when ร visibly starts its rime.
        self.starts_following_initial_cluster(index)
            && characters
                .get(index + 2)
                .is_some_and(|value| ThaiScalar(value.character).is_dependent())
    }

    pub(crate) fn starts_following_syllable(self, index: usize) -> bool {
        let characters = self.0;
        // Rule: a base followed by a dependent mark, initial cluster, or carrier
        // construction is a structural following-syllable candidate. An open
        // base-plus-อ spelling is sufficient even without a final base, and a
        // bare อ+base candidate must be recognized as well. Otherwise an ordinary
        // coda such as จ in `กิจอสัง` would be consumed before the following
        // `อ`-carrier candidate is considered. A base immediately before a
        // leading-vowel construction is intentionally not included here: that
        // shape is handled only by the final-r rule that has a separate positive
        // ownership check. Otherwise a preceding ordinary coda such as ญ in
        // `เหรียญเจือะ` would be left unresolved merely because the next syllable
        // happens to use a preposed vowel.
        // Examples: `กะกรา` sees `กร` as a following candidate; `กะกิ` sees the
        // marked `กิ`; `กะกอน` sees the `อ+น` carrier shape; `กิจอสัง` sees the
        // bare `อ+ส` candidate; and `เหมือนตอ` sees `ตอ` as an open carrier
        // syllable. A lone base with no such evidence remains eligible for
        // conservative ambiguity.
        characters
            .get(index)
            .is_some_and(|value| ThaiScalar(value.character).is_base())
            && (characters
                .get(index + 1)
                .is_some_and(|value| ThaiScalar(value.character).is_dependent())
                || self.starts_following_initial_cluster(index)
                || self.starts_following_carrier_syllable(index)
                || self.starts_carrier_syllable_at(index)
                || self.starts_following_open_carrier_syllable(index))
    }

    pub(crate) fn starts_following_open_carrier_syllable(self, index: usize) -> bool {
        let characters = self.0;
        // Rule: a base followed directly by a bare อ is already a vowel-bearing
        // following candidate, even when no final base follows the carrier. If อ
        // is immediately followed by a dependent mark, it is instead the onset's
        // own explicit rime (`อับ`), not an open carrier; short-ิ ownership keeps
        // the preceding final unresolved in that shape.
        // Examples: `ตอ` and `กอ` are complete open carrier syllables; after
        // `เหมือน`, the `ตอ` in `เหมือนตอ` therefore prevents `น` from being
        // silently committed as the preceding final. In `กลิ่นอับ`, the `อั`
        // rime is handled by `starts_carrier_rime_syllable_at`, so `น` remains
        // unresolved. A later final, as in `ตอก`, is covered by the same
        // bare-carrier prefix and remains part of that candidate.
        characters
            .get(index)
            .is_some_and(|value| ThaiScalar(value.character).is_base())
            && characters
                .get(index + 1)
                .is_some_and(|value| ThaiScalar(value.character).is_vowel_carrier())
            && !characters
                .get(index + 2)
                .is_some_and(|value| ThaiScalar(value.character).is_dependent())
    }

    pub(crate) fn starts_following_carrier_syllable(self, index: usize) -> bool {
        let characters = self.0;
        // Rule: base + อ + base is a following carrier syllable shape.
        // Examples: `กอน` has the `ก + อ + น` carrier shape after a preceding
        // candidate, while `กะ` has no such three-base lookahead. The test is
        // structural and does not claim that every possible spelling is lexical.
        self.starts_following_open_carrier_syllable(index)
            && characters
                .get(index + 2)
                .is_some_and(|value| ThaiScalar(value.character).is_base())
    }

    pub(crate) fn starts_carrier_syllable_at(self, index: usize) -> bool {
        let characters = self.0;
        // Rule: a bare อ plus a following base is one implicit carrier spelling;
        // keep that prefix out of the preceding vowel's final-coda slot.
        // Examples: `อน` is one carrier candidate, and `ขออน` can expose `ขอ | อน`.
        // The guard prevents the preceding vowel from consuming `อ` as if it were
        // an ordinary final consonant.
        characters
            .get(index)
            .is_some_and(|value| ThaiScalar(value.character).is_vowel_carrier())
            && characters
                .get(index + 1)
                .is_some_and(|value| ThaiScalar(value.character).is_base())
    }

    pub(crate) fn starts_following_implicit_syllable(self, index: usize) -> bool {
        let characters = self.0;
        // Rule: two adjacent bases provide an implicit-vowel candidate only when
        // the second base is itself bare. If a dependent mark follows that second
        // base, it already begins an explicit rime and the first base cannot be
        // consumed as part of an implicit pair. Examples: `จม` is a two-base
        // implicit candidate, while the `ภา` in `สุขภาพ` is an explicit following
        // rime and keeps `ข` available to ownership resolution instead of proving
        // it belongs to `สุข`. At the end of a run there is no right-hand anchor
        // for a new candidate, so `พุทธ` consumes `ท` as the final of `พุท` and
        // leaves the terminal `ธ` to normal resolution. The listed-cluster rule
        // still requires a third bare base before it treats a cluster as a
        // three-base implicit candidate.
        let Some(first) = characters.get(index).map(|value| value.character) else {
            return false;
        };
        let Some(second) = characters.get(index + 1).map(|value| value.character) else {
            return false;
        };
        if characters.get(index + 2).is_none()
            || !ThaiScalar(first).is_base()
            || !ThaiScalar(second).is_base()
            || !ThaiScalar(second).is_coda()
        {
            return false;
        }

        if characters
            .get(index + 2)
            .is_some_and(|value| ThaiScalar(value.character).is_dependent())
        {
            return false;
        }

        !ThaiScalar(first).forms_initial_cluster_with(ThaiScalar(second))
            || characters
                .get(index + 2)
                .is_some_and(|value| ThaiScalar(value.character).is_base())
    }

    pub(crate) fn starts_syllable_after_final(self, index: usize) -> bool {
        let characters = self.0;
        // Rule: after a consumed final, a dependent-mark base or listed cluster
        // marks the next syllable and prevents final-base absorption. Check the
        // current position as well as the position after it: when the current
        // base is the first member of a cluster, it is the next onset, not the
        // previous syllable's coda. Examples: after `คืน`, a marked next base
        // starts its own candidate; `เพื่อ` must leave `ตร` available in
        // `เพื่อตรวจ`; and `เป็นอยู่` must not become one overlong spelling.
        if self.starts_following_initial_cluster(index) {
            return true;
        }

        let next = index + 1;
        if characters
            .get(next)
            .is_some_and(|value| ThaiScalar(value.character).is_leading_vowel())
            && characters
                .get(next + 1)
                .is_some_and(|value| ThaiScalar(value.character).is_base())
        {
            // Rule: after a completed ือ carrier, a valid leading-vowel spelling
            // is enough to keep the ordinary final unresolved. The final may close
            // the carrier or begin the following orthographic run, and the parser
            // must preserve that choice instead of silently consuming it.
            // Examples: `เหมือนแส` becomes `เหมือ | ?น | แส`, while the same
            // `น` before a following base-led carrier remains governed by the
            // ordinary carrier lookahead below, as in `เหมือนน้ำ`.
            return true;
        }
        self.starts_following_syllable(next)
    }
}
