use crate::character::ThaiCharacter;
use crate::character::ThaiScalar;
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

/// The complete result of scanning one local orthographic construction.
///
/// Keeping the scan facts together gives atom construction one ownership
/// boundary: atomization discovers the source shape, while this module turns
/// that shape into the semantic atom used by resolution.
#[derive(Clone, Debug)]
#[allow(
    clippy::struct_excessive_bools,
    reason = "independent facts of one scan, each tested on its own"
)]
pub(crate) struct AtomScan {
    pub(crate) range: ByteRange,
    pub(crate) character: char,
    pub(crate) first_base: Option<char>,
    pub(crate) has_nucleus: bool,
    pub(crate) has_vowel_rime: bool,
    pub(crate) has_ue_rime: bool,
    pub(crate) has_independent_vowel: bool,
    pub(crate) has_tone_mark: bool,
    pub(crate) has_hard_ending: bool,
    pub(crate) deferred_silent_coda: bool,
    pub(crate) is_opaque: bool,
    pub(crate) has_leading_vowel: bool,
    pub(crate) has_carrier: bool,
    pub(crate) has_glide: bool,
    pub(crate) consumed_ue_carrier: bool,
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

impl ThaiAtom {
    /// Interprets the source-local facts of one scanned construction as an atom. The scanner
    /// owns only those facts; their meaning as onset, nucleus, ending, and silent-mark shapes
    /// is decided here, so atomization never builds a partially classified flag bag that
    /// resolution would have to repair.
    ///
    /// Examples: `เกิน` gets a leading-vowel onset and explicit rime; `กำ` gets hard-ending
    /// evidence; and `ฟิล์ม` gets an opaque mark shape with its carrier and cluster properties
    /// derived once here. `characters` are the scalars the scan consumed.
    pub(crate) fn from_scan(scan: AtomScan, characters: &[ThaiCharacter]) -> Self {
        let has_structural_nucleus =
            scan.has_nucleus && (scan.first_base.is_some() || scan.has_independent_vowel);
        let onset = if scan.has_leading_vowel {
            OnsetShape::LeadingVowel {
                base: scan.first_base,
            }
        } else if let Some(first_base) = scan.first_base {
            OnsetShape::Base(first_base)
        } else if scan.has_independent_vowel {
            OnsetShape::IndependentVowel(scan.character)
        } else if characters
            .first()
            .is_some_and(|value| ThaiScalar(value.character).is_dependent())
        {
            OnsetShape::DependentMark
        } else {
            OnsetShape::None
        };
        // Rule: dependent vowel material without a consonant base is a distinct
        // malformed shape. It must not become `None`, because resolution needs to
        // preserve it as text rather than absorb it into the preceding vowel.
        // Example: `กีา` atomizes as `กี` plus an `OrphanRime("า")`, so the latter
        // cannot be mistaken for a valid second syllable or a second rime.
        let is_orphan_rime = scan.has_nucleus
            && scan.first_base.is_none()
            && !scan.has_independent_vowel
            && (scan.has_vowel_rime || scan.has_ue_rime);
        let nucleus = if is_orphan_rime {
            NucleusShape::OrphanRime
        } else if !has_structural_nucleus {
            NucleusShape::None
        } else if scan.has_independent_vowel {
            NucleusShape::IndependentVowel
        } else if scan.has_vowel_rime || scan.has_ue_rime || scan.has_carrier || scan.has_glide {
            NucleusShape::Rime {
                explicit: scan.has_vowel_rime || scan.has_ue_rime,
                carrier: scan.has_carrier,
                completed_ue_carrier: scan.consumed_ue_carrier,
                glide: scan.has_glide,
            }
        } else {
            NucleusShape::Implicit
        };

        ThaiAtom {
            range: scan.range,
            onset,
            nucleus,
            has_tone_mark: scan.has_tone_mark,
            deferred_silent_coda: scan.deferred_silent_coda,
            // Thai PHINTHU (U+0E3A) suppresses the inherent vowel in Pali/Sanskrit
            // transcription and therefore never supplies nucleus evidence.
            has_phinthu: characters.iter().any(|value| value.character == '\u{0e3a}'),
            ending: if scan.has_hard_ending
                || characters
                    .iter()
                    .any(|value| ThaiScalar(value.character).is_garan_mark())
            {
                EndingShape::Hard
            } else {
                EndingShape::Open
            },
            silent_mark: if !characters
                .iter()
                .any(|value| ThaiScalar(value.character).is_garan_mark())
            {
                SilentMarkShape::None
            } else if scan.is_opaque {
                SilentMarkShape::Opaque {
                    has_carrier: characters
                        .iter()
                        .any(|value| ThaiScalar(value.character).is_vowel_carrier()),
                    has_initial_cluster: characters.get(..2).is_some_and(|values| {
                        ThaiScalar(values[0].character)
                            .forms_initial_cluster_with(ThaiScalar(values[1].character))
                    }),
                }
            } else {
                SilentMarkShape::Attached
            },
        }
    }

    // Rule: translate this atom-relative range only when a public part is
    // emitted, keeping byte-offset ownership in the atom type.
    // Example: the atom for `เกิน` is measured from the Thai-run start; when
    // the run follows text such as `Xเกิน`, adding the run offset still emits
    // exactly `เกิน` and never includes the preceding `X`.
    pub(crate) fn absolute_range(&self, source_offset: usize) -> ByteRange {
        (source_offset + self.range.start)..(source_offset + self.range.end)
    }

    pub(crate) fn relative_range(&self) -> &ByteRange {
        &self.range
    }

    pub(crate) fn first_base(&self) -> Option<char> {
        match self.onset {
            OnsetShape::Base(base) | OnsetShape::LeadingVowel { base: Some(base) } => Some(base),
            OnsetShape::None
            | OnsetShape::LeadingVowel { base: None }
            | OnsetShape::IndependentVowel(_)
            | OnsetShape::DependentMark => None,
        }
    }

    pub(crate) fn has_nucleus(&self) -> bool {
        matches!(
            self.nucleus,
            NucleusShape::Implicit | NucleusShape::Rime { .. } | NucleusShape::IndependentVowel
        )
    }

    pub(crate) fn has_explicit_rime(&self) -> bool {
        matches!(
            self.nucleus,
            NucleusShape::Rime { explicit: true, .. } | NucleusShape::IndependentVowel
        )
    }

    pub(crate) fn has_phinthu(&self) -> bool {
        self.has_phinthu
    }
}
