#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum OnsetShape {
    None,
    Base(char),
    LeadingVowel { base: Option<char> },
    IndependentVowel(char),
    DependentMark,
}
