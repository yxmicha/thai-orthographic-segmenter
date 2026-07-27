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
}

fn vowel_dependent_character(input: &mut &[ThaiCharacter]) -> ModalResult<ThaiCharacter> {
    any.verify(|value: &ThaiCharacter| {
        ThaiScalar(value.character).is_dependent() && ThaiScalar(value.character).is_vowel()
    })
    .parse_next(input)
}
