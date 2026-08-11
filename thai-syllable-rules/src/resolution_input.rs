/// The source context immediately following a maximal Thai run.
///
/// This is deliberately richer than a boolean so source punctuation policy is
/// named at the source/Thai seam and cannot silently spread as an unrelated
/// flag through ownership resolution.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum FollowingSource {
    /// The Thai run is not immediately followed by one eligible period.
    None,
    /// The Thai run is immediately followed by exactly one ASCII period.
    SinglePeriod,
}
