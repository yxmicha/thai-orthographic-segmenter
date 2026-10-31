#![allow(
    clippy::cast_precision_loss,
    reason = "report figures are ratios of counts far below 2^52"
)]
//! Prints construction time and per-layer throughput over a text file with one text per line.
//! The word layer is measured text by text, in batches of 64 texts, and in batches spread over
//! every available core.
//!
//! Usage: `cargo run --release -p thai-word-segmenter --example throughput -- <lines.txt>`

use std::{env, fs, time::Instant};
use thai_syllable_rules::split;
use thai_syllable_segmenter::SyllableSegmenter;
use thai_word_segmenter::WordSegmenter;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let path = env::args()
        .nth(1)
        .ok_or("missing text file path argument")?;
    let text = fs::read_to_string(&path)?;
    let lines = text.lines().collect::<Vec<_>>();
    let characters = lines.iter().map(|line| line.chars().count()).sum::<usize>();
    let line_count = lines.len();
    println!("{line_count} lines, {characters} characters");
    let started = Instant::now();
    let syllables = SyllableSegmenter::new()?;
    let milliseconds = started.elapsed().as_secs_f64() * 1e3;
    println!("SyllableSegmenter::new {milliseconds:.1} ms");
    let started = Instant::now();
    let words = WordSegmenter::new()?;
    let milliseconds = started.elapsed().as_secs_f64() * 1e3;
    println!("WordSegmenter::new     {milliseconds:.1} ms");
    let report = |layer: &str, seconds: f64| {
        let rate = characters as f64 / seconds;
        println!("{layer:<16} {seconds:8.2} s {rate:>10.0} chars/s");
    };
    let started = Instant::now();
    for line in &lines {
        split(line);
    }
    report("rules", started.elapsed().as_secs_f64());
    let started = Instant::now();
    for line in &lines {
        syllables.split(line)?;
    }
    report("syllable", started.elapsed().as_secs_f64());
    let started = Instant::now();
    for line in &lines {
        words.split(line)?;
    }
    report("word", started.elapsed().as_secs_f64());
    let started = Instant::now();
    for batch in lines.chunks(64) {
        words.split_many(batch)?;
    }
    report("word, batches", started.elapsed().as_secs_f64());
    let threads = std::thread::available_parallelism()?;
    let started = Instant::now();
    for batch in lines.chunks(64 * threads.get()) {
        words.split_many_on(batch, threads)?;
    }
    report(
        &format!("word, {threads} threads"),
        started.elapsed().as_secs_f64(),
    );
    Ok(())
}
