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

#[derive(Clone, Copy, Debug)]
enum SourceChunk<'a> {
    // Rule: accepted Thai scalars are parsed as one maximal Thai run.
    // Examples: `พระคัมภีร์` is sent to Thai atomization as one run, while a
    // space in `พระ คัมภีร์` ends the first run before the second begins. The
    // run boundary is based on the explicit Thai alphabet, not on Unicode's
    // broad Thai block alone.
    Thai(&'a str),
    // Rule: periods are scanned separately so only one eligible period can
    // become Part::Dot; repeated or misplaced periods remain Part::Text.
    // Examples: `นะ.` can become `Syllable("นะ") | Dot(".")`, but `นะ..`
    // keeps the punctuation text-visible. A leading `.นะ` is also text until
    // a valid Thai part immediately precedes a single period.
    Periods(&'a str),
    // Rule: every scalar outside the accepted Thai alphabet and period is
    // preserved as source text and forms a hard boundary around Thai runs.
    // Examples: `ไทย English` preserves the ASCII word and space, and `ก๑฿ข`
    // prevents `ก` from attaching to `ข`. Punctuation and digits remain in
    // their original order instead of being normalized away.
    Text(&'a str),
}

impl SourceChunk<'_> {
    fn text(&self) -> &str {
        match self {
            Self::Thai(text) | Self::Periods(text) | Self::Text(text) => text,
        }
    }
}
