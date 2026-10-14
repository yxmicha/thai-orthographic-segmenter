//! Persistent length-framed adapter for source-preserving dataset preprocessing.
use std::io::{self, BufRead, Read, Write};
use thai_syllable_segmenter::SyllableSegmenter;

const MAX_FRAME_BYTES: usize = 1_048_576;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let segmenter = SyllableSegmenter::new()?;
    let mut input = io::stdin().lock();
    let mut output = io::stdout().lock();
    let mut header = String::new();
    loop {
        header.clear();
        if input.read_line(&mut header)? == 0 {
            break;
        }
        let length: usize = header.trim().parse()?;
        if length > MAX_FRAME_BYTES {
            return Err(format!("input frame exceeds {MAX_FRAME_BYTES} bytes").into());
        }
        let mut bytes = vec![0; length];
        input.read_exact(&mut bytes)?;
        let text = std::str::from_utf8(&bytes)?;
        let mut offset = 0;
        for syllable in segmenter.split(text)? {
            offset += syllable.chars().count();
            write!(output, "{offset} ")?;
        }
        writeln!(output)?;
        output.flush()?;
    }
    Ok(())
}
