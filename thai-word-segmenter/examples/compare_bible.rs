#![allow(
    clippy::cast_precision_loss,
    reason = "report figures are ratios of counts far below 2^52"
)]
//! Compares word boundaries against zero-width-space annotated Bible chapters.
//!
//! Usage: `cargo run -p thai-word-segmenter --example compare_bible -- <chapter.json>`

use std::{collections::BTreeSet, env, fs, ops::AddAssign, path::PathBuf};
use thai_word_segmenter::WordSegmenter;

const ZERO_WIDTH_SPACE: char = '\u{200b}';
const PRINTED_VERSES: usize = 20;

#[derive(serde::Deserialize)]
struct Chunk {
    #[serde(rename = "t")]
    tag: String,
    #[serde(rename = "v")]
    value: Option<String>,
}

/// Annotated verse text whose zero-width spaces mark reference word boundaries.
struct Verse {
    number: String,
    marked_text: String,
}

#[derive(Default)]
struct BoundaryCounts {
    true_positives: usize,
    false_positives: usize,
    false_negatives: usize,
    true_negatives: usize,
}

impl BoundaryCounts {
    fn total(&self) -> usize {
        self.true_positives + self.false_positives + self.false_negatives + self.true_negatives
    }

    fn precision(&self) -> f64 {
        self.true_positives as f64 / (self.true_positives + self.false_positives) as f64
    }

    fn recall(&self) -> f64 {
        self.true_positives as f64 / (self.true_positives + self.false_negatives) as f64
    }

    fn f1_score(&self) -> f64 {
        2.0 * self.precision() * self.recall() / (self.precision() + self.recall())
    }

    fn accuracy(&self) -> f64 {
        (self.true_positives + self.true_negatives) as f64 / self.total() as f64
    }
}

impl AddAssign for BoundaryCounts {
    fn add_assign(&mut self, other: Self) {
        self.true_positives += other.true_positives;
        self.false_positives += other.false_positives;
        self.false_negatives += other.false_negatives;
        self.true_negatives += other.true_negatives;
    }
}

impl Verse {
    fn read_all(chunks: Vec<Chunk>) -> Vec<Self> {
        let mut verses = Vec::new();
        let mut current_number: Option<String> = None;
        let mut marked_text = String::new();
        for chunk in chunks {
            if chunk.tag == "v" {
                verses.extend(Self::finished(current_number.take(), &marked_text));
                current_number = chunk.value;
                marked_text.clear();
            } else if matches!(chunk.tag.as_str(), "c" | "wj" | "q" | "q1" | "q2" | "qc")
                && let Some(text) = chunk.value
            {
                marked_text.push_str(&text);
            }
        }
        verses.extend(Self::finished(current_number, &marked_text));
        verses
    }

    /// A verse from its collected text; `None` before the first verse marker or when empty.
    fn finished(number: Option<String>, marked_text: &str) -> Option<Self> {
        let marked_text = marked_text.trim();
        if marked_text.is_empty() {
            return None;
        }
        Some(Self {
            number: number?,
            marked_text: marked_text.to_owned(),
        })
    }

    fn reference_breaks(&self) -> BTreeSet<usize> {
        let mut breaks = BTreeSet::new();
        let mut offset = 0;
        for character in self.marked_text.chars() {
            if character == ZERO_WIDTH_SPACE {
                breaks.insert(offset);
            } else {
                offset += character.len_utf8();
            }
        }
        breaks
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let path = PathBuf::from(
        env::args()
            .nth(1)
            .ok_or("missing chapter JSON path argument")?,
    );
    let path_display = path.display();
    let segmenter = WordSegmenter::new()?;
    let content =
        fs::read_to_string(&path).map_err(|error| format!("read {path_display}: {error}"))?;
    let chunks: Vec<Chunk> = serde_json::from_str(&content)
        .map_err(|error| format!("parse chapter JSON {path_display}: {error}"))?;
    let verses = Verse::read_all(chunks);

    let mut counts = BoundaryCounts::default();
    let mut exact_matches = 0;
    println!("--- SAMPLE VERSES ({path_display}) ---");
    for (index, verse) in verses.iter().enumerate() {
        let clean_text = verse.marked_text.replace(ZERO_WIDTH_SPACE, "");
        let words = segmenter.split(&clean_text)?;
        assert_eq!(words.concat(), clean_text);

        let mut predicted_breaks = BTreeSet::new();
        let mut predicted_offset = 0;
        for word in &words[..words.len().saturating_sub(1)] {
            predicted_offset += word.len();
            predicted_breaks.insert(predicted_offset);
        }

        // Only gaps between two Thai letters are judged; punctuation and other scripts are not.
        let is_thai_boundary_character = |character: char| {
            ('\u{0e01}'..='\u{0e4e}').contains(&character) && character != 'ฯ' && character != 'ๆ'
        };
        let characters: Vec<(usize, char)> = clean_text.char_indices().collect();
        let eligible: BTreeSet<usize> = characters
            .windows(2)
            .filter(|pair| {
                is_thai_boundary_character(pair[0].1) && is_thai_boundary_character(pair[1].1)
            })
            .map(|pair| pair[1].0)
            .collect();
        let reference: BTreeSet<_> = verse
            .reference_breaks()
            .intersection(&eligible)
            .copied()
            .collect();
        let predicted: BTreeSet<_> = predicted_breaks.intersection(&eligible).copied().collect();
        let true_positives = reference.intersection(&predicted).count();
        let false_positives = predicted.difference(&reference).count();
        let false_negatives = reference.difference(&predicted).count();
        counts += BoundaryCounts {
            true_positives,
            false_positives,
            false_negatives,
            true_negatives: eligible.len() - (true_positives + false_positives + false_negatives),
        };
        if reference == predicted {
            exact_matches += 1;
        }

        if index < PRINTED_VERSES || index + 1 == verses.len() {
            let number = &verse.number;
            let reference = verse.marked_text.replace(ZERO_WIDTH_SPACE, " | ");
            let predicted = words.join(" | ");
            println!("\n[{number}]");
            println!("  Reference : {reference}");
            println!("  Predicted : {predicted}");
        }
    }

    println!("\n================ SUMMARY ================");
    let verse_count = verses.len();
    let exact_percent = (f64::from(exact_matches) / verse_count as f64) * 100.0;
    let BoundaryCounts {
        true_positives,
        false_positives,
        false_negatives,
        true_negatives,
    } = &counts;
    let accuracy = counts.accuracy() * 100.0;
    let precision = counts.precision() * 100.0;
    let recall = counts.recall() * 100.0;
    let f1_score = counts.f1_score() * 100.0;
    println!("Verses evaluated: {verse_count}");
    println!("Exact verse matches: {exact_matches} / {verse_count} ({exact_percent:.1}%)");
    println!("True Positives (TP):  {true_positives}");
    println!("False Positives (FP): {false_positives}");
    println!("False Negatives (FN): {false_negatives}");
    println!("True Negatives (TN):  {true_negatives}");
    println!("Boundary Accuracy:    {accuracy:.2}%");
    println!("Break Precision:      {precision:.2}%");
    println!("Break Recall:         {recall:.2}%");
    println!("Break F1-Score:       {f1_score:.2}%");
    println!("=========================================");
    Ok(())
}
