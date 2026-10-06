//! Single-precision matrix multiplication against weights packed once at load time.
//!
//! General matrix-multiply libraries repack the weight operand on every call, which costs more
//! than the multiplication when a window has only a handful of syllables. Here the weights are
//! stored as panels of [`PANEL_WIDTH`] output columns, so each call streams them once.
#![allow(
    clippy::inline_always,
    reason = "runtime CPU-feature dispatch only works when every callee is inlined into the `#[target_feature]` function"
)]

use std::sync::{Mutex, PoisonError};

/// Output columns held by one packed panel and one micro-kernel accumulator row.
const PANEL_WIDTH: usize = 16;
/// Cache lines of the next panel requested ahead of use.
const PREFETCH_LINES: usize = 1024;
/// Input rows multiplied against a panel by one micro-kernel invocation.
const BLOCK_ROWS: usize = 6;

/// Operands of a matrix operation whose lengths do not match the matrix shape.
///
/// # Examples
///
/// ```
/// use thai_syllable_segmenter::inference::{PackedMatrix, ShapeError};
///
/// // Three weights cannot form a 2x2 matrix.
/// assert_eq!(PackedMatrix::new(&[1.0, 0.0, 0.0], 2, 2, vec![0.0; 2]).unwrap_err(), ShapeError);
/// ```
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, thiserror::Error)]
#[error("operand lengths do not match the matrix shape")]
pub struct ShapeError;

/// Reusable storage for input rows copied into micro-kernel order by
/// [`PackedMatrix::multiply_into`]. Keeping one per forward pass avoids an allocation per product.
///
/// # Examples
///
/// ```
/// use thai_syllable_segmenter::inference::{InputBlocks, PackedMatrix};
///
/// let matrix = PackedMatrix::new(&[1.0, 0.0, 0.0, 1.0], 2, 2, vec![0.0; 2])?;
/// // One buffer serves every product of a pass.
/// let mut blocks = InputBlocks::default();
/// let mut output = [0.0; 2];
/// matrix.multiply_into::<false>(&[3.0, 4.0], &mut blocks, &mut output)?;
/// matrix.multiply_into::<false>(&[5.0, 6.0], &mut blocks, &mut output)?;
/// assert_eq!(output, [5.0, 6.0]);
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
#[derive(Default)]
pub struct InputBlocks(Vec<f32>);

impl std::fmt::Debug for InputBlocks {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("InputBlocks")
            .field("capacity", &self.0.capacity())
            .finish()
    }
}

/// Input-row storage shared by the forward passes of one model, so each pass starts from buffers
/// that earlier passes already grew instead of allocating and faulting in fresh memory.
///
/// # Examples
///
/// ```
/// use thai_syllable_segmenter::inference::BlockPool;
///
/// let pool = BlockPool::default();
/// let blocks = pool.take();
/// pool.give_back(blocks);
/// ```
#[derive(Default)]
pub struct BlockPool(Mutex<Vec<InputBlocks>>);

impl std::fmt::Debug for BlockPool {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let idle = self.0.lock().unwrap_or_else(PoisonError::into_inner).len();
        formatter
            .debug_struct("BlockPool")
            .field("idle", &idle)
            .finish()
    }
}

impl BlockPool {
    /// Hands out storage left by an earlier pass, or empty storage when none is idle.
    ///
    /// # Examples
    ///
    /// ```
    /// use thai_syllable_segmenter::inference::BlockPool;
    ///
    /// let pool = BlockPool::default();
    /// let blocks = pool.take(); // empty: no earlier pass left storage
    /// pool.give_back(blocks);
    /// let reused = pool.take(); // the storage given back
    /// # drop(reused);
    /// ```
    pub fn take(&self) -> InputBlocks {
        self.0
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .pop()
            .unwrap_or_default()
    }

    /// Returns storage for later passes, possibly on other threads.
    ///
    /// # Examples
    ///
    /// ```
    /// use thai_syllable_segmenter::inference::BlockPool;
    ///
    /// let pool = BlockPool::default();
    /// pool.give_back(pool.take());
    /// assert!(format!("{pool:?}").contains("idle: 1"));
    /// ```
    pub fn give_back(&self, blocks: InputBlocks) {
        self.0
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .push(blocks);
    }
}

/// A `[inputs, outputs]` weight matrix with a bias vector, packed for [`Self::multiply`] and [`Self::multiply_into`].
///
/// # Examples
///
/// ```
/// use thai_syllable_segmenter::inference::{InputBlocks, PackedMatrix};
///
/// // y = x · W + b for a 2x2 identity W and a bias of one.
/// let matrix = PackedMatrix::new(&[1.0, 0.0, 0.0, 1.0], 2, 2, vec![1.0; 2])?;
/// let mut output = [0.0; 2];
/// matrix.multiply_into::<false>(&[3.0, 4.0], &mut InputBlocks::default(), &mut output)?;
/// assert_eq!(output, [4.0, 5.0]);
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
#[derive(Clone)]
pub struct PackedMatrix {
    inputs: usize,
    outputs: usize,
    /// `[panel][input][PANEL_WIDTH]`; columns past `outputs` are zero.
    panels: Vec<f32>,
    /// One value per output, padded with zeros to a whole number of panels so every panel reads
    /// [`PANEL_WIDTH`] values.
    bias: Vec<f32>,
}

/// Multiply-add that fuses rounding only where the CPU has the instruction; without it a fused
/// operation would fall back to a slow software routine.
///
/// # Examples
///
/// ```
/// use thai_syllable_segmenter::inference::multiply_add;
///
/// assert_eq!(multiply_add::<false>(2.0, 3.0, 1.0), 7.0);
/// ```
#[inline(always)]
#[must_use]
pub fn multiply_add<const FMA: bool>(left: f32, right: f32, sum: f32) -> f32 {
    if FMA {
        left.mul_add(right, sum)
    } else {
        left * right + sum
    }
}

impl std::fmt::Debug for PackedMatrix {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("PackedMatrix")
            .field("inputs", &self.inputs)
            .field("outputs", &self.outputs)
            .finish_non_exhaustive()
    }
}

impl PackedMatrix {
    /// Packs a row-major `[inputs, outputs]` weight matrix. Returns [`ShapeError`] unless
    /// `weight` holds `inputs * outputs` values, `bias` one value per output, and both
    /// dimensions are positive.
    ///
    /// # Errors
    ///
    /// Returns [`ShapeError`] when a dimension is zero, or `weight` or `bias` has the wrong
    /// length.
    ///
    /// # Examples
    ///
    /// ```
    /// use thai_syllable_segmenter::inference::PackedMatrix;
    ///
    /// let matrix = PackedMatrix::new(&[1.0, 2.0, 3.0, 4.0, 5.0, 6.0], 2, 3, vec![0.0; 3])?;
    /// assert_eq!((matrix.inputs(), matrix.outputs()), (2, 3));
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn new(
        weight: &[f32],
        inputs: usize,
        outputs: usize,
        bias: Vec<f32>,
    ) -> Result<Self, ShapeError> {
        if inputs == 0 || outputs == 0 || weight.len() != inputs * outputs || bias.len() != outputs
        {
            return Err(ShapeError);
        }
        let panel_count = outputs.div_ceil(PANEL_WIDTH);
        let mut panels = vec![0.0; panel_count * inputs * PANEL_WIDTH];
        for (input, row) in weight.chunks_exact(outputs).enumerate() {
            for (panel, columns) in row.chunks(PANEL_WIDTH).enumerate() {
                let start = (panel * inputs + input) * PANEL_WIDTH;
                if let Some(target) = panels.get_mut(start..start + columns.len()) {
                    target.copy_from_slice(columns);
                }
            }
        }
        let mut bias = bias;
        bias.resize(panel_count * PANEL_WIDTH, 0.0);
        Ok(Self {
            inputs,
            outputs,
            panels,
            bias,
        })
    }

    /// Number of output columns.
    ///
    /// # Examples
    ///
    /// ```
    /// use thai_syllable_segmenter::inference::PackedMatrix;
    ///
    /// let matrix = PackedMatrix::new(&[1.0; 6], 2, 3, vec![0.0; 3])?;
    /// assert_eq!(matrix.outputs(), 3);
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    #[must_use]
    pub fn outputs(&self) -> usize {
        self.outputs
    }

    /// Writes `input · weight + bias` for row-major `input` with `inputs` columns, into `output`
    /// with `outputs` columns.
    ///
    /// The input rows are first copied into `blocks` in groups of up to `BLOCK_ROWS` rows with
    /// the values of one column adjacent, so the micro-kernel reads both operands sequentially.
    /// Returns [`ShapeError`] unless `input` holds a whole number of rows and `output` holds one
    /// output row for each of them.
    ///
    /// # Errors
    ///
    /// Returns [`ShapeError`] when `input` is not a whole number of rows or `output` does not
    /// hold one output row per input row.
    ///
    /// # Examples
    ///
    /// ```
    /// use thai_syllable_segmenter::inference::{InputBlocks, PackedMatrix};
    ///
    /// // Two input rows of two values, multiplied by a 2x1 matrix that adds them.
    /// let matrix = PackedMatrix::new(&[1.0, 1.0], 2, 1, vec![0.0])?;
    /// let mut output = [0.0; 2];
    /// matrix.multiply_into::<false>(&[1.0, 2.0, 3.0, 4.0], &mut InputBlocks::default(), &mut output)?;
    /// assert_eq!(output, [3.0, 7.0]);
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    #[inline(always)]
    pub fn multiply_into<const FMA: bool>(
        &self,
        input: &[f32],
        blocks: &mut InputBlocks,
        output: &mut [f32],
    ) -> Result<(), ShapeError> {
        if self.inputs == 0 || !input.len().is_multiple_of(self.inputs) {
            return Err(ShapeError);
        }
        let rows = input.len() / self.inputs;
        if output.len() != rows * self.outputs {
            return Err(ShapeError);
        }
        let mut packed = std::mem::take(&mut blocks.0);
        packed.clear();
        packed.reserve(input.len());
        let mut row = 0;
        while row < rows {
            let take = (rows - row).min(BLOCK_ROWS);
            let block = &input[row * self.inputs..][..take * self.inputs];
            let start = packed.len();
            packed.resize(start + block.len(), 0.0);
            let target = &mut packed[start..];
            for (offset, source) in block.chunks_exact(self.inputs).enumerate() {
                for (column, value) in source.iter().enumerate() {
                    target[column * take + offset] = *value;
                }
            }
            row += take;
        }
        let panel_floats = self.inputs * PANEL_WIDTH;
        for (panel_index, panel) in self.panels.chunks_exact(panel_floats).enumerate() {
            let next_panel = self.panels.get((panel_index + 1) * panel_floats..);
            let column = panel_index * PANEL_WIDTH;
            let mut row = 0;
            while row < rows {
                let take = (rows - row).min(BLOCK_ROWS);
                Self::prefetch(next_panel, row / BLOCK_ROWS, rows.div_ceil(BLOCK_ROWS));
                let block = &packed[row * self.inputs..][..take * self.inputs];
                match take {
                    6 => self.block::<FMA, 6>(panel, block, output, row, column),
                    5 => self.block::<FMA, 5>(panel, block, output, row, column),
                    4 => self.block::<FMA, 4>(panel, block, output, row, column),
                    3 => self.block::<FMA, 3>(panel, block, output, row, column),
                    2 => self.block::<FMA, 2>(panel, block, output, row, column),
                    _ => self.block::<FMA, 1>(panel, block, output, row, column),
                }
                row += take;
            }
        }
        blocks.0 = packed;
        Ok(())
    }

    /// Asks the CPU to start loading `panel` into the second-level cache while the current
    /// panel is being multiplied, hiding the latency of streaming weights on small inputs. The
    /// lines are spread over the `blocks` row blocks that use the current panel, `block` being
    /// the index of the one running now, so no block issues them all at once.
    #[inline(always)]
    fn prefetch(panel: Option<&[f32]>, block: usize, blocks: usize) {
        #[cfg(target_arch = "x86_64")]
        if let Some(panel) = panel {
            let total = PREFETCH_LINES.min(panel.len().div_ceil(PANEL_WIDTH));
            let per_block = total.div_ceil(blocks);
            let first = block * per_block;
            let Some(rest) = panel.get(first * PANEL_WIDTH..) else {
                return;
            };
            for line in rest
                .chunks(PANEL_WIDTH)
                .take(per_block.min(total.saturating_sub(first)))
            {
                // SAFETY: SSE is part of the x86-64 baseline, and a prefetch of a valid address
                // has no architectural effect.
                #[allow(unsafe_code, reason = "a prefetch hint cannot fault")]
                unsafe {
                    std::arch::x86_64::_mm_prefetch::<{ std::arch::x86_64::_MM_HINT_T1 }>(
                        line.as_ptr().cast::<i8>(),
                    );
                }
            }
        }
    }

    /// Multiplies one packed block of `ROWS` input rows by one weight panel.
    #[inline(always)]
    fn block<const FMA: bool, const ROWS: usize>(
        &self,
        panel: &[f32],
        block: &[f32],
        output: &mut [f32],
        first_row: usize,
        column: usize,
    ) {
        let mut sums = [[0.0_f32; PANEL_WIDTH]; ROWS];
        for (values, weights) in block
            .as_chunks::<ROWS>()
            .0
            .iter()
            .zip(panel.as_chunks::<PANEL_WIDTH>().0)
        {
            for (value, sum) in values.iter().zip(sums.iter_mut()) {
                for (total, weight) in sum.iter_mut().zip(weights) {
                    *total = multiply_add::<FMA>(*value, *weight, *total);
                }
            }
        }
        let width = PANEL_WIDTH.min(self.outputs - column);
        let bias = self
            .bias
            .get(column..)
            .and_then(|rest| rest.first_chunk::<PANEL_WIDTH>())
            .copied()
            .unwrap_or([0.0; PANEL_WIDTH]);
        for (row, sum) in sums.iter().enumerate() {
            let finished: [f32; PANEL_WIDTH] =
                std::array::from_fn(|index| sum[index] + bias[index]);
            let Some(target) = output.get_mut((first_row + row) * self.outputs + column..) else {
                return;
            };
            match target.first_chunk_mut::<PANEL_WIDTH>() {
                Some(full) if width == PANEL_WIDTH => *full = finished,
                _ => target[..width].copy_from_slice(&finished[..width]),
            }
        }
    }
}
