use pyo3::prelude::*;
use thai_syllable_rules::{Part, split};

fn kind(part: &Part) -> &'static str {
    match part {
        Part::Syllable(_) => "syllable",
        Part::Undecided(_) => "undecided",
        Part::Invalid(_) => "invalid",
        Part::Dot(_) => "dot",
        Part::Text(_) => "text",
    }
}
