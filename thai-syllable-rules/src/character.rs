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

impl From<char> for ThaiScalar {
    fn from(character: char) -> Self {
        Self(character)
    }
}

impl std::fmt::Display for ThaiScalar {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Display::fmt(&self.0, formatter)
    }
}

impl ThaiScalar {

    /// Returns whether a Thai character is a dependent vowel, tone mark, or mark.
    ///
    /// # Examples
    ///
    /// ```
    /// use thai_syllable_rules::ThaiScalar;
    ///
    /// assert!(ThaiScalar::new('ะ').is_dependent());
    /// assert!(ThaiScalar::new('\u{0e48}').is_dependent()); // tone mark
    /// assert!(!ThaiScalar::new('ก').is_dependent());
    /// ```
    //
    // Rule: dependent characters stay inside the current orthographic atom; they
    // never become standalone public parts.
    // Examples: `ก่`, `กั`, `ก์`, and `เกิน` keep their tone, vowel, or silent
    // mark attached to the local base. A malformed leading `ิ` is preserved in
    // the source run but is not emitted as an independent `Part`.
    #[must_use]
    pub const fn is_dependent(self) -> bool {
        let character = self.0;
        matches!(
            character,
            '\u{0e30}' // ะ
                | '\u{0e31}' // อั
                | '\u{0e32}' // า
                | '\u{0e33}' // ำ
                | '\u{0e34}'..='\u{0e3a}' // อิ อี อึ อื อุ อู อฺ
                | '\u{0e45}' // ๅ
                | '\u{0e47}'..='\u{0e4e}' // อ็ อ่ อ้ อ๊ อ๋ อ์ อํ อ๎
        )
    }
}
