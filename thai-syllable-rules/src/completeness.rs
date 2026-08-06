use crate::atom::ThaiAtom;
use crate::character::ThaiScalar;

/// A nucleus alone does not complete a rime that requires an ending.
/// This is independent of optional prefix/coda eligibility in the resolver.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum Completeness {
    Complete,
    RequiresHost,
    RequiresConstituent,
    RequiresCoda,
    RequiresCarrierOrCoda,
}

impl Completeness {
    pub(crate) fn of_atom(atom: &ThaiAtom, source: &str) -> Self {
        let text = &source[atom.relative_range().clone()];
        if text == "บ่" {
            // Attested Thai negator, not a general license for tone-only cells.
            return Self::Complete;
        }
        let mut characters = text.chars();
        if atom.has_silent_mark()
            && !atom.starts_with_leading_vowel()
            && text.ends_with('์')
            && text
                .chars()
                .filter(|character| ThaiScalar(*character).is_base())
                .count()
                == 1
        {
            // A one-base garan cell silences its own local material and has no
            // independent host. It may extend a preceding syllable, but it
            // cannot be advertised as a complete standalone candidate.
            return Self::RequiresHost;
        }
        if text.starts_with('เ')
            && ((text.contains('ี') && !text.contains('ย'))
                || (text.contains('ื') && !text.contains('อ')))
        {
            // Leading เ-ีย requires ย and leading เ-ือ requires อ. A hard
            // ending or another local closure cannot discharge a missing
            // constituent of the compound-vowel frame.
            return Self::RequiresConstituent;
        }
        if characters
            .next()
            .is_some_and(|character| ThaiScalar(character).is_base())
            && characters
                .next()
                .is_some_and(|character| ThaiScalar(character).is_tone_mark())
            && characters.next().is_none()
        {
            return Self::RequiresCoda;
        }
        if atom.has_hard_ending() || atom.has_glide() || atom.has_carrier() {
            return Self::Complete;
        }
        if text.contains('ั') || (text.starts_with('เ') && text.contains('ิ')) || text.contains('็')
        {
            Self::RequiresCoda
        } else if text.contains('ื') && !atom.has_tone_mark {
            // An unmarked ื nucleus still needs its carrier or coda, but a
            // tone-marked form may remain an open candidate while that coda
            // is undecided. This preserves `พื้นที่` as `พื้ | ?น | ที่`
            // instead of misclassifying the legal boundary fragment as
            // invalid before ownership resolution runs.
            Self::RequiresCarrierOrCoda
        } else {
            Self::Complete
        }
    }
}
