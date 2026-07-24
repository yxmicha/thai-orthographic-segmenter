#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum OnsetShape {
    None,
    Base(char),
    LeadingVowel { base: Option<char> },
    IndependentVowel(char),
    DependentMark,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum NucleusShape {
    None,
    /// Dependent vowel material without a consonant base.
    ///
    /// This is deliberately not a nucleus: it is malformed or incomplete
    /// source material that resolution must keep separate from a preceding
    /// syllable instead of treating it as a second valid vowel pattern.
    OrphanRime,
    Implicit,
    Rime {
        explicit: bool,
        carrier: bool,
        completed_ue_carrier: bool,
        glide: bool,
    },
    IndependentVowel,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum EndingShape {
    Open,
    Hard,
}
