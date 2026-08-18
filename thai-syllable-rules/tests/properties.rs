//! Properties of the rule parser on arbitrary text: it never panics, and its parts partition the
//! input on character boundaries.

use proptest::prelude::*;

/// Thai letters, vowels and marks mixed with ASCII, spaces, dots and other scripts.
fn text() -> impl Strategy<Value = String> {
    proptest::collection::vec(
        prop_oneof![
            8 => proptest::char::range('\u{0e00}', '\u{0e7f}'),
            2 => proptest::char::range('a', 'z'),
            1 => prop_oneof![Just(' '), Just('.'), Just('\n'), Just('\u{200b}'), Just('ก')],
            1 => any::<char>(),
        ],
        0..40,
    )
    .prop_map(|characters| characters.into_iter().collect())
}
