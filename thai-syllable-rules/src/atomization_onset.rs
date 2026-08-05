use crate::atom::ThaiAtom;
use crate::character::ThaiScalar;
use crate::character_sequence::CharacterSequence;
use crate::construction_scan::ConstructionScan;

impl CharacterSequence<'_> {
    /// Scans one construction's onset (leading vowel, base, cluster, or independent vowel)
    /// and the nucleus marks that belong to it.
    pub(crate) fn scan_onset_and_nucleus(
        self,
        scan: &mut ConstructionScan,
        previous: Option<&ThaiAtom>,
    ) {
        let character = scan.character;
        if ThaiScalar(character).is_leading_vowel() {
            self.scan_leading_vowel_onset(scan);
        } else if ThaiScalar(character).is_base() {
            self.scan_base_onset(scan, previous);
        } else if ThaiScalar(character).is_independent_vowel() {
            self.scan_independent_vowel_onset(scan);
        } else if ThaiScalar(character).is_dependent() {
            // Rule: a dependent mark without a local base is consumed into the
            // current atom so it can never become a standalone public part.
            // Examples: malformed input beginning with `ิ` or `์` remains
            // source material inside the current run; it never produces a
            // public `Syllable("ิ")` or `Undecided("์")` part. A valid `กิ`
            // still records the mark as the nucleus of `ก`.
            scan.has_vowel_rime =
                self.consume_dependent_characters(&mut scan.index, &mut scan.has_nucleus, false);
        } else {
            scan.index += 1;
        }
    }

    /// Scans a construction that starts with one consonant and consumes its dependent marks and
    /// orthographic rime.
    fn scan_base_onset(self, scan: &mut ConstructionScan, previous: Option<&ThaiAtom>) {
        let characters = self.0;
        let character = scan.character;
        // Rule: a base-led atom starts with one consonant and consumes its
        // immediately following dependent marks and orthographic rime.
        // Examples: `กะ` starts at `ก` and consumes `ะ`; `เป็น` consumes
        // the `เ...็` rime and its final `น`; `สวัสดี` is still read as
        // adjacent local atoms rather than one unbounded consonant run.
        scan.first_base = Some(character);
        let bare_predecessor_blocks_carrier = previous.is_some_and(|atom| {
            if atom.has_nucleus() || atom.has_tone_mark || atom.has_silent_mark() {
                return false;
            }

            let carrier_base_has_explicit_rime = characters
                .get(scan.index + 2)
                .is_some_and(|value| ThaiScalar(value.character).is_dependent());
            let preceding_base_can_be_a_coda = atom
                .first_base()
                .is_some_and(|character| ThaiScalar(character).is_coda());
            let following_syllable_after_carrier = self.starts_following_syllable(scan.index + 2);

            // Rule: a coda-capable bare atom before a bare carrier must not
            // be collapsed into the carrier when a later candidate anchors
            // both ownership readings. The preceding base may close the
            // previous syllable, while `อ` and its base may form the next
            // implicit syllable; collapsing them would erase all three
            // attachment choices before resolution can expose them.
            // Examples: `ว่ากอนด้า` must preserve `ก`, `อ`, and `น` as
            // separate undecided atoms between `ว่า` and `ด้า`. A carrier
            // with its own dependent rime, such as `อับ`, is also kept
            // separate from a bare predecessor so its marks cannot be
            // misclassified as an implicit carrier. A tone-marked
            // predecessor is excluded: `ป้อง` has positive local spelling
            // evidence and must remain one carrier construction.
            carrier_base_has_explicit_rime
                || (preceding_base_can_be_a_coda && following_syllable_after_carrier)
        });
        let is_bare_carrier_syllable = ThaiScalar(character).is_vowel_carrier()
            && characters
                .get(scan.index + 1)
                .is_some_and(|value| ThaiScalar(value.character).is_coda())
            && !bare_predecessor_blocks_carrier
            && (characters
                .get(scan.index..scan.index + 2)
                .is_some_and(|values| {
                    ThaiScalar(values[0].character)
                        .forms_initial_cluster_with(ThaiScalar(values[1].character))
                })
                || !characters
                    .get(scan.index + 2)
                    .is_some_and(|value| ThaiScalar(value.character).is_dependent()));
        if is_bare_carrier_syllable {
            // Rule: a bare อ prefix plus its following base is one
            // implicit-vowel syllable, as in อน. The prefix supplies the
            // spelling's inherent a-like sound; it is not a second
            // explicit vowel nucleus that should force an internal split.
            // Examples: `อน` stays one candidate, while `ขออน` can become
            // `ขอ | อน`. If the following base has its own rime, as in
            // `อนา`, the later resolver can expose `อ | นา` instead. When
            // a bare unresolved atom precedes a carrier whose base has an
            // explicit rime, the carrier is intentionally left atomized
            // as `อ` plus that following base. This preserves both
            // choices in `วิฟอยู่`: `วิ | ?ฟ | ?อ | ยู่`.
            scan.index += 2;
            scan.has_nucleus = true;
            (
                scan.has_vowel_rime,
                scan.has_ue_rime,
                scan.consumed_short_a_coda,
            ) = self.consume_rime(&mut scan.index, &mut scan.has_nucleus, true);
        } else {
            self.scan_consonant_rime(scan, previous);
        }

        self.scan_extended_rime(scan);
    }

    /// Scans a consonant-led construction that is not a bare carrier syllable: its onset
    /// cluster, dependent marks, and the carrier forms that complete its rime.
    fn scan_consonant_rime(self, scan: &mut ConstructionScan, previous: Option<&ThaiAtom>) {
        let characters = self.0;
        let start = scan.start;
        let character = scan.character;
        scan.index += 1;
        (
            scan.has_vowel_rime,
            scan.has_ue_rime,
            scan.consumed_short_a_coda,
        ) = self.consume_rime(&mut scan.index, &mut scan.has_nucleus, true);

        if !scan.has_vowel_rime
            && characters
                .get(scan.index)
                .is_some_and(|value| ThaiScalar(value.character).is_independent_vowel())
            && characters.get(scan.index).is_some_and(|value| {
                ThaiScalar(character).forms_initial_cluster_with(ThaiScalar(value.character))
            })
        {
            // Rule: a permitted base + ฤ/ฦ construction supplies a
            // nucleus even though the independent vowel follows the
            // base rather than preceding it.
            // Examples: `กฤ` and `กฦ` have a nucleus after the base;
            // the parser therefore does not leave `ฤ` or `ฦ` as a
            // detached dependent mark. The following `า` in a larger
            // form is handled by the independent-vowel continuation.
            scan.has_independent_vowel = true;
            scan.has_nucleus = true;
            scan.index += 1;
            self.consume_dependent_characters(&mut scan.index, &mut scan.has_nucleus, false);
        }

        let carrier_belongs_to_following_syllable = (previous.is_some_and(ThaiAtom::is_opaque)
            && self.starts_following_open_carrier_syllable(start))
            || characters
                .get(scan.index..scan.index + 2)
                .is_some_and(|values| {
                    ThaiScalar(values[0].character).is_vowel_carrier()
                        && (ThaiScalar(values[1].character).is_dependent()
                            || (ThaiScalar(values[1].character).is_base()
                                && ThaiScalar(character).is_coda()
                                && previous.is_some_and(|atom| {
                                    atom.has_nucleus() && !atom.has_hard_ending()
                                })))
                });
        if (!scan.has_vowel_rime || scan.has_ue_rime)
            && !carrier_belongs_to_following_syllable
            && characters
                .get(scan.index..scan.index + 1)
                .is_some_and(|values| ThaiScalar(values[0].character).is_vowel_carrier())
        {
            // Rule: attach อ plus its dependent mark as a carrier
            // when the current base has no competing rime. A following
            // bare base is not enough to reject that attachment: a
            // tone-marked base with no vowel still requires the carrier
            // to complete its own spelling, even when the carrier and
            // base also resemble a listed onset such as อย.
            // Examples: `ขอ` uses `อ` as the carrier after `ข`; in a
            // sequence such as `กะกเรณุ`, the `อ`-carrier lookahead
            // cannot steal the following structured construction.
            // `อร่อยที่` therefore keeps the second อ with the
            // tone-marked ร่ and leaves ย undecided: `อร่อ | ?ย | ที่`.
            // `ป้อง` uses the same carrier completion, while an
            // explicit dependent mark or a coda that can begin the
            // next candidate still supplies positive evidence for
            // leaving the carrier to the right.
            // A competing explicit rime, such as the one already in
            // `กีอา`, also prevents an unrelated carrier merge. An
            // onset-only base from the complete non-coda inventory
            // must keep its own carrier even after an open preceding
            // syllable: `ว่าฮอนด้า` therefore keeps `ฮอ` and leaves
            // `น` available to either side of the boundary. The
            // coda predicate, not a spelling-specific branch, defines
            // that inventory. An opaque garan candidate is another
            // certainty boundary: in `หงษ์ทอง`, `ท` can belong to
            // either side, so it stays bare and the following `อง`
            // carrier starts at `อ`.
            scan.index += 1;
            scan.consumed_ue_carrier = scan.has_ue_rime;
            scan.has_carrier = true;
            scan.has_nucleus = true;
            self.consume_dependent_characters(&mut scan.index, &mut scan.has_nucleus, true);
        }
    }

    /// Consumes the base that follows an ฦ construction when it can only be that construction's
    /// coda.
    fn scan_extended_rime(self, scan: &mut ConstructionScan) {
        let characters = self.0;
        let start = scan.start;
        if !scan.has_vowel_rime
            && characters
                .get(scan.index..scan.index + 2)
                .is_some_and(|values| {
                    values[0].character == '\u{0e26}' // ฦ
                        && values[1].character != '\u{0e45}' // ๅ
                })
        {
            // Rule: the independent-vowel pair ฦๅ belongs to this base-led
            // atom when it follows a base without another rime.
            // Example: `กฦๅ` keeps the base and the independent-vowel
            // extension together; it is not split into `ก | ฦ | ๅ`.
            scan.has_nucleus = true;
            scan.index += 1;
        }

        if !scan.has_vowel_rime
            && characters.get(start..scan.index).is_some_and(|values| {
                values
                    .iter()
                    .any(|value| ThaiScalar(value.character).is_tone_mark())
            })
            && characters
                .get(scan.index..scan.index + 1)
                .is_some_and(|values| values[0].character == '\u{0e27}' /* ว */)
        // ว
        {
            // Rule: tone-marked base + ว + base is a closed orthographic
            // glide construction unless a following preposed-vowel
            // syllable competes for the boundary. In that competing shape
            // the final base remains a separate atom so resolution can
            // expose its uncertain ownership. Examples: `ช่วย` and
            // terminal `อ้วน` retain their final ย/น, while `ม้วนได้`
            // becomes `ม้ว | ?น | ได้`.
            scan.has_nucleus = true;
            scan.has_glide = true;
            scan.index += 1;
            if characters.get(scan.index).is_some_and(|value| {
                ThaiScalar(value.character).is_base() && ThaiScalar(value.character).is_coda()
            }) && !self.starts_following_leading_vowel_syllable(scan.index)
            {
                scan.index += 1;
            }
        }
    }

    /// Scans a construction that starts with an independent vowel (ฤ or ฦ).
    fn scan_independent_vowel_onset(self, scan: &mut ConstructionScan) {
        let characters = self.0;
        let character = scan.character;
        // Rule: ฤ and ฦ are independent vowel nuclei and may stand without
        // a preceding consonant base.
        // Examples: `ฤดู` begins with an independent nucleus and `ฦ` is
        // still a candidate at the start of a run. Neither character is
        // classified as an ordinary consonant onset.
        scan.has_independent_vowel = true;
        scan.has_nucleus = true;
        scan.index += 1;
        if characters
            .get(scan.index..scan.index + 1)
            .is_some_and(|values| values[0].character == '\u{0e45}' /* ๅ */)
        // ๅ
        {
            scan.index += 1;
        } else if character == '\u{0e24}' {
            // Rule: ฤ followed by bases and a garan mark is one silent-mark
            // construction rather than unrelated bare consonants.
            // Example: `ฤทธิ์` keeps the bases and garan mark associated
            // with the independent `ฤ`; it is not atomized as `ฤ | ท | ธ`
            // with an orphan `์` left for resolution.
            let mut probe = scan.index;
            while characters
                .get(probe..probe + 1)
                .is_some_and(|values| ThaiScalar(values[0].character).is_base())
            {
                probe += 1;
            }
            let mut has_silent_mark = false;
            while characters
                .get(probe..probe + 1)
                .is_some_and(|values| ThaiScalar(values[0].character).is_dependent())
            {
                has_silent_mark |= ThaiScalar(characters[probe].character).is_garan_mark();
                probe += 1;
            }
            if has_silent_mark {
                scan.index = probe;
            }
        }
    }
}
