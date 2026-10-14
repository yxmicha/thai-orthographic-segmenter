# thai-word-segmenter

Source-preserving Thai word segmentation. Orthographic syllables from
[`thai-syllable-segmenter`](https://docs.rs/thai-syllable-segmenter) are grouped into lexical
clusters (words, compounds, names, productive bound forms) by a contextual Transformer and a
dictionary of protected multi-syllable words. It runs on the CPU with bundled weights
(epoch 6 of the training run).

## Installation

```sh
cargo add thai-word-segmenter
```

The minimum supported Rust version is 1.95.

## Usage

```rust
use thai_word_segmenter::WordSegmenter;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let segmenter = WordSegmenter::new()?;
    let words = segmenter.split("ในปฐมกาลพระวาทะทรงดำรงอยู่")?;
    assert_eq!(words, ["ในปฐมกาล", "พระวาทะ", "ทรงดำรง", "อยู่"]);
    Ok(())
}
```

`split` returns borrowed slices whose concatenation is the input; `ranges` returns the same
segments as byte ranges.

- `split_many(&texts)` and `ranges_many(&texts)` give the same result as calling `split` on each
  text, but score all the texts' Thai runs together, which is roughly twice as fast for short
  texts. Pass bounded batches, since memory grows with their total length.
- `split_many_on(&texts, threads)` and `ranges_many_on` do the same on up to `threads` worker
  threads that share the segmenter, giving the identical result.
- `WordSegmenter::with_dictionary(text)` swaps in a newline-delimited dictionary of Thai
  words but keeps the bundled model.
- `WordSegmenter::load(directory)` loads a trained artifact directory (`config.json`,
  `dictionary.txt`, `model.safetensors` as 32-bit floats or in the compact 16-bit or 8-bit forms).
- Failures are `thai_word_segmenter::Error`. A Thai run over 4,096 scalars fails with
  `Error::Syllables(ThaiRunTooLong)` because the syllable segmenter rejects it.

## Cargo features

| Feature | Effect |
| --- | --- |
| `burn` | Adds the `network` module (the Burn network that trains the model), `encoding::EncodedBatch` and `InferenceModel::new`. Their signatures use Burn types, so they follow Burn's stability, not this crate's. The segmenters themselves read safetensors checkpoints directly and need no Burn. |

## Behaviour

- **Windows**: syllables are scored in windows of 96 owned syllables with 16 syllables of
  context on each side (the model's 128-syllable capacity). Longer runs are processed window
  by window.
- **Decision**: the model scores each gap between syllables as KEEP or BREAK. A gap breaks
  when the BREAK probability is at least 0.5 and the gap is not inside a dictionary match.
- **Dictionary**: 43,276 entries, every one of 2 or 3 syllables (34,382 and 8,894). Matching is
  leftmost-longest and non-overlapping: gaps inside a selected match never break, and
  overlapping entries do not chain into a larger cluster. Everything else is left to the
  model.
- **Punctuation**: closing quotes, brackets and `, . ! ? : ; % ๆ ฯ …` attach to the preceding
  cluster. Opening quotes and brackets attach to the following one. Straight quotes are paired
  in order, and an apostrophe between letters or digits is not a quote. Horizontal spaces next
  to attached punctuation join it. A non-Thai span that begins with closing punctuation
  (`. … ’ ” ) ] }`) is split after it, so `ปฐก.28:12` gives `ปฐก.` and `28:12`.

## Bundled model size

The bundled `model/model.safetensors` stores its large matrices as 8-bit integers with one scale
per column or row (whichever reconstructs the matrix more exactly), which shrinks the training
run's 22 MB of 32-bit weights to 5.7 MB and keeps the packaged crate at 5.4 MiB, well under the
10 MiB crates.io limit.

Rounding costs a little accuracy. On 6,638 lines (30,695 word boundaries) the 8-bit model places
the same boundaries as the 32-bit model except for 23 (0.075%), each on a different line. For
comparison, 16-bit floats changed none, which would make the crate 9.5 MiB, and bfloat16 changed
5. `cargo run --release --example convert-checkpoint` in `thai-syllable-segmenter` converts a
32-bit burnpack checkpoint to either form (`--int8` for this one); check your own text before
shipping a rounded model.

## Training

The model is trained on a reviewed lexical-cluster dataset, on a GPU only.

- **Granularity**: it is trained to under-segment on purpose. Splitting everyday reading
  material into too many pieces hurts readability, so the model keeps collocations and
  compounds together and cuts only where it is confident.
- **Input**: windows of up to 128 syllables. Each syllable gets 48 ordered character slots
  (long forms keep both ends) and one of 16,383 hashed syllable buckets (row zero is padding).
  A syllable is about 3.2 Thai characters on average (3.15 on John 1).
- **Network**: 6 layers, 8 heads, width 256, feed-forward 1,024, about 5.5 million
  parameters. It scores the gap after every syllable.
- **Labels**: cleaned dictionary matches supply weak lexical supervision. The word markers in
  source corpora are not used as labels. Corpora supply occurrence weights and observed
  context.
- **Objective**: two-class (KEEP/BREAK) softmax cross-entropy, weighted per gap by frequency and
  evidence weights and by square-root class balancing. Unknown or disputed gaps and the final
  syllable have zero weight.
- **Training set**: 9,586,683 training and 934,893 validation examples.
- **Checkpoint choice**: epoch 6 had the best selection score, the mean of raw balanced
  accuracy on validation gaps and on held-out complete dictionary forms (0.839). At epoch 6
  raw validation balanced accuracy is 93.0%, or 93.3% with dictionary protection. When last
  checked (epochs 7 to 18), later epochs lowered the training loss without beating it.

These scores measure agreement with model-reviewed labels, not independently reviewed
correctness on natural sentences.

## Example evaluation

`cargo run -p thai-word-segmenter --example compare_bible -- <chapter.json>` compares
segmentation with the zero-width-space word marks of a THA-TCV Bible chapter. It judges only
gaps between two Thai letters. On John 1 (51 verses) it reports precision 97.95%, recall
83.28% and F1 90.02%, with 17 false-positive and 163 false-negative breaks, and 2 verses
matching exactly. The model breaks less often than the reference marks. This is one chapter
under a different word convention, not a benchmark.

