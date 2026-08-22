use thai_syllable_rules::Part;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TextPart<'a> {
    Syllable(&'a str),
    Undecided(&'a str),
    Invalid(&'a str),
    Dot(&'a str),
    Text(&'a str),
}

#[must_use]
pub fn text_parts<'a>(source: &'a str, parts: &[Part]) -> Vec<TextPart<'a>> {
    parts
        .iter()
        .map(|part| match part {
            Part::Syllable(_) => TextPart::Syllable(part.text(source)),
            Part::Undecided(_) => TextPart::Undecided(part.text(source)),
            Part::Invalid(_) => TextPart::Invalid(part.text(source)),
            Part::Dot(_) => TextPart::Dot(part.text(source)),
            Part::Text(_) => TextPart::Text(part.text(source)),
        })
        .collect()
}
