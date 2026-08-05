use crate::character::ThaiCharacter;
use crate::character::ThaiScalar;
use crate::character_sequence::CharacterSequence;
use winnow::{
    ModalResult, Parser,
    combinator::{alt, opt, repeat},
    token::any,
};

impl CharacterSequence<'_> {
    pub(crate) fn rime_has_explicit_hard_vowel(self, start: usize, end: usize) -> bool {
        let characters = self.0;
        // Rule: a glide may not absorb a following base when its own rime already
        // contains a hard vowel. Keeping this probe beside rime consumption makes
        // the glide decision use the same semantic vowel vocabulary as atomization.
        // Examples: `เปี๊ยะ` may consume its permitted glide, while a hard rime
        // remains closed so a later base can begin another candidate.
        characters[start..end]
            .iter()
            // ะ and ำ are explicit hard vowel endings that close a local spelling.
            .any(|value| matches!(value.character, '\u{0e30}' | '\u{0e33}'))
            || characters[start..end]
                .windows(2)
                .any(|pair| pair[0].character == 'ํ' && pair[1].character == 'า')
            || characters[start..end].windows(3).any(|triple| {
                triple[0].character == 'ํ'
                    && ThaiScalar(triple[1].character).is_tone_mark()
                    && triple[2].character == 'า'
            })
    }

    pub(crate) fn consume_rime(
        self,
        index: &mut usize,
        has_nucleus: &mut bool,
        allow_compound_vowel: bool,
    ) -> (bool, bool, bool) {
        // Rule: every base-led construction consumes its dependent rime through
        // the same local parser, then applies the short-ั coda rule at the rime's
        // boundary. Keeping this sequence shared prevents leading-vowel atoms,
        // carrier atoms, and ordinary base atoms from acquiring different vowel
        // or final semantics merely because they entered through different parser
        // branches.
        let rime_start = *index;
        let has_vowel_rime =
            self.consume_dependent_characters(index, has_nucleus, allow_compound_vowel);
        let has_ue_rime = self.contains_character(rime_start, *index, '\u{0e37}'); // ื
        let consumed_short_a_coda = 'consume_coda_required_by_short_a: {
            let characters = self.0;
            // Short-ั requires a final, but cannot take a marked onset from the next
            // vowel pattern. Post-final silent ิ/ุ and garan remain supported; marked
            // ย/ว belongs to the glide parser rather than the ordinary coda path.
            // Examples: `กัด` consumes `ด`, but a glide-shaped continuation such as
            // `กัวยะ` leaves `ย` available when its following mark proves that it is
            // part of the glide rather than an ordinary final.
            if self.contains_character(rime_start, *index, '\u{0e31}') // อั
                && characters
                    .get(*index)
                    .is_some_and(|value| ThaiScalar(value.character).is_coda())
                && !characters
                    .get(*index + 1)
                    .is_some_and(|value| {
                        ThaiScalar(value.character).is_dependent()
                            && (ThaiScalar(characters[*index].character).is_glide_base()
                                || !matches!(value.character, 'ิ' | 'ุ' | '์'))
                    })
            {
                *index += 1;
                break 'consume_coda_required_by_short_a true;
            }
            false
        };
        (has_vowel_rime, has_ue_rime, consumed_short_a_coda)
    }

    pub(crate) fn consume_dependent_characters(
        self,
        index: &mut usize,
        has_nucleus: &mut bool,
        allow_compound_vowel: bool,
    ) -> bool {
        let characters = self.0;
        // Rule: parse one dependent-mark sequence as one vowel pattern plus any
        // surrounding tone, garan, or other non-vowel marks; stop before a second
        // spoken-vowel mark so it cannot be hidden in the same atom. The parser
        // admits only the explicit compound `าะ` pattern and repeated `า`
        // lengthening as additional vowel-shape forms.
        // Examples: `เกิน` consumes its one `ิ` pattern, `เคราะห์` keeps `าะ`,
        // and `ค่าา` keeps repeated `า`; malformed `กีา` stops before the second
        // `า`, allowing resolution to preserve that orphan mark as `Text` instead
        // of emitting one syllable with two unrelated spoken vowels.
        let rime_start = *index;
        let mut input = &characters[*index..];
        let mut parser = alt((
            (
                vowel_dependent_character,
                repeat::<_, ThaiCharacter, (), _, _>(0.., non_vowel_dependent_character).void(),
            )
                .map(|_| true),
            (
                repeat::<_, ThaiCharacter, (), _, _>(1.., non_vowel_dependent_character).void(),
                opt(vowel_dependent_character),
                repeat::<_, ThaiCharacter, (), _, _>(0.., non_vowel_dependent_character).void(),
            )
                .map(|((), vowel, ())| vowel.is_some()),
        ));
        let Ok(has_vowel_rime) = parser.parse_next(&mut input) else {
            return false;
        };
        *index += characters[*index..].len() - input.len();
        let closed_rime = self.rime_has_explicit_hard_vowel(rime_start, *index);
        if allow_compound_vowel
            && !closed_rime
            && characters[*index - 1].character == '\u{0e32}' // า
            && characters
                .get(*index)
                .is_some_and(|value| value.character == '\u{0e30}')
        // ะ
        {
            // Rule: the `าะ` spelling is one compound vowel pattern, even though
            // its tail contains both า and ะ. Consume only that recognized pair;
            // malformed adjacent dependent vowels elsewhere remain outside the
            // atom.
            // Examples: `พาะ` and the second spelling in `เฉพาะ` keep `าะ`
            // together, while `กีา` and `กุา` still stop before their second,
            // unsupported dependent vowel.
            let mut parser = short_a_dependent_character;
            if parser.parse_next(&mut input).is_ok() {
                *index += 1;
            }
        } else if !closed_rime && characters[*index - 1].character == '\u{0e32}' // า
            && characters
                .get(*index)
                .is_some_and(|value| value.character == '\u{0e32}')
        // า
        {
            // Rule: repeated long-า marks are an orthographic lengthening of the
            // same vowel pattern, not a new base-less syllable. Consume the run
            // only when it repeats the exact preceding vowel mark.
            // Example: informal `ค่าา` remains one candidate, while unlike-mark
            // sequences such as `กีา` still stop before the second vowel.
            let remaining_before = input.len();
            let mut parser =
                repeat::<_, ThaiCharacter, (), _, _>(1.., long_a_dependent_character).void();
            if parser.parse_next(&mut input).is_ok() {
                *index += remaining_before - input.len();
            }
        }
        *has_nucleus |= has_vowel_rime;
        has_vowel_rime
    }
}

fn vowel_dependent_character(input: &mut &[ThaiCharacter]) -> ModalResult<ThaiCharacter> {
    any.verify(|value: &ThaiCharacter| {
        ThaiScalar(value.character).is_dependent() && ThaiScalar(value.character).is_vowel()
    })
    .parse_next(input)
}

fn non_vowel_dependent_character(input: &mut &[ThaiCharacter]) -> ModalResult<ThaiCharacter> {
    any.verify(|value: &ThaiCharacter| {
        ThaiScalar(value.character).is_dependent() && !ThaiScalar(value.character).is_vowel()
    })
    .parse_next(input)
}

pub(crate) fn short_a_dependent_character(
    input: &mut &[ThaiCharacter],
) -> ModalResult<ThaiCharacter> {
    any.verify(|value: &ThaiCharacter| value.character == '\u{0e30}') // ะ
        .parse_next(input)
}

pub(crate) fn long_a_dependent_character(
    input: &mut &[ThaiCharacter],
) -> ModalResult<ThaiCharacter> {
    any.verify(|value: &ThaiCharacter| value.character == '\u{0e32}') // า
        .parse_next(input)
}
