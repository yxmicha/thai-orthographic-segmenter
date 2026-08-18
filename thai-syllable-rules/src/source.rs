use winnow::error::ContextError;
use winnow::{
    Parser,
    combinator::{alt, repeat},
    token::{take_till, take_while},
};

use crate::character::ThaiScalar;
use crate::part::{Part, push_text};
use crate::resolution_input::FollowingSource;
use crate::thai_legality::ThaiLegality;

fn split_source(text: &str) -> Vec<Part> {
    // Rule: scan maximal runs without normalizing, dropping, or reordering
    // any scalar. The parser receives only the Thai runs from this layer.
    // Example: scanning `กะ ภาษา.` yields Thai chunks `กะ` and `ภาษา`, a
    // whitespace chunk, and a period chunk; concatenating all chunks returns
    // the exact input byte-for-byte. This is why range offsets remain stable.
    let mut parser = repeat::<_, _, Vec<SourceChunk<'_>>, ContextError, _>(
        0..,
        alt((
            take_while(1.., |character| ThaiScalar(character).is_accepted()).map(SourceChunk::Thai),
            take_while(1.., |character: char| character == '.').map(SourceChunk::Periods),
            take_till(1.., |character: char| {
                ThaiScalar(character).is_accepted() || character == '.'
            })
            .map(SourceChunk::Text),
        )),
    );
    let Ok(chunks) = parser.parse(text) else {
        return if text.is_empty() {
            Vec::new()
        } else {
            vec![Part::Text(0..text.len())]
        };
    };

    let mut parts = Vec::new();
    let mut cursor = 0;

    for (index, chunk) in chunks.iter().enumerate() {
        let end = cursor + chunk.text().len();
        match chunk {
            SourceChunk::Thai(thai_text) => {
                // Rule: a non-Thai chunk ends the previous Thai run, so
                // attachment decisions never cross whitespace or punctuation.
                // Examples: `กะ มเหสี` cannot let the leading `ม` attach across
                // the space, and `นะ,กะ` keeps the comma between independent
                // Thai resolutions. The same applies to ASCII and Thai digits.
                let following_source = if chunks
                    .get(index + 1)
                    .is_some_and(|chunk| matches!(chunk, SourceChunk::Periods(".")))
                {
                    FollowingSource::SinglePeriod
                } else {
                    FollowingSource::None
                };
                for part in ThaiLegality::new(thai_text, cursor, following_source).resolve() {
                    match part {
                        Part::Text(range) => push_text(&mut parts, range),
                        Part::Syllable(_)
                        | Part::Undecided(_)
                        | Part::Invalid(_)
                        | Part::Dot(_) => {
                            parts.push(part);
                        }
                    }
                }
            }
            SourceChunk::Periods(periods)
                if *periods == "." && parts.last().is_some_and(Part::can_precede_dot) =>
            {
                // Rule: exactly one adjacent period is punctuation only after
                // a structural syllable or an explicitly ambiguous part.
                // Examples: `นะ.` and `นะก.` produce a `Dot`, while `.นะ`,
                // `นะ..`, and `นะ .` keep the period as `Text` because the
                // required adjacency or single-period condition is absent.
                parts.push(Part::Dot(cursor..end));
            }
            SourceChunk::Periods(_) | SourceChunk::Text(_) => {
                // Rule: invalid periods and all other non-Thai material are
                // retained as Text rather than being interpreted as Thai.
                // Examples: `นะ..`, `นะ,`, `นะ  ก`, and `abc` preserve their
                // punctuation, whitespace, and Latin text. No invalid period
                // can become a `Dot` merely because a Thai syllable is nearby.
                push_text(&mut parts, cursor..end);
            }
        }
        cursor = end;
    }

    parts
}

impl Part {
    fn can_precede_dot(&self) -> bool {
        // Rule: a period has Thai punctuation status only immediately after a
        // Syllable or Undecided part; the source scanner has already enforced
        // adjacency.
        // Examples: `นะ.` qualifies after `Syllable("นะ")`, and `นะก.` qualifies
        // after `Undecided("ก")`. A period after `Text`, after another period, or
        // across a space is not eligible.
        matches!(self, Part::Syllable(_) | Part::Undecided(_))
    }
}

#[derive(Clone, Copy, Debug)]
enum SourceChunk<'a> {
    // Rule: accepted Thai scalars are parsed as one maximal Thai run.
    // Examples: `พระคัมภีร์` is sent to Thai atomization as one run, while a
    // space in `พระ คัมภีร์` ends the first run before the second begins. The
    // run boundary is based on the explicit Thai alphabet, not on Unicode's
    // broad Thai block alone.
    Thai(&'a str),
    // Rule: periods are scanned separately so only one eligible period can
    // become Part::Dot; repeated or misplaced periods remain Part::Text.
    // Examples: `นะ.` can become `Syllable("นะ") | Dot(".")`, but `นะ..`
    // keeps the punctuation text-visible. A leading `.นะ` is also text until
    // a valid Thai part immediately precedes a single period.
    Periods(&'a str),
    // Rule: every scalar outside the accepted Thai alphabet and period is
    // preserved as source text and forms a hard boundary around Thai runs.
    // Examples: `ไทย English` preserves the ASCII word and space, and `ก๑฿ข`
    // prevents `ก` from attaching to `ข`. Punctuation and digits remain in
    // their original order instead of being normalized away.
    Text(&'a str),
}

impl SourceChunk<'_> {
    fn text(&self) -> &str {
        match self {
            Self::Thai(text) | Self::Periods(text) | Self::Text(text) => text,
        }
    }
}
