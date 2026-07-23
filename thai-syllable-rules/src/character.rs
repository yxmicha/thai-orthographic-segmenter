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

    /// Returns whether a Thai character is a preposed vowel.
    ///
    /// # Examples
    ///
    /// ```
    /// use thai_syllable_rules::ThaiScalar;
    ///
    /// assert!(ThaiScalar::new('เ').is_leading_vowel());
    /// assert!(!ThaiScalar::new('ะ').is_leading_vowel());
    /// ```
    //
    // Rule: these vowels begin an atom and are resolved together with the base
    // that follows them.
    // Examples: `เกา`, `แสง`, `โก`, `ใจ`, and `ไตร` begin with `เ`, `แ`, `โ`,
    // `ใ`, or `ไ`; the following base and any legal marks are considered by the
    // same atomization step. A leading vowel without a base, such as `เแท`'s
    // first `เ`, is later preserved as malformed `Text` rather than joined to a
    // different vowel pattern.
    #[must_use]
    pub const fn is_leading_vowel(self) -> bool {
        let character = self.0;
        matches!(
            character,
            '\u{0e40}' // เ
                | '\u{0e41}' // แ
                | '\u{0e42}' // โ
                | '\u{0e43}' // ใ
                | '\u{0e44}' // ไ
        )
    }

    /// Returns whether a Thai character is a consonant base.
    ///
    /// # Examples
    ///
    /// ```
    /// use thai_syllable_rules::ThaiScalar;
    ///
    /// assert!(ThaiScalar::new('ก').is_base());
    /// assert!(!ThaiScalar::new('เ').is_base());
    /// ```
    //
    // Rule: ฤ and ฦ are independent vowels, not consonant bases, so they are
    // excluded from this predicate even though they are in the letter range.
    // Examples: `ฤดู` starts with an independent nucleus, whereas `ก` is a base
    // and can begin an implicit or explicit consonant-led construction. This
    // distinction prevents `ฤ` from being mistaken for an onset in `ฤทธิ์`.
    #[must_use]
    pub const fn is_base(self) -> bool {
        let character = self.0;
        character >= '\u{0e01}'
            && character <= '\u{0e2e}' // ก ข ฃ ค ฅ ฆ ง จ ฉ ช ซ ฌ ญ ฎ ฏ ฐ ฑ ฒ ณ ด ต ถ ท ธ น บ ป ผ ฝ พ ฟ ภ ม ย ร ฤ ล ฦ ว ศ ษ ส ห ฬ อ ฮ
            && character != '\u{0e24}' // ฤ
            && character != '\u{0e26}' // ฦ
    }
}
