//! Reading trained weights from a safetensors checkpoint into the engine's plain-data types.
//!
//! A [safetensors](https://github.com/huggingface/safetensors) file stores named tensors.
//! [`Checkpoint`] reads one without a deep-learning framework and hands out validated weights
//! by name, so a model loader asks for exactly the tensors it expects and
//! [`Checkpoint::finish`] reports any it left unread.
//!
//! Tensors may be stored as 32-bit floats (`F32`) or as 16-bit floats (`F16`) to halve the
//! file. A 16-bit matrix may also be stored as byte planes, which compress better because the
//! high bytes hold the sign and exponent: the file's `f16_layout` metadata is [`F16_PLANES`],
//! and the tensor is a `U8` tensor of shape `[2, ...shape]` whose first row holds the high
//! bytes of every value and whose second row holds the low bytes.
//!
//! A matrix may instead be quantized to 8 bits: an `I8` tensor of shape `[rows, columns]`
//! next to an `F32` tensor `{name}.scale` of shape `[1, columns]` (one scale per column) or
//! `[rows, 1]` (one scale per row). Each value is the integer times its scale. The
//! `convert-checkpoint` example of this crate writes all of these forms.

use std::collections::BTreeMap;

use safetensors::{Dtype, SafeTensors};

use super::{
    encoder_layer::EncoderLayer,
    packed_matrix::ShapeError,
    weights::{
        AttentionWeights, EmbeddingTable, FeedForwardWeights, LayerNormWeights, WeightMatrix,
    },
};

/// Value of the `f16_layout` metadata key that marks 16-bit matrices stored as byte planes.
pub const F16_PLANES: &str = "planes";

/// Metadata key naming the layout of 16-bit tensors.
pub const F16_LAYOUT_KEY: &str = "f16_layout";

/// Failures reading a checkpoint or asking it for weights.
///
/// # Examples
///
/// ```
/// use thai_syllable_segmenter::inference::{Checkpoint, CheckpointError};
///
/// // Bytes that are not a safetensors file are rejected.
/// assert!(matches!(Checkpoint::from_bytes(b"not a checkpoint"), Err(CheckpointError::Read(_))));
/// ```
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum CheckpointError {
    /// The bytes are not a readable safetensors file.
    #[error("read safetensors checkpoint")]
    Read(#[source] Box<dyn std::error::Error + Send + Sync>),
    /// A tensor is stored as neither 32-bit nor 16-bit floats.
    #[error("checkpoint tensor {name:?} is not 32-bit or 16-bit floating point")]
    UnsupportedType {
        /// Name of the tensor.
        name: String,
    },
    /// A requested tensor is not in the checkpoint.
    #[error("checkpoint has no tensor {0:?}")]
    Missing(String),
    /// A tensor has a different shape from the one requested.
    #[error("checkpoint tensor {name:?} has shape {found:?}, expected {expected:?}")]
    Shape {
        /// Name of the tensor.
        name: String,
        /// Shape stored in the checkpoint.
        found: Vec<usize>,
        /// Shape the caller asked for.
        expected: Vec<usize>,
    },
    /// An 8-bit tensor lacks a usable `{name}.scale` tensor.
    #[error(
        "int8 checkpoint tensor {name:?} needs an f32 `{name}.scale` of shape [1, columns] or [rows, 1]"
    )]
    Scale {
        /// Name of the quantized tensor.
        name: String,
    },
    /// A tensor given to [`Checkpoint::from_tensors`] holds a different number of values than
    /// its shape implies.
    #[error("checkpoint tensor {name:?} has {found} values for shape {shape:?}")]
    Length {
        /// Name of the tensor.
        name: String,
        /// Number of values supplied.
        found: usize,
        /// Shape supplied.
        shape: Vec<usize>,
    },
    /// Tensors remain that no loader asked for.
    #[error("checkpoint has unused tensors {0:?}")]
    Unused(Vec<String>),
    /// The requested weights do not form a valid layer.
    #[error(transparent)]
    Layer(#[from] ShapeError),
}

/// Named tensors of a trained network, read from a safetensors file.
///
/// Each loader method removes the tensors it reads, so data moves out without a copy and
/// [`Checkpoint::finish`] can report what was never used.
///
/// # Examples
///
/// ```
/// use thai_syllable_segmenter::inference::Checkpoint;
///
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// # let bytes = std::fs::read(concat!(env!("CARGO_MANIFEST_DIR"), "/model/weights.safetensors"))?;
/// let mut checkpoint = Checkpoint::from_bytes(&bytes)?;
/// // The bundled syllable model has a 72-row, 16-wide character embedding.
/// let characters = checkpoint.embedding("characters", 72, 16)?;
/// assert_eq!(characters.values().len(), 72 * 16);
/// # Ok(())
/// # }
/// ```
pub struct Checkpoint {
    tensors: BTreeMap<String, (Vec<usize>, Vec<f32>)>,
}

impl std::fmt::Debug for Checkpoint {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("Checkpoint")
            .field("tensors", &self.tensors.len())
            .finish_non_exhaustive()
    }
}

/// Widens an IEEE 754 binary16 value, exactly.
fn half_to_single(bits: u16) -> f32 {
    let sign = u32::from(bits >> 15) << 31;
    let exponent = u32::from((bits >> 10) & 0x1f);
    let mantissa = u32::from(bits & 0x3ff);
    let widened = match (exponent, mantissa) {
        (0, 0) => sign,
        // A subnormal half is a normal single: shift the mantissa up to its leading one.
        (0, _) => {
            let shift = mantissa.leading_zeros() - 21;
            let mantissa = (mantissa << shift) & 0x3ff;
            sign | ((113 - shift) << 23) | (mantissa << 13)
        }
        (0x1f, 0) => sign | 0x7f80_0000,
        (0x1f, _) => sign | 0x7fc0_0000 | (mantissa << 13),
        _ => sign | ((exponent + 112) << 23) | (mantissa << 13),
    };
    f32::from_bits(widened)
}

/// Multiplies each 8-bit value of a `[rows, columns]` matrix by its column's or row's scale.
fn dequantize(
    name: &str,
    shape: &[usize],
    data: &[u8],
    scale: Option<&safetensors::tensor::TensorView<'_>>,
) -> Result<Vec<f32>, CheckpointError> {
    let invalid = || CheckpointError::Scale {
        name: name.to_owned(),
    };
    let [rows, columns] = *shape else {
        return Err(invalid());
    };
    let scale = scale
        .filter(|scale| scale.dtype() == Dtype::F32)
        .ok_or_else(invalid)?;
    let scales = scale
        .data()
        .as_chunks::<4>()
        .0
        .iter()
        .map(|chunk| f32::from_le_bytes(*chunk))
        .collect::<Vec<_>>();
    let per_column = match scale.shape() {
        [1, count] if *count == columns => true,
        [count, 1] if *count == rows => false,
        _ => return Err(invalid()),
    };
    Ok(data
        .iter()
        .enumerate()
        .map(|(index, value)| {
            let scale = if per_column {
                scales[index % columns]
            } else {
                scales[index / columns]
            };
            f32::from(i8::from_le_bytes([*value])) * scale
        })
        .collect())
}

fn read_error(error: impl std::error::Error + Send + Sync + 'static) -> CheckpointError {
    CheckpointError::Read(Box::new(error))
}

impl Checkpoint {
    /// Reads every tensor of a safetensors file, widening 16-bit floats to 32 bits.
    ///
    /// # Errors
    ///
    /// Returns [`CheckpointError::Read`] when `bytes` are not a valid safetensors file, or
    /// [`CheckpointError::UnsupportedType`] when a tensor is neither a 32-bit float, a 16-bit
    /// float, nor a byte-plane pair of the file's [`F16_PLANES`] layout.
    ///
    /// # Examples
    ///
    /// ```
    /// use thai_syllable_segmenter::inference::Checkpoint;
    ///
    /// assert!(Checkpoint::from_bytes(b"").is_err());
    /// ```
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, CheckpointError> {
        let (_, header) = SafeTensors::read_metadata(bytes).map_err(read_error)?;
        let planes = header
            .metadata()
            .as_ref()
            .and_then(|metadata| metadata.get(F16_LAYOUT_KEY))
            .is_some_and(|layout| layout == F16_PLANES);
        let file = SafeTensors::deserialize(bytes).map_err(read_error)?;
        let all = file.tensors().into_iter().collect::<BTreeMap<_, _>>();
        let mut tensors = BTreeMap::new();
        for (name, tensor) in &all {
            let data = tensor.data();
            let mut shape = tensor.shape().to_vec();
            let values = match tensor.dtype() {
                Dtype::F32 => {
                    if let Some(base) = name.strip_suffix(".scale")
                        && all.get(base).is_some_and(|base| base.dtype() == Dtype::I8)
                    {
                        continue; // consumed by the quantized tensor it scales
                    }
                    data.as_chunks::<4>()
                        .0
                        .iter()
                        .map(|chunk| f32::from_le_bytes(*chunk))
                        .collect()
                }
                Dtype::F16 => data
                    .as_chunks::<2>()
                    .0
                    .iter()
                    .map(|chunk| half_to_single(u16::from_le_bytes(*chunk)))
                    .collect(),
                Dtype::U8 if planes && shape.first() == Some(&2) && shape.len() > 1 => {
                    shape.remove(0);
                    let (high, low) = data.split_at(data.len() / 2);
                    high.iter()
                        .zip(low)
                        .map(|(high, low)| half_to_single(u16::from_be_bytes([*high, *low])))
                        .collect()
                }
                Dtype::I8 => dequantize(name, &shape, data, all.get(&format!("{name}.scale")))?,
                _ => {
                    return Err(CheckpointError::UnsupportedType { name: name.clone() });
                }
            };
            tensors.insert(name.clone(), (shape, values));
        }
        Ok(Self { tensors })
    }

    fn take(&mut self, name: &str, expected: &[usize]) -> Result<Vec<f32>, CheckpointError> {
        let (found, _) = self
            .tensors
            .get(name)
            .ok_or_else(|| CheckpointError::Missing(name.to_owned()))?;
        if found != expected {
            return Err(CheckpointError::Shape {
                name: name.to_owned(),
                found: found.clone(),
                expected: expected.to_vec(),
            });
        }
        self.tensors
            .remove(name)
            .map(|(_, values)| values)
            .ok_or_else(|| CheckpointError::Missing(name.to_owned()))
    }

    /// Takes the embedding table `{name}.weight` of `rows` rows of `width` values.
    ///
    /// # Errors
    ///
    /// Returns [`CheckpointError::Missing`] or [`CheckpointError::Shape`] when the tensor is
    /// absent or differs from `[rows, width]`.
    ///
    /// # Examples
    ///
    /// ```
    /// use thai_syllable_segmenter::inference::{Checkpoint, CheckpointError};
    ///
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// # let bytes = std::fs::read(concat!(env!("CARGO_MANIFEST_DIR"), "/model/weights.safetensors"))?;
    /// let mut checkpoint = Checkpoint::from_bytes(&bytes)?;
    /// // Asking for the wrong shape is an error, and the tensor stays unread.
    /// assert!(matches!(checkpoint.embedding("kinds", 3, 16), Err(CheckpointError::Shape { .. })));
    /// assert_eq!(checkpoint.embedding("kinds", 2, 16)?.values().len(), 32);
    /// # Ok(())
    /// # }
    /// ```
    pub fn embedding(
        &mut self,
        name: &str,
        rows: usize,
        width: usize,
    ) -> Result<EmbeddingTable, CheckpointError> {
        Ok(EmbeddingTable::from(
            self.take(&format!("{name}.weight"), &[rows, width])?,
        ))
    }

    /// Takes the linear layer `{name}.weight`, shaped `[inputs, outputs]`, with `{name}.bias`.
    ///
    /// # Errors
    ///
    /// Returns [`CheckpointError::Missing`] or [`CheckpointError::Shape`] when a tensor is
    /// absent or has the wrong shape.
    ///
    /// # Examples
    ///
    /// ```
    /// use thai_syllable_segmenter::inference::Checkpoint;
    ///
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// # let bytes = std::fs::read(concat!(env!("CARGO_MANIFEST_DIR"), "/model/weights.safetensors"))?;
    /// let mut checkpoint = Checkpoint::from_bytes(&bytes)?;
    /// let projection = checkpoint.linear("projection", 240, 64)?;
    /// assert_eq!((projection.inputs(), projection.outputs()), (240, 64));
    /// # Ok(())
    /// # }
    /// ```
    pub fn linear(
        &mut self,
        name: &str,
        inputs: usize,
        outputs: usize,
    ) -> Result<WeightMatrix, CheckpointError> {
        let weight = self.take(&format!("{name}.weight"), &[inputs, outputs])?;
        let bias = self.take(&format!("{name}.bias"), &[outputs])?;
        Ok(WeightMatrix::new(weight, inputs, outputs, bias)?)
    }

    /// Takes the layer norm `{name}.gamma` and `{name}.beta` of `width` features.
    ///
    /// # Errors
    ///
    /// Returns [`CheckpointError::Missing`] or [`CheckpointError::Shape`] when a tensor is
    /// absent or is not `[width]`.
    ///
    /// # Examples
    ///
    /// ```
    /// use thai_syllable_segmenter::inference::Checkpoint;
    ///
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// # let bytes = std::fs::read(concat!(env!("CARGO_MANIFEST_DIR"), "/model/weights.safetensors"))?;
    /// let mut checkpoint = Checkpoint::from_bytes(&bytes)?;
    /// let norm = checkpoint.layer_norm("encoder.layers.0.norm_1", 64)?;
    /// assert_eq!(norm.width(), 64);
    /// # Ok(())
    /// # }
    /// ```
    pub fn layer_norm(
        &mut self,
        name: &str,
        width: usize,
    ) -> Result<LayerNormWeights, CheckpointError> {
        let gamma = self.take(&format!("{name}.gamma"), &[width])?;
        let beta = self.take(&format!("{name}.beta"), &[width])?;
        Ok(LayerNormWeights::new(gamma, beta)?)
    }

    /// Takes one pre-norm encoder layer stored under `name`: `mha.{query,key,value,output}`,
    /// `norm_2` before attention, `norm_1` before the feed-forward network, and
    /// `pwff.linear_inner` and `pwff.linear_outer`.
    ///
    /// # Errors
    ///
    /// Returns the errors of [`Self::linear`] and [`Self::layer_norm`] for the layer's tensors,
    /// or [`CheckpointError::Layer`] when `heads` does not divide `model_dimension`.
    ///
    /// # Examples
    ///
    /// ```
    /// use thai_syllable_segmenter::inference::Checkpoint;
    ///
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// # let bytes = std::fs::read(concat!(env!("CARGO_MANIFEST_DIR"), "/model/weights.safetensors"))?;
    /// let mut checkpoint = Checkpoint::from_bytes(&bytes)?;
    /// let layer = checkpoint.encoder_layer("encoder.layers.0", 64, 128, 4)?;
    /// assert_eq!(layer.feed_forward_dimension(), 128);
    /// # Ok(())
    /// # }
    /// ```
    pub fn encoder_layer(
        &mut self,
        name: &str,
        model_dimension: usize,
        feed_forward_dimension: usize,
        heads: usize,
    ) -> Result<EncoderLayer, CheckpointError> {
        let width = model_dimension;
        let attention = AttentionWeights::new(
            self.layer_norm(&format!("{name}.norm_2"), width)?,
            self.linear(&format!("{name}.mha.query"), width, width)?,
            self.linear(&format!("{name}.mha.key"), width, width)?,
            self.linear(&format!("{name}.mha.value"), width, width)?,
            self.linear(&format!("{name}.mha.output"), width, width)?,
            heads,
        )?;
        let feed_forward = FeedForwardWeights::new(
            self.layer_norm(&format!("{name}.norm_1"), width)?,
            self.linear(
                &format!("{name}.pwff.linear_inner"),
                width,
                feed_forward_dimension,
            )?,
            self.linear(
                &format!("{name}.pwff.linear_outer"),
                feed_forward_dimension,
                width,
            )?,
        )?;
        Ok(EncoderLayer::new(attention, feed_forward)?)
    }
}
