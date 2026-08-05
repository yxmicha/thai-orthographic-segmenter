use crate::atom::ThaiAtom;
use crate::character::ThaiScalar;
use crate::character_sequence::CharacterSequence;
use crate::construction_scan::ConstructionScan;

impl CharacterSequence<'_> {

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
}
