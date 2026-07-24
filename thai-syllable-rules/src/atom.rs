use crate::part::ByteRange;

// Atomization recognizes the local orthographic constructions.
// It deliberately records candidates and metadata instead of deciding the
// final public boundary; `resolution` makes that attachment decision visible.
// Example: atomization can identify `กะ`, bare `ต`, bare `ก`, and `ลม` in
// `กะตกลม`, while resolution decides which bare atoms become `Syllable` and
// which remain `Undecided`. The atom layer therefore does not silently choose
// between `ตก | ลม` and another structurally possible attachment.
/// A local Thai orthographic candidate.
///
/// The atom owns semantic shape classifications rather than exposing a bag of
/// independently mutable flags. Resolution can ask whether a candidate has a
/// hard ending or an explicit rime, but it cannot construct an impossible
/// combination by setting unrelated fields one at a time.
#[derive(Clone, Debug)]
pub(crate) struct ThaiAtom {
    pub(crate) range: ByteRange,
    pub(crate) onset: OnsetShape,
    pub(crate) nucleus: NucleusShape,
    pub(crate) has_tone_mark: bool,
    pub(crate) has_phinthu: bool,
    pub(crate) deferred_silent_coda: bool,
    pub(crate) ending: EndingShape,
    pub(crate) silent_mark: SilentMarkShape,
}

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

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum SilentMarkShape {
    None,
    Attached,
    Opaque {
        has_carrier: bool,
        has_initial_cluster: bool,
    },
}
