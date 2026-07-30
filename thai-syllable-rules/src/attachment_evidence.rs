/// Closure supplied by the preceding rime, independently of coda eligibility.
#[derive(Clone, Copy)]
pub(crate) enum LocalClosure {
    OptionalCoda,
    RequiredCoda,
    ToneMarkedCarrier,
}
