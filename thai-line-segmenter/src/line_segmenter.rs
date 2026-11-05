use std::{borrow::Cow, sync::LazyLock};

use regex::Regex;
use thai_word_segmenter::WordSegmenter;

use crate::{
    Error, NON_BREAKING_SPACE, ZERO_WIDTH_JOINER, ZERO_WIDTH_SPACE, icu_dictionary::IcuDictionary,
};

const DEFAULT_ICU_DICTIONARY: &str = include_str!("../data/icu/th-icu.txt");
const ZERO_WIDTH_NON_JOINER: char = '\u{200C}';
const ZERO_WIDTH_NO_BREAK_SPACE: char = '\u{FEFF}';
const ZERO_WIDTH_CHARACTERS: [char; 4] = [
    ZERO_WIDTH_SPACE,
    ZERO_WIDTH_NON_JOINER,
    ZERO_WIDTH_JOINER,
    ZERO_WIDTH_NO_BREAK_SPACE,
];
const SPECIAL_SPACES: [char; 5] = [
    NON_BREAKING_SPACE,
    '\u{202F}',
    '\u{2009}',
    '\u{200A}',
    '\u{205F}',
];

static SEGMENT_PATTERN: LazyLock<Result<Regex, regex::Error>> = LazyLock::new(|| {
    Regex::new(
        r#"(?x)
        (?P<punctuation_prefix>
            [
                \(      # (
                \$      # $
                0-9     # digits
                \u0E50-\u0E59      # Thai digits
                \u0ED0-\u0ED9      # Lao digits
                \u1040-\u1049      # Burmese digits
                \u17E0-\u17E9      # Khmer digits
                \u2070             # Superscript digit 0
                \u00b2\u00b3\u00b9 # Superscript digits 1-3
                \u2074-\u2079      # Superscript digits 4-9
                \u2080-\u2089      # Subscript digits 0-9
                \[      # [
                \{      # {
                "       # "
                '       # '
                \u2019  # ’
                \u201C  # “
                \u00AB  # «
                \u2039  # ‹
                \u2012  # ‒
                \u2013  # –
                \u2014  # —
                \u2015  # ―
                \u2016  # ‖
                \u2017  # ‗
                \u2026  # …
                \u00B7  # ·
                \u2022  # •
            ]+
            [
                \          # Normal Space
                \u00A0     # Non-Breaking Space
                \u202F     # Narrow No-Break Space
                \u2009     # Thin Space
                \u200A     # Hair Space
                \u205F     # Medium Mathematical Space
            ]*
        )
      |
        (?P<punctuation_suffix>
            [
                \          # Normal Space
                \u00A0     # Non-Breaking Space
                \u202F     # Narrow No-Break Space
                \u2009     # Thin Space
                \u200A     # Hair Space
                \u205F     # Medium Mathematical Space
            ]*
            [
                "       # "
                !       # !
                %       # %
                $       # $
                '       # '
                \)      # )
                \+      # +
                ,       # ,
                -       # -
                \.      # .
                :       # :
                ;       # ;
                =       # =
                >       # >
                \?      # ?
                \]      # ]
                `       # `
                |       # |
                }       # }
                ~       # ~
                \u201D  # ”
                \u2019  # ’
                \u00BB  # »
                \u203A  # ›
                \u0E2F  # ฯ
                \u0E46  # ๆ
            ]+
        )
      |
        (?P<thai>
            [
                \u200B     # Zero Width Space
                \u200C     # Zero Width Non-Joiner
                \u200D     # Zero Width Joiner
                \uFEFF     # Zero Width No-Break Space / BOM
                \u0E00-\u0E2E
                \u0E30-\u0E45
                \u0E47-\u0E4F
                \u0E5A-\u0E7F     # Thai punctuation and unassigned code points
            ]+
        )
      |
        (?P<untreated>
            [
                ^
                "          # "
                !          # !
                %          # %
                $          # $
                '          # '
                +          # +
                ,          # ,
                -          # -
                .          # .
                :          # :
                ;          # ;
                =          # =
                >          # >
                ?          # ?
                |          # |
                ~          # ~
                `          # `
                \( \)      # ( )
                \[ \]      # [ ]
                \{ \}      # { }
                0-9        # digits
                \u2070             # Superscript digit 0
                \u00b2\u00b3\u00b9 # Superscript digits 1-3
                \u2074-\u2079      # Superscript digits 4-9
                \u2080-\u2089      # Subscript digits 0-9
                \u2019 \u201C      # ’ “
                \u00AB \u00BB      # « »
                \u2039 \u203A      # ‹ ›
                \u2012     # ‒
                \u2013     # –
                \u2014     # —
                \u2015     # ―
                \u2016     # ‖
                \u2017     # ‗
                \u2026     # …
                \u00B7     # ·
                \u2022     # •
                \u201D     # ”
                \u0E00-\u0E7F  # Thai
            ]+
        )
    "#,
    )
});

/// Thai line segmenter that combines neural word segmentation with punctuation attachment
/// and ICU zero-width joiner insertion.
///
/// # Examples
///
/// ```
/// use thai_line_segmenter::{LineSegmenter, ZERO_WIDTH_JOINER, ZERO_WIDTH_SPACE};
///
/// let segmenter = LineSegmenter::new()?;
/// let marked = segmenter.normalize_line_boundaries("ในปฐมกาลพระวาทะ")?;
/// // A zero-width space marks the break between words, joiners keep the words together.
/// assert!(marked.contains(ZERO_WIDTH_SPACE));
/// assert!(marked.contains(ZERO_WIDTH_JOINER));
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
pub struct LineSegmenter {
    word_segmenter: WordSegmenter,
    icu_dictionary: IcuDictionary,
}

impl std::fmt::Debug for LineSegmenter {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("LineSegmenter")
            .finish_non_exhaustive()
    }
}

impl LineSegmenter {
    /// Creates a new line segmenter using the bundled epoch-6 word segmenter and default ICU dictionary.
    ///
    /// # Errors
    ///
    /// Returns an [`Error`] when the bundled word model cannot be loaded.
    ///
    /// # Examples
    ///
    /// ```
    /// use thai_line_segmenter::LineSegmenter;
    ///
    /// // Building a segmenter takes about 40 ms; build one and reuse it, from any number of threads.
    /// let segmenter = LineSegmenter::new()?;
    /// # drop(segmenter);
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn new() -> Result<Self, Error> {
        Self::with_icu_dictionary(DEFAULT_ICU_DICTIONARY)
    }

    /// Creates a new line segmenter with a custom ICU dictionary.
    ///
    /// # Errors
    ///
    /// Returns [`Error::IcuDictionary`] when an entry contains a NUL character, or
    /// [`Error::Words`] when the bundled word model cannot be loaded.
    ///
    /// # Examples
    ///
    /// ```
    /// use thai_line_segmenter::LineSegmenter;
    ///
    /// let segmenter = LineSegmenter::with_icu_dictionary("ปฐม\nกาล\n")?;
    /// assert_eq!(segmenter.normalize_line_boundaries("")?, "");
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn with_icu_dictionary(icu_dictionary: &str) -> Result<Self, Error> {
        Ok(Self {
            word_segmenter: WordSegmenter::new()?,
            icu_dictionary: IcuDictionary::new(icu_dictionary)?,
        })
    }
}

enum Content<'a> {
    Thai(&'a str),
    Untreated(&'a str),
}
