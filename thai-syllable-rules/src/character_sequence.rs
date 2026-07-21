use crate::character::ThaiCharacter;

/// A borrowed run of Thai characters with their source byte ranges.
///
/// Structural rules that inspect neighbouring characters (atomization, ending closure,
/// lookahead, รหัน and reading-alternative recognition) are methods on this view, so the
/// sequence they examine is always the receiver rather than an ad hoc slice argument.
#[derive(Clone, Copy)]
pub(crate) struct CharacterSequence<'a>(pub(crate) &'a [ThaiCharacter]);
