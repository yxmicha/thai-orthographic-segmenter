#[derive(Clone, Copy, Debug)]
pub(crate) struct ThaiCharacter {
    pub(crate) character: char,
    pub(crate) start: usize,
    pub(crate) end: usize,
}

/// A Unicode scalar classified by its role in Thai orthography.
///
/// Wrap a `char` to ask what role it plays in written Thai:
///
/// ```
/// use thai_syllable_rules::ThaiScalar;
///
/// let consonant = ThaiScalar::new('ก');
/// assert!(consonant.is_base());
/// assert!(!consonant.is_dependent());
/// assert_eq!(char::from(consonant), 'ก');
/// assert_eq!(consonant.to_string(), "ก");
/// ```
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ThaiScalar(pub(crate) char);
