# thai-syllable-segmenter

[![crates.io](https://img.shields.io/crates/v/thai-syllable-segmenter.svg)](https://crates.io/crates/thai-syllable-segmenter)
[![docs.rs](https://img.shields.io/docsrs/thai-syllable-segmenter)](https://docs.rs/thai-syllable-segmenter)
[![downloads](https://img.shields.io/crates/d/thai-syllable-segmenter.svg)](https://crates.io/crates/thai-syllable-segmenter)
[![MSRV](https://img.shields.io/crates/msrv/thai-syllable-segmenter.svg)](https://crates.io/crates/thai-syllable-segmenter)
[![CI](https://github.com/yxmicha/thai-orthographic-segmenter/actions/workflows/ci.yml/badge.svg)](https://github.com/yxmicha/thai-orthographic-segmenter/actions/workflows/ci.yml)
[![License](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue.svg)](https://github.com/yxmicha/thai-orthographic-segmenter#license)

Source-preserving Thai syllable segmentation. It combines the candidates from
[`thai-syllable-rules`](https://docs.rs/thai-syllable-rules) with a small contextual Transformer
and picks the best complete segmentation with a dynamic-programming decoder. It runs on the
CPU with bundled weights: no model download, CUDA runtime or lexicon.

## Installation

```sh
cargo add thai-syllable-segmenter
```

## Usage

```rust
use thai_syllable_segmenter::SyllableSegmenter;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let segmenter = SyllableSegmenter::new()?;
    let pieces = segmenter.split("ฉันกินข้าว สวัสดี!")?;
    assert_eq!(pieces, ["ฉัน", "กิน", "ข้าว", " ", "สวัส", "ดี", "!"]);
    Ok(())
}
```

Construct the segmenter once and reuse it. `split` returns borrowed slices whose
concatenation is the input. Empty input gives an empty vector.

## How it works

- Text is cut into maximal runs accepted by `ThaiScalar::is_accepted` from `thai-syllable-rules`. Every
  other run (Latin, digits, whitespace, punctuation, the baht sign) is returned unchanged.
  Each Thai run is segmented independently, with no context across the spans between runs.
- Within a run, a leading vowel is grouped with its following base and dependent marks stay
  with the preceding base. Candidate boundaries fall only between these cells, so a mark is
  never separated from its base.
- Candidates are spans of consecutive cells up to 48 scalars. The rules' parts guide the
  model but do not fix the answer: a candidate may split or merge them. Rule disagreement is
  a soft cost (0.25 for every candidate, 2.0 when the rules do not recognize a complete
  one, 4.0 per extra vowel nucleus, 0.5 for cutting inside a rule anchor), not a constraint.
- A 2-layer, 4-head Transformer encoder (width 64) over character embeddings, overlapping
  character bigrams and trigrams, and rule features scores each candidate's start and end.
  The decoder returns the best-scoring complete path. It returns no confidence score and no
  alternative paths.
- The network runs on this crate's own CPU inference code, public as
  `thai_syllable_segmenter::inference` so that `thai-word-segmenter` can run its Transformer
  on the same primitives: weights packed once for matrix products, pre-norm encoder layers over
  stacked windows, and AVX2 with fused multiply-add chosen at run time when the CPU has them.
  The weights are a safetensors file, which `inference::Checkpoint` reads directly; no
  deep-learning framework is a dependency.
- Failures are `thai_syllable_segmenter::Error`: `ThaiRunTooLong` for a Thai run over 4,096
  scalars (input is never truncated), `CellTooLong` for one written cell over the 48-scalar
  span limit, plus weight, tensor and invariant errors.

## Training

The bundled model was trained by rules-aware structured distillation: one compact student
learns the path distribution of a larger ensemble while keeping supervised training on
annotated orthographic paths. The objective combines edge marginals and complete-path
cross-entropy over the lattice with negative log-likelihood on annotated paths.

The crate ships frozen weights; the training pipeline is not part of it.

## Results on the Han-solo test set

198 Thai-bearing records (of 199) with 3,643 annotated Thai syllable spans:

| Syllable segmenter | Thai span precision | Thai span recall | Thai span F1 | Exact Thai segmentation |
| --- | ---: | ---: | ---: | ---: |
| `thai-syllable-segmenter` | **98.38%** | **98.30%** | **98.34%** | **172 / 198 (86.87%)** |
| [Han-solo](https://github.com/PyThaiNLP/Han-solo) released CRF | 98.00% | 98.19% | 98.09% | 165 / 198 (83.33%) |
| [SSG](https://github.com/ponrawee/ssg) via PyThaiNLP `ssg` | 96.23% | 98.11% | 97.16% | 154 / 198 (77.78%) |
| PyThaiNLP `dict` | 93.96% | 96.49% | 95.21% | 125 / 198 (63.13%) |

- A **Thai span** is a maximal run of Thai characters that neither a syllable boundary nor a
  non-Thai character interrupts. Non-Thai characters are never part of a span, so an
  annotated `จมก.` and a prediction `จมก` + `.` agree.
- **Precision** and **recall** count predicted and annotated spans with identical start and
  end offsets. **F1** is their harmonic mean. **Exact** counts records whose predicted span
  set equals the annotated one.
- Engine versions: PyThaiNLP 5.1.2, SSG 0.0.8, python-crfsuite 0.9.12, and the released
  Han-solo checkpoint.
- The current crate produces output identical to the benchmarked predictions on all 199
  records. The rows match the benchmark's stored predictions and metrics.

Treat these as benchmark agreement, not a generalization estimate. The test set was used in
earlier experiments, the development set was used repeatedly for model selection, and
overlap with the training data was not screened. Annotation conventions differ between
datasets. No speed or memory comparison is claimed.
