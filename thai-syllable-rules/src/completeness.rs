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
