use std::ops::Range;

/// Groups of texts each worker thread claims, per worker, so uneven cores finish together.
const GROUPS_PER_WORKER: usize = 4;
/// Smallest group of texts worth scoring together, in bytes of text.
const MINIMUM_GROUP_BYTES: usize = 1024;

/// Consecutive groups of texts of similar total length, scored together by one worker.
pub struct TextGroups(Vec<Range<usize>>);

impl TextGroups {
    /// Divides `texts` into about [`GROUPS_PER_WORKER`] groups per worker, none shorter than
    /// [`MINIMUM_GROUP_BYTES`] unless it is the last.
    pub fn plan(texts: &[&str], workers: usize) -> Self {
        let target = texts
            .iter()
            .map(|text| text.len())
            .sum::<usize>()
            .div_ceil((workers * GROUPS_PER_WORKER).max(1))
            .max(MINIMUM_GROUP_BYTES);
        let mut groups = Vec::new();
        let (mut first, mut length) = (0, 0);
        for (index, text) in texts.iter().enumerate() {
            length += text.len();
            if length >= target {
                groups.push(first..index + 1);
                (first, length) = (index + 1, 0);
            }
        }
        if first < texts.len() {
            groups.push(first..texts.len());
        }
        Self(groups)
    }
}
