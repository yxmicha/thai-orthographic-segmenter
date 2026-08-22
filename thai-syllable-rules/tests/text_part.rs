#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TextPart<'a> {
    Syllable(&'a str),
    Undecided(&'a str),
    Invalid(&'a str),
    Dot(&'a str),
    Text(&'a str),
}
