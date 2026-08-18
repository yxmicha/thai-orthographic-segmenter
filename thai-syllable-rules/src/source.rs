use crate::part::{Part, push_text};

impl Part {
    fn can_precede_dot(&self) -> bool {
        // Rule: a period has Thai punctuation status only immediately after a
        // Syllable or Undecided part; the source scanner has already enforced
        // adjacency.
        // Examples: `นะ.` qualifies after `Syllable("นะ")`, and `นะก.` qualifies
        // after `Undecided("ก")`. A period after `Text`, after another period, or
        // across a space is not eligible.
        matches!(self, Part::Syllable(_) | Part::Undecided(_))
    }
}
