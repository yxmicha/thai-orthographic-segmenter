use crate::character::ThaiScalar;
use crate::character_sequence::CharacterSequence;

impl CharacterSequence<'_> {
    pub(crate) fn starts_following_initial_cluster(self, index: usize) -> bool {
        let characters = self.0;
        // Rule: a listed onset pair is evidence that the next base run begins a
        // following syllable rather than extending the current final.
        // Examples: after a closed candidate, `กรา` is recognized as a following
        // onset construction, while an unlisted `กม` remains an attachment
        // question for implicit resolution.
        characters.get(index..index + 2).is_some_and(|values| {
            ThaiScalar(values[0].character)
                .forms_initial_cluster_with(ThaiScalar(values[1].character))
        })
    }
}
