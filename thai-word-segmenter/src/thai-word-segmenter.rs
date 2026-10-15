#![forbid(unsafe_code)]
#![warn(missing_docs)]
//! Source-preserving Thai word segmentation bundled with a trained contextual model.
//!
//! Orthographic syllables from [`thai_syllable_segmenter`] are grouped into lexical clusters
//! (words, compounds, names and productive bound forms) by a contextual Transformer and a
//! dictionary of protected multi-syllable words. The clusters are slices of the input, so
//! concatenating them gives the input back, and punctuation is attached to its neighbouring
//! cluster.
//!
//! # Examples
//!
//! ```
//! use thai_word_segmenter::WordSegmenter;
//!
//! let segmenter = WordSegmenter::new()?;
//! let text = "ในปฐมกาลพระวาทะทรงดำรงอยู่";
//! let words = segmenter.split(text)?;
//! assert_eq!(words[..3], ["ในปฐมกาล", "พระวาทะ", "ทรงดำรง"]);
//! assert_eq!(words.concat(), text);
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```
//!
//! Score many texts together with [`WordSegmenter::split_many`], or spread them over threads with
//! [`WordSegmenter::split_many_on`], for several times the single-text throughput. Use
//! [`WordSegmenter::with_dictionary`] or [`WordSegmenter::load`] for a custom dictionary or a
//! trained model.
//!
//! # Edge cases, invariants and panics
//!
//! - Empty input gives an empty vector, and text without Thai is split only at punctuation and
//!   script boundaries. A Thai run over 4,096 scalars fails with
//!   [`Error::Syllables`].
//! - A dictionary entry must be non-empty Thai text ([`Error::DictionaryEntry`]), and a model
//!   configuration must satisfy [`model::ClusterModelConfig::validate`]; for example the window
//!   size is fixed at [`model::WINDOW_SYLLABLES`].
//! - [`WordSegmenter::split_many`] and [`WordSegmenter::split_many_on`] return exactly what
//!   splitting each text alone returns,.
//! - Segmentation never panics for any input; property-based tests check this on arbitrary
//!   Unicode, arbitrary partitions and arbitrary scores.
//!
//! # Modules
//!
//! [`WordSegmenter`] is the entry point. The public modules expose the pipeline stages so that a
//! training run can encode, score and decode exactly as inference does: [`encoding`] turns
//! syllables into model input, [`model`] holds the model dimensions and their configuration,
//! [`inference`] is the fast CPU engine that runs a trained checkpoint, and [`decoding`] and
//! [`dictionary`] turn scores into boundaries.
//!
//! # Features
//!
//! - `burn`: adds the `network` module, the Burn network that trains the model, plus
//!   `encoding::EncodedBatch` and `InferenceModel::new`, which convert Burn models. Their
//!   signatures use Burn types, so they follow Burn's stability rather than this crate's. The
//!   segmenters themselves need no Burn: they read safetensors checkpoints directly.
//!
//! # Threads
//!
//! A [`WordSegmenter`] is `Send + Sync`. Building one takes about 40 ms, so build one and share
//! it.

/// Boundary decoding and punctuation attachment.
pub mod decoding;
/// Dictionary protection for known lexical clusters.
pub mod dictionary;

mod error;
