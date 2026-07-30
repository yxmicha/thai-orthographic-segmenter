/// Closure supplied by the preceding rime, independently of coda eligibility.
#[derive(Clone, Copy)]
pub(crate) enum LocalClosure {
    OptionalCoda,
    RequiredCoda,
    ToneMarkedCarrier,
}

/// A possible coda is not an ownership proof when a right reading survives.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum AttachmentEvidence {
    Ineligible,
    LeftOnly,
    Undecided,
}
