use crate::character::ThaiScalar;
use crate::part::Part;
use crate::resolution::ResolutionState;

impl ResolutionState<'_> {

    /// Exposes a bare อ-carrier with its final base as separate undecided parts when an opaque
    /// garan candidate or an open leading vowel keeps their ownership ambiguous; `None` when the
    /// carrier pair stays one complete candidate.
    fn resolve_ambiguous_bare_carrier(&mut self, index: usize) -> Option<usize> {
        let implicit_carrier_text = &self.text()[self.ownership.atom_range(index)];
        let has_left_syllable_before_unresolved =
            self.ownership.parts.last().is_some_and(Part::is_undecided)
                && self
                    .ownership
                    .parts
                    .iter()
                    .rev()
                    .find(|part| !part.is_undecided())
                    .is_some_and(Part::is_syllable);
        let has_open_leading_vowel_before_carrier = index
            .checked_sub(1)
            .and_then(|previous| self.atoms().get(previous))
            .is_some_and(|previous| {
                previous.starts_with_leading_vowel() && !previous.has_hard_ending()
            });
        let carrier_ends_with_unlicensed_final_r = implicit_carrier_text
            .chars()
            .nth(1)
            .is_some_and(|character| ThaiScalar(character).is_final_r());
        let is_ambiguous_bare_carrier = self.atoms()[index]
            .first_base()
            .is_some_and(|character| ThaiScalar(character).is_vowel_carrier())
            && !self.atoms()[index].has_explicit_rime()
            && !self.atoms()[index].has_silent_mark()
            && implicit_carrier_text.chars().count() == 2
            && implicit_carrier_text
                .chars()
                .all(|character| ThaiScalar(character).is_base())
            && implicit_carrier_text
                .chars()
                .next()
                .is_some_and(|character| ThaiScalar(character).is_vowel_carrier())
            && ((has_left_syllable_before_unresolved
                && self.atoms()[index + 1..]
                    .iter()
                    .any(|atom| atom.has_nucleus() && atom.has_silent_mark()))
                || (has_open_leading_vowel_before_carrier
                    && carrier_ends_with_unlicensed_final_r
                    && self.atoms()[index + 1..]
                        .iter()
                        .any(super::atom::ThaiAtom::has_nucleus)));
        if is_ambiguous_bare_carrier {
            // Rule: a bare อ-carrier plus one final base is only exposed as
            // independent attachment choices when a later opaque garan candidate
            // keeps the right-hand ownership genuinely ambiguous, or when an
            // open leading-vowel atom immediately precedes it. Before an
            // ordinary following syllable, the carrier pair is otherwise a
            // complete implicit candidate. Examples: `ติของฮอร์` becomes
            // `ติ | ข? | อ? | ง? | ฮอร์`, while `มาลองใช้` becomes
            // `มา | ล? | อง | ใช้`. In `เยอรมัน`, the open leading-vowel
            // atom leaves both carrier bases unresolved: `เย | ?อ | ?ร | มัน`.
            // Standalone `อน`, `ขออน`, and explicit-rime `อนุ` keep their
            // complete candidates as well.
            let atom_range = self.ownership.atom_range(index);
            let character_ranges: Vec<_> = self
                .characters()
                .iter()
                .filter(|character| {
                    character.start >= atom_range.start && character.end <= atom_range.end
                })
                .map(|character| {
                    (self.source_offset() + character.start)..(self.source_offset() + character.end)
                })
                .collect();
            for range in character_ranges {
                self.ownership.push_undecided(range);
            }
            return Some(index + 1);
        }
        None
    }
}
