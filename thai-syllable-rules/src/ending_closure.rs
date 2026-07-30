use crate::character_sequence::CharacterSequence;

// The helpers in this module implement hard syllable endings and final
// extensions and silent marks. They report
// structural evidence to atomization; they do not choose lexical boundaries.
// Example: these helpers can mark `กำ` as closed and `ฟิล์ม` as an opaque
// silent-mark spelling, but `resolution` still decides how an adjacent bare
// consonant is exposed between neighboring candidates.
impl CharacterSequence<'_> {

    pub(crate) fn is_standalone_ko_taikhu(self, start: usize, end: usize) -> bool {
        let characters = self.0;
        // Rule: recognize exactly the independent `ก็` spelling: ก (U+0E01)
        // followed by taikhu ็ (U+0E47), with no preceding vowel pattern or
        // trailing continuation already consumed by atomization.
        characters.get(start..end).is_some_and(|values| {
            values.len() == 2
                && values[0].character == '\u{0e01}' // ก
                && values[1].character == '\u{0e47}' // ็
        })
    }
}
