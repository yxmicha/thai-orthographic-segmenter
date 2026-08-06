use crate::character_sequence::CharacterSequence;
use winnow::{
    ModalResult,
    error::{ContextError, ErrMode},
};

#[derive(Clone, Copy, Debug)]
pub(crate) struct ThaiCharacter {
    pub(crate) character: char,
    pub(crate) start: usize,
    pub(crate) end: usize,
}

pub(crate) fn thai_characters(input: &mut &str) -> ModalResult<Vec<ThaiCharacter>> {
    // Rule: parse one maximal run from the accepted alphabet and retain each
    // scalar's byte range for exact source reconstruction.
    // Examples: the accepted portion of `ไทย 123` is one Thai run `ไทย`, then
    // whitespace and ASCII digits form hard source boundaries. Combining marks
    // remain in the run with their original byte ranges; no normalization changes
    // the substring later returned by `split`.
    let mut characters = Vec::with_capacity(input.len() / 3 + 1);
    let mut offset = 0;
    for character in input
        .chars()
        .take_while(|character| ThaiScalar(*character).is_accepted())
    {
        let start = offset;
        offset += character.len_utf8();
        characters.push(ThaiCharacter {
            character,
            start,
            end: offset,
        });
    }
    if characters.is_empty() {
        return Err(ErrMode::Backtrack(ContextError::new()));
    }
    *input = &input[offset..];
    Ok(characters)
}

impl CharacterSequence<'_> {
    pub(crate) fn contains_character(self, start: usize, end: usize, character: char) -> bool {
        let characters = self.0;
        // Rule helpers inspect only the current atom's scalar range; a matching
        // mark elsewhere in the Thai run cannot affect this construction.
        // Example: when checking `กะ` inside `กะกำ`, `contains_character` sees the
        // first `ะ` but not the later `ำ`; each atom's hard-ending decision stays
        // local to its own scalar range.
        characters[start..end]
            .iter()
            .any(|value| value.character == character)
    }
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

impl From<ThaiScalar> for char {
    fn from(scalar: ThaiScalar) -> Self {
        scalar.0
    }
}

impl std::fmt::Display for ThaiScalar {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Display::fmt(&self.0, formatter)
    }
}

impl ThaiScalar {
    /// Wraps `character` for classification.
    ///
    /// # Examples
    ///
    /// ```
    /// use thai_syllable_rules::ThaiScalar;
    ///
    /// assert!(ThaiScalar::new('ก').is_base());
    /// ```
    #[must_use]
    pub const fn new(character: char) -> Self {
        Self(character)
    }

    /// The wrapped character.
    ///
    /// # Examples
    ///
    /// ```
    /// use thai_syllable_rules::ThaiScalar;
    ///
    /// assert_eq!(ThaiScalar::new('ก').character(), 'ก');
    /// ```
    #[must_use]
    pub const fn character(self) -> char {
        self.0
    }

    /// Returns whether a character is an accepted Thai orthographic character.
    ///
    /// # Examples
    ///
    /// ```
    /// use thai_syllable_rules::ThaiScalar;
    ///
    /// assert!(ThaiScalar::new('ก').is_accepted());
    /// // Thai digits, the baht sign and other scripts stay outside Thai runs.
    /// assert!(!ThaiScalar::new('๑').is_accepted());
    /// assert!(!ThaiScalar::new('a').is_accepted());
    /// ```
    //
    // Rule: use an explicit allow-list of assigned Thai letters, vowels, and
    // marks. Thai punctuation, digits, the baht sign, and unassigned code points
    // remain outside Thai runs even though some share the Thai Unicode block.
    // Examples: `กะ` is one accepted Thai run, while `ก๑฿ข` is scanned as `ก`,
    // `๑฿`, and `ข` around a preserved non-Thai region. `ฯ`, `ๆ`, `๐`, and `฿`
    // therefore cannot create syllable candidates or silently disappear.
    //
    // Thai-block inventory:
    // U+0E00 is unassigned.
    // U+0E01..U+0E2E are accepted letters:
    // ก ข ฃ ค ฅ ฆ ง จ ฉ ช ซ ฌ ญ ฎ ฏ ฐ ฑ ฒ ณ ด ต ถ ท ธ น บ ป ผ ฝ พ ฟ ภ ม ย ร ฤ ล ฦ ว ศ ษ ส ห ฬ อ ฮ
    // U+0E2F is ฯ and is excluded.
    // U+0E30..U+0E3A are accepted marks: อะ อั อา อำ อิ อี อึ อื อุ อู อฺ.
    // U+0E3B..U+0E3E are unassigned; U+0E3F is ฿ and is excluded.
    // U+0E40..U+0E45 are accepted vowels: เ แ โ ใ ไ ๅ.
    // U+0E46 is ๆ and is excluded.
    // U+0E47..U+0E4E are accepted marks: อ็ อ่ อ้ อ๊ อ๋ อ์ อํ อ๎.
    // U+0E4F is ๏ and is excluded.
    // U+0E50..U+0E59 are Thai digits and are excluded: ๐ ๑ ๒ ๓ ๔ ๕ ๖ ๗ ๘ ๙.
    // U+0E5A..U+0E5B are ๚ ๛ and are excluded; U+0E5C..U+0E7F are unassigned.
    #[must_use]
    pub const fn is_accepted(self) -> bool {
        let character = self.0;
        matches!(
            character,
            '\u{0e01}'..='\u{0e2e}' // ก ข ฃ ค ฅ ฆ ง จ ฉ ช ซ ฌ ญ ฎ ฏ ฐ ฑ ฒ ณ ด ต ถ ท ธ น บ ป ผ ฝ พ ฟ ภ ม ย ร ฤ ล ฦ ว ศ ษ ส ห ฬ อ ฮ
                | '\u{0e30}'..='\u{0e3a}' // ะ อั า ำ อิ อี อึ อื อุ อู อฺ
                | '\u{0e40}'..='\u{0e45}' // เ แ โ ใ ไ ๅ
                | '\u{0e47}'..='\u{0e4e}' // อ็ อ่ อ้ อ๊ อ๋ อ์ อํ อ๎
        )
    }

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

    pub(crate) const fn is_coda(self) -> bool {
        let character = self.0;
        // Rule: only consonants with a Thai final-sound spelling can attach as a
        // coda. The complete accepted-base set that cannot be a bare coda is
        // ฃ, ฅ, ฉ, ฌ, ผ, ฝ, ห, อ, and ฮ; these must move into a following onset
        // or carrier construction, or remain with a garan-bearing spelling.
        // Examples: `แกหรอก` moves ห into `หรอก` because ห cannot close `แก`,
        // while `แกกรอก` keeps ก available as a possible coda; `แกห์รอก` keeps
        // ห with its garan construction instead of treating it as a bare coda.
        matches!(
            character,
            '\u{0e01}' // ก
                | '\u{0e02}' // ข
                | '\u{0e04}' // ค
                | '\u{0e06}' // ฆ
                | '\u{0e07}' // ง
                | '\u{0e08}' // จ
                | '\u{0e0a}' // ช
                | '\u{0e0b}' // ซ
                | '\u{0e0d}' // ญ
                | '\u{0e0e}' // ฎ
                | '\u{0e0f}' // ฏ
                | '\u{0e10}' // ฐ
                | '\u{0e11}' // ฑ
                | '\u{0e12}' // ฒ
                | '\u{0e13}' // ณ
                | '\u{0e14}' // ด
                | '\u{0e15}' // ต
                | '\u{0e16}' // ถ
                | '\u{0e17}' // ท
                | '\u{0e18}' // ธ
                | '\u{0e19}' // น
                | '\u{0e1a}' // บ
                | '\u{0e1b}' // ป
                | '\u{0e1e}' // พ
                | '\u{0e1f}' // ฟ
                | '\u{0e20}' // ภ
                | '\u{0e21}' // ม
                | '\u{0e22}' // ย
                | '\u{0e23}' // ร
                | '\u{0e25}' // ล
                | '\u{0e27}' // ว
                | '\u{0e28}' // ศ
                | '\u{0e29}' // ษ
                | '\u{0e2a}' // ส
                | '\u{0e2c}' // ฬ
        )
    }

    pub(crate) const fn is_vowel_carrier(self) -> bool {
        let character = self.0;
        // Thai vowel carrier: อ (U+0E2D).
        character == '\u{0e2d}' // อ
    }

    pub(crate) const fn is_garan_mark(self) -> bool {
        let character = self.0;
        // Thai garan mark: ์ (U+0E4C).
        // The mark silences the preceding base sequence; it is never a nucleus or
        // a public part by itself. Keeping this category in one predicate prevents
        // suffix scanners from disagreeing about what counts as garan.
        character == '\u{0e4c}' // ์
    }

    pub(crate) const fn is_glide_base(self) -> bool {
        let character = self.0;
        // Thai glide bases: ย (U+0E22) and ว (U+0E27).
        // The surrounding vowel pattern still decides whether either base is a
        // glide or the onset/coda of another candidate.
        matches!(character, '\u{0e22}' | '\u{0e27}') // ย ว
    }

    pub(crate) const fn is_final_r(self) -> bool {
        let character = self.0;
        // Thai final ร: ร (U+0E23).
        // This role-specific predicate keeps final-r extension logic independent
        // from raw scalar comparisons. Examples: `บัตร` may consume a final ร,
        // while `รอก` uses ร as the onset of the following syllable.
        character == '\u{0e23}' // ร
    }

    pub(crate) const fn is_inherent_final_leading_vowel(self) -> bool {
        let character = self.0;
        // Leading vowels with an inherent final: ใ (U+0E43) and ไ (U+0E44).
        matches!(character, '\u{0e43}' | '\u{0e44}') // ใ ไ
    }

    pub(crate) const fn is_vowel(self) -> bool {
        let character = self.0;
        // Rule: this predicate identifies every scalar that can supply a vowel
        // nucleus, including independent vowels and the long-vowel mark.
        // Examples: `ะ`, `า`, `ิ`, `ื`, `เ`, `ไ`, `ฤ`, and `ๅ` count as vowel-bearing
        // material for atomization. Tone marks such as `่` modify an atom but do not
        // themselves make a nucleus, so `ก่` still depends on its base structure.
        matches!(
            character,
            '\u{0e24}' // ฤ
                | '\u{0e26}' // ฦ
                | '\u{0e30}' // ะ
                | '\u{0e31}' // อั
                | '\u{0e32}' // า
                | '\u{0e33}' // ำ
                | ('\u{0e34}'..='\u{0e39}') /* อิ อี อึ อื อุ อู */
                | '\u{0e45}' // ๅ
        )
    }

    /// Returns whether a Thai character is an independent vowel (ฤ or ฦ), which carries a nucleus
    /// without a consonant base.
    ///
    /// # Examples
    ///
    /// ```
    /// use thai_syllable_rules::ThaiScalar;
    ///
    /// assert!(ThaiScalar::new('ฤ').is_independent_vowel());
    /// assert!(!ThaiScalar::new('ก').is_independent_vowel());
    /// ```
    #[must_use]
    pub const fn is_independent_vowel(self) -> bool {
        let character = self.0;
        // Rule: ฤ and ฦ can carry a nucleus without a preceding consonant base.
        // Examples: both `ฤ` and `ฦ` can begin a candidate in `ฤดู` or `ฦๅ`; they are
        // not forced to wait for a preceding `ก`-like base. A following garan sequence
        // is handled by the silent-mark rules rather than by base classification.
        matches!(
            character,
            '\u{0e24}' // ฤ
                | '\u{0e26}' // ฦ
        )
    }

    pub(crate) const fn is_tone_mark(self) -> bool {
        let character = self.0;
        // Rule: tone marks modify the current onset and do not start a part.
        // Examples: `ก้`, `ก่`, and `ฟอร์ม` keep `้`/`่` with the surrounding spelling;
        // no `Part::Syllable("้")` or `Part::Undecided("่")` can be emitted. A tone
        // mark on a bare consonant may affect later attachment, but not its category.
        matches!(
            character,
            '\u{0e48}' // อ่
                | '\u{0e49}' // อ้
                | '\u{0e4a}' // อ๊
                | '\u{0e4b}' // อ๋
        )
    }

    /// Returns whether two adjacent Thai bases form an attested initial cluster.
    ///
    /// This is the shared orthographic inventory used by the rules and by bounded
    /// downstream resolvers when deciding whether an undecided base may become the
    /// onset of a following candidate. A non-cluster pair must remain available as
    /// a syllable boundary; it must not be accepted merely because the concatenated
    /// text resembles a dictionary word.
    ///
    /// # Examples
    ///
    /// ```
    /// use thai_syllable_rules::ThaiScalar;
    ///
    /// // กร is a listed initial cluster; ขก is not.
    /// assert!(ThaiScalar::new('ก').forms_initial_cluster_with(ThaiScalar::new('ร')));
    /// assert!(!ThaiScalar::new('ข').forms_initial_cluster_with(ThaiScalar::new('ก')));
    /// ```
    #[must_use]
    pub const fn forms_initial_cluster_with(self, second: Self) -> bool {
        let (first, second) = (self.0, second.0);
        // Rule: only this explicit onset inventory may keep two adjacent bases in
        // one initial cluster. Other adjacent bases remain attachment candidates.
        // Examples: `กร`, `กล`, `กว`, `คร`, `ตร`, `พล`, `หญ`, and `อย` are listed
        // pairs, so `กรา` can keep `กร` in one onset. A neighboring pair such as `กม`
        // is not silently promoted to a cluster; its attachment remains visible to
        // the implicit and ambiguity resolvers.
        matches!(
            (first, second),
            (
                '\u{0e01}' | '\u{0e02}' | '\u{0e04}',
                '\u{0e23}' | '\u{0e25}' | '\u{0e27}'
            ) | (
                '\u{0e08}'
                    | '\u{0e0b}'
                    | '\u{0e15}'
                    | '\u{0e14}'
                    | '\u{0e17}'
                    | '\u{0e28}'
                    | '\u{0e2a}',
                '\u{0e23}'
            ) | (
                '\u{0e1b}' | '\u{0e1e}' | '\u{0e1a}' | '\u{0e1f}',
                '\u{0e23}' | '\u{0e25}'
            ) | ('\u{0e1c}', '\u{0e25}')
                | ('\u{0e2d}', '\u{0e22}')
                | (
                    '\u{0e01}' | '\u{0e15}' | '\u{0e1b}' | '\u{0e1e}' | '\u{0e17}',
                    '\u{0e24}'
                )
                | (
                    '\u{0e2b}',
                    '\u{0e07}'
                        | '\u{0e0d}'
                        | '\u{0e19}'
                        | '\u{0e21}'
                        | '\u{0e22}'
                        | '\u{0e23}'
                        | '\u{0e25}'
                        | '\u{0e27}'
                )
        )
    }

    pub(crate) const fn forms_final_r_cluster_with(self, second: Self) -> bool {
        let (first, second) = (self.0, second.0);
        // Final-ร clusters that are also valid initial pairs but may serve as a
        // preceding rime extension: กร, ตร, ทร, and สร. This is a role-specific
        // subset of the authoritative initial-cluster inventory, not a second
        // independent cluster table.
        matches!(
            (first, second),
            (
                '\u{0e01}' | '\u{0e15}' | '\u{0e17}' | '\u{0e2a}',
                '\u{0e23}'
            ) // ส ร
        )
    }
}
