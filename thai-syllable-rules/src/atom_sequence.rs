use crate::atom::ThaiAtom;

/// A borrowed run of local Thai atoms; the receiver for implicit-vowel candidate rules.
#[derive(Clone, Copy)]
pub(crate) struct AtomSequence<'a>(pub(crate) &'a [ThaiAtom]);
