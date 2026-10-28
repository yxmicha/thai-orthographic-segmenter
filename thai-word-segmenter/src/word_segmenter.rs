use std::{
    fs::File,
    io::BufReader,
    num::NonZeroUsize,
    ops::Range,
    path::Path,
    sync::atomic::{AtomicUsize, Ordering},
    thread,
};

use thai_syllable_segmenter::SyllableSegmenter;

use crate::{
    Error,
    decoding::{BoundaryScores, SourceText},
    dictionary::ClusterDictionary,
    inference::InferenceModel,
    model::{CONTEXT_SYLLABLES, ClusterModelConfig, OWNED_SYLLABLES},
    text_groups::TextGroups,
    text_runs::TextRuns,
};

const BUNDLED_CONFIG: &str = include_str!("../model/config.json");
const BUNDLED_DICTIONARY: &str = include_str!("../model/dictionary.txt");
const BUNDLED_WEIGHTS: &[u8] = include_bytes!("../model/model.safetensors");

/// Reusable CPU word segmenter: syllables from `thai-syllable-segmenter`, grouped by a contextual
/// cluster model with accepted-dictionary protection and punctuation attachment.
///
/// # Examples
///
/// ```
/// use thai_word_segmenter::WordSegmenter;
///
/// let segmenter = WordSegmenter::new()?;
/// let text = "ในปฐมกาลพระวาทะทรงดำรงอยู่";
/// let words = segmenter.split(text)?;
/// assert_eq!(words.concat(), text);
/// assert_eq!(words[..3], ["ในปฐมกาล", "พระวาทะ", "ทรงดำรง"]);
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
pub struct WordSegmenter {
    syllables: SyllableSegmenter,
    config: ClusterModelConfig,
    model: InferenceModel,
    dictionary: ClusterDictionary,
}

impl std::fmt::Debug for WordSegmenter {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("WordSegmenter")
            .finish_non_exhaustive()
    }
}

impl WordSegmenter {
    /// Loads the bundled epoch-6 model with its 2-to-3-syllable dictionary.
    ///
    /// # Errors
    ///
    /// Returns an [`Error`] when the bundled model cannot be loaded, which indicates a corrupt
    /// build.
    ///
    /// # Examples
    ///
    /// ```
    /// use thai_word_segmenter::WordSegmenter;
    ///
    /// // Building a segmenter takes about 40 ms; build one and reuse it, from any number of threads.
    /// let segmenter = WordSegmenter::new()?;
    /// # drop(segmenter);
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn new() -> Result<Self, Error> {
        Self::with_dictionary(BUNDLED_DICTIONARY)
    }

    /// Loads the bundled epoch-6 model with a custom newline-delimited dictionary.
    ///
    /// # Errors
    ///
    /// Returns [`Error::DictionaryEntry`] when an entry is empty or contains non-Thai characters,
    /// or another [`Error`] when the bundled model cannot be loaded.
    ///
    /// # Examples
    ///
    /// ```
    /// use thai_word_segmenter::{Error, WordSegmenter};
    ///
    /// // The dictionary replaces the bundled one; a dictionary word is never split.
    /// let segmenter = WordSegmenter::with_dictionary("ปฐมกาลพระวาทะ\n")?;
    /// let words = segmenter.split("ในปฐมกาลพระวาทะทรงดำรงอยู่")?;
    /// assert!(words.iter().any(|word| word.contains("ปฐมกาลพระวาทะ")));
    /// assert!(matches!(WordSegmenter::with_dictionary("abc"), Err(Error::DictionaryEntry(_))));
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn with_dictionary(dictionary_text: &str) -> Result<Self, Error> {
        let config: ClusterModelConfig =
            serde_json::from_str(BUNDLED_CONFIG).map_err(Error::ConfigFormat)?;
        let dictionary = ClusterDictionary::read(dictionary_text.as_bytes())?;
        let model = InferenceModel::from_weight_bytes(&config, BUNDLED_WEIGHTS)?;
        Ok(Self {
            syllables: SyllableSegmenter::new()?,
            config,
            model,
            dictionary,
        })
    }

    /// Loads a trained artifact directory containing `config.json`, `dictionary.txt`, and
    /// `model.safetensors`, in 32-bit floats or the compact 16-bit or 8-bit forms that the
    /// `convert-checkpoint` example of `thai-syllable-segmenter` writes from a training run's
    /// burnpack.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Open`] when an artifact file is missing, [`Error::ConfigFormat`] or
    /// [`Error::Config`] for an invalid configuration, [`Error::DictionaryEntry`] for an invalid
    /// dictionary entry, or [`Error::CheckpointLoad`] when the weights cannot be read.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use thai_word_segmenter::WordSegmenter;
    ///
    /// // An artifact directory holds `config.json`, `dictionary.txt` and `model.safetensors`.
    /// let segmenter = WordSegmenter::load("path/to/artifact")?;
    /// let words = segmenter.split("ในปฐมกาล")?;
    /// assert_eq!(words.concat(), "ในปฐมกาล");
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn load(directory: impl AsRef<Path>) -> Result<Self, Error> {
        let directory = directory.as_ref();
        let open = |name: &str| {
            let path = directory.join(name);
            File::open(&path).map_err(|source| Error::Open { path, source })
        };
        let config: ClusterModelConfig =
            serde_json::from_reader(open("config.json")?).map_err(Error::ConfigFormat)?;
        let dictionary = ClusterDictionary::read(BufReader::new(open("dictionary.txt")?))?;
        let weights_path = directory.join("model.safetensors");
        let weights = std::fs::read(&weights_path).map_err(|source| Error::Open {
            path: weights_path,
            source,
        })?;
        let model = InferenceModel::from_weight_bytes(&config, &weights)?;
        Ok(Self {
            syllables: SyllableSegmenter::new()?,
            config,
            model,
            dictionary,
        })
    }

    /// Segments text into words, compounds, bound forms, and punctuation-attached units.
    /// Concatenating the returned slices reconstructs the input text exactly.
    ///
    /// # Errors
    ///
    /// [`Error::Syllables`] when a Thai run is longer than 4,096 scalars, or another [`Error`]
    /// when the model cannot score a window.
    ///
    /// # Examples
    ///
    /// ```
    /// use thai_word_segmenter::WordSegmenter;
    ///
    /// let segmenter = WordSegmenter::new()?;
    /// let text = "ในปฐมกาลพระวาทะทรงดำรงอยู่";
    /// let words = segmenter.split(text)?;
    /// // Concatenating the slices reconstructs the input exactly.
    /// assert_eq!(words.concat(), text);
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn split<'a>(&self, text: &'a str) -> Result<Vec<&'a str>, Error> {
        let ranges = self.ranges(text)?;
        Ok(ranges.into_iter().map(|range| &text[range]).collect())
    }

    /// Returns byte ranges of the [`Self::split`] segments, including preserved whitespace and
    /// non-Thai text.
    ///
    /// # Errors
    ///
    /// [`Error::Syllables`] when a Thai run is longer than 4,096 scalars, or another [`Error`]
    /// when the model cannot score a window.
    ///
    /// # Examples
    ///
    /// ```
    /// use thai_word_segmenter::WordSegmenter;
    ///
    /// let segmenter = WordSegmenter::new()?;
    /// assert_eq!(segmenter.ranges("ในปฐมกาล")?, [0..24]);
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn ranges(&self, text: &str) -> Result<Vec<Range<usize>>, Error> {
        Ok(self
            .ranges_many(&[text])?
            .into_iter()
            .next()
            .unwrap_or_default())
    }

    /// Returns the [`Self::ranges`] of every text, scoring all of their Thai runs together.
    ///
    /// # Errors
    ///
    /// [`Error::Syllables`] when a Thai run is longer than 4,096 scalars, or another [`Error`]
    /// when the model cannot score a window.
    ///
    /// # Examples
    ///
    /// ```
    /// use thai_word_segmenter::WordSegmenter;
    ///
    /// let segmenter = WordSegmenter::new()?;
    /// assert_eq!(segmenter.ranges_many(&["ในปฐมกาล", "พระวาทะ"])?, [vec![0..24], vec![0..21]]);
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn ranges_many(&self, texts: &[&str]) -> Result<Vec<Vec<Range<usize>>>, Error> {
        let layouts = texts
            .iter()
            .map(|text| Ok(TextRuns::new(self.syllables.split(text)?)))
            .collect::<Result<Vec<_>, Error>>()?;
        let thai_runs = layouts
            .iter()
            .flat_map(TextRuns::thai_runs)
            .collect::<Vec<_>>();
        let mut breaks = self.breaks(&thai_runs)?.into_iter();
        texts
            .iter()
            .zip(&layouts)
            .map(|(text, layout)| {
                let ranges = layout.ranges(&mut breaks)?;
                if text.is_empty() {
                    Ok(ranges)
                } else {
                    SourceText::new(text).attach_punctuation(&ranges)
                }
            })
            .collect()
    }

    /// Dictionary-protected BREAK decisions for every syllable of every Thai run.
    fn breaks(&self, runs: &[&[&str]]) -> Result<Vec<Vec<bool>>, Error> {
        let mut spans = Vec::new();
        let mut windows = Vec::new();
        let mut wanted = Vec::new();
        for (run_index, run) in runs.iter().enumerate() {
            let protected = self.dictionary.protected_gaps(run);
            for start in (0..run.len()).step_by(OWNED_SYLLABLES) {
                let left = start.saturating_sub(CONTEXT_SYLLABLES);
                let end = (start + OWNED_SYLLABLES).min(run.len());
                let right = (end + CONTEXT_SYLLABLES).min(run.len());
                spans.push((run_index, start..end, start - left));
                windows.push(
                    run.get(left..right)
                        .ok_or(Error::Ranges("window outside the run"))?,
                );
                // Only owned gaps that may break are read; the last syllable never breaks.
                wanted.push(
                    (left..right)
                        .map(|index| {
                            (start..end).contains(&index)
                                && index + 1 < run.len()
                                && !protected.get(index).copied().unwrap_or(true)
                        })
                        .collect::<Vec<_>>(),
                );
            }
        }
        let differences = self.model.differences_wanted(&windows, &wanted)?;
        let mut probabilities = runs
            .iter()
            .map(|run| vec![0.0; run.len()])
            .collect::<Vec<_>>();
        for ((run_index, owned, skipped), window) in spans.into_iter().zip(&differences) {
            let owned_differences =
                window
                    .get(skipped..skipped + owned.len())
                    .ok_or(Error::Scores(
                        "logits are shorter than the inference window",
                    ))?;
            let owned_probabilities = probabilities
                .get_mut(run_index)
                .and_then(|run| run.get_mut(owned))
                .ok_or(Error::Scores("owned syllables outside their run"))?;
            for (probability, difference) in owned_probabilities.iter_mut().zip(owned_differences) {
                *probability = 1.0 / (1.0 + difference.exp());
            }
        }
        runs.iter()
            .zip(&probabilities)
            .map(|(run, probabilities)| {
                BoundaryScores::new(probabilities, self.config.break_threshold)
                    .select_breaks(run, &self.dictionary)
            })
            .collect()
    }
}
