//! Activation, softmax and reduction math written so the compiler vectorizes it.
#![allow(
    clippy::inline_always,
    reason = "runtime CPU-feature dispatch only works when every callee is inlined into the `#[target_feature]` function"
)]

/// Interleaved partial sums so reductions vectorize.
pub(super) const LANES: usize = 8;

/// Values to reduce to one number.
///
/// # Examples
///
/// ```
/// use thai_syllable_segmenter::inference::Values;
///
/// assert_eq!(Values::new(&[1.0, 2.0, 3.0]).sum(), 6.0);
/// ```
#[derive(Clone, Copy, Debug)]
pub struct Values<'a>(pub(crate) &'a [f32]);

/// Values to transform in place with the Gaussian error linear unit.
///
/// # Examples
///
/// ```
/// use thai_syllable_segmenter::inference::Activations;
///
/// let mut values = [0.0, 8.0];
/// Activations::new(&mut values).gelu();
/// assert_eq!(values[0], 0.0);
/// assert!((values[1] - 8.0).abs() < 1e-5);
/// ```
#[derive(Debug)]
pub struct Activations<'a>(pub(crate) &'a mut [f32]);

/// One row of attention scores, before and during its softmax.
///
/// # Examples
///
/// ```
/// use thai_syllable_segmenter::inference::AttentionRow;
///
/// let mut scores = [1.0, 3.0];
/// let mut row = AttentionRow::new(&mut scores);
/// let maximum = row.scale_and_maximum(1.0);
/// let total = row.exponentiate(maximum);
/// assert_eq!(maximum, 3.0);
/// assert!((total - (1.0 + (-2.0_f32).exp())).abs() < 1e-5);
/// ```
#[derive(Debug)]
pub struct AttentionRow<'a>(pub(crate) &'a mut [f32]);

impl<'a> Values<'a> {
    /// Views `values` for reduction.
    ///
    /// # Examples
    ///
    /// ```
    /// use thai_syllable_segmenter::inference::Values;
    ///
    /// assert_eq!(Values::new(&[2.0]).sum(), 2.0);
    /// ```
    #[must_use]
    pub const fn new(values: &'a [f32]) -> Self {
        Self(values)
    }
}

impl<'a> Activations<'a> {
    /// Views `values` for an in-place activation.
    ///
    /// # Examples
    ///
    /// ```
    /// use thai_syllable_segmenter::inference::Activations;
    ///
    /// let mut values = [0.0];
    /// Activations::new(&mut values).gelu();
    /// assert_eq!(values, [0.0]);
    /// ```
    #[must_use]
    pub const fn new(values: &'a mut [f32]) -> Self {
        Self(values)
    }
}

impl<'a> AttentionRow<'a> {
    /// Views `scores` as one attention row.
    ///
    /// # Examples
    ///
    /// ```
    /// use thai_syllable_segmenter::inference::AttentionRow;
    ///
    /// let mut scores = [0.5];
    /// assert_eq!(AttentionRow::new(&mut scores).scale_and_maximum(0.5), 1.0);
    /// ```
    #[must_use]
    pub const fn new(scores: &'a mut [f32]) -> Self {
        Self(scores)
    }
}

impl Values<'_> {
    /// Sum over `LANES` interleaved accumulators.
    ///
    /// # Examples
    ///
    /// ```
    /// use thai_syllable_segmenter::inference::Values;
    ///
    /// assert_eq!(Values::new(&[1.0, 2.0, 3.0, 4.0]).sum(), 10.0);
    /// ```
    #[inline(always)]
    #[must_use]
    pub fn sum(&self) -> f32 {
        let mut lanes = [0.0_f32; LANES];
        let (chunks, remainder) = self.0.as_chunks::<LANES>();
        for chunk in chunks {
            for (lane, value) in lanes.iter_mut().zip(chunk) {
                *lane += value;
            }
        }
        lanes.iter().sum::<f32>() + remainder.iter().sum::<f32>()
    }

    /// Sum of squared differences from `mean` over `LANES` interleaved accumulators.
    ///
    /// # Examples
    ///
    /// ```
    /// use thai_syllable_segmenter::inference::Values;
    ///
    /// let values = Values::new(&[1.0, 3.0]);
    /// assert_eq!(values.squared_deviations(2.0), 2.0);
    /// ```
    #[inline(always)]
    #[must_use]
    pub fn squared_deviations(&self, mean: f32) -> f32 {
        let mut lanes = [0.0_f32; LANES];
        let (chunks, remainder) = self.0.as_chunks::<LANES>();
        for chunk in chunks {
            for (lane, value) in lanes.iter_mut().zip(chunk) {
                *lane += (value - mean) * (value - mean);
            }
        }
        lanes.iter().sum::<f32>()
            + remainder
                .iter()
                .map(|value| (value - mean) * (value - mean))
                .sum::<f32>()
    }
}

impl Activations<'_> {
    /// Exact-form GELU, `x * (1 + erf(x / sqrt 2)) / 2`, applied in place. The error function is
    /// a rational approximation with absolute error below 5e-8.
    ///
    /// # Examples
    ///
    /// ```
    /// use thai_syllable_segmenter::inference::Activations;
    ///
    /// let mut values = [-8.0, 0.0, 8.0];
    /// Activations::new(&mut values).gelu();
    /// assert!(values[0].abs() < 1e-5);
    /// assert_eq!(values[1], 0.0);
    /// assert!((values[2] - 8.0).abs() < 1e-5);
    /// ```
    #[inline(always)]
    pub fn gelu(self) {
        const NUMERATOR: [f32; 7] = [
            1.128_378_8,
            0.204_227_24,
            0.051_149_22,
            0.003_896_387_6,
            0.000_147_478_09,
            -1.996_948_5e-6,
            2.011_880_7e-8,
        ];
        const DENOMINATOR: [f32; 5] =
            [1.0, 0.514_320_7, 0.116_786_49, 0.014_730_7, 0.001_006_631_6];
        for value in self.0 {
            let clamped = (*value / std::f32::consts::SQRT_2).clamp(-3.9, 3.9);
            let square = clamped * clamped;
            let numerator = NUMERATOR
                .iter()
                .rev()
                .fold(0.0, |sum, term| sum * square + term);
            let denominator = DENOMINATOR
                .iter()
                .rev()
                .fold(0.0, |sum, term| sum * square + term);
            let error_function = clamped * numerator / denominator;
            *value = *value * (1.0 + error_function) / 2.0;
        }
    }
}

impl AttentionRow<'_> {
    /// Divides every score by `scale` and returns the largest, over `LANES` interleaved
    /// accumulators; negative infinity when the row is empty.
    ///
    /// # Examples
    ///
    /// ```
    /// use thai_syllable_segmenter::inference::AttentionRow;
    ///
    /// let mut scores = [2.0, 6.0, 4.0];
    /// let maximum = AttentionRow::new(&mut scores).scale_and_maximum(2.0);
    /// assert_eq!(maximum, 3.0);
    /// assert_eq!(scores, [1.0, 3.0, 2.0]);
    /// ```
    #[inline(always)]
    pub fn scale_and_maximum(&mut self, scale: f32) -> f32 {
        let mut lanes = [f32::NEG_INFINITY; LANES];
        let (chunks, remainder) = self.0.as_chunks_mut::<LANES>();
        for chunk in chunks {
            for (lane, value) in lanes.iter_mut().zip(chunk) {
                *value /= scale;
                *lane = lane.max(*value);
            }
        }
        for value in remainder.iter_mut() {
            *value /= scale;
        }
        remainder
            .iter()
            .chain(&lanes)
            .fold(f32::NEG_INFINITY, |largest, value| largest.max(*value))
    }

    /// Replaces every score `v` with `exp(v - maximum)` and returns their sum, over `LANES`
    /// interleaved accumulators.
    ///
    /// # Examples
    ///
    /// ```
    /// use thai_syllable_segmenter::inference::AttentionRow;
    ///
    /// let mut scores = [0.0, 0.0];
    /// let total = AttentionRow::new(&mut scores).exponentiate(0.0);
    /// assert!((total - 2.0).abs() < 1e-5);
    /// ```
    #[inline(always)]
    pub fn exponentiate(&mut self, maximum: f32) -> f32 {
        let mut lanes = [0.0_f32; LANES];
        let (chunks, remainder) = self.0.as_chunks_mut::<LANES>();
        for chunk in chunks {
            for (lane, value) in lanes.iter_mut().zip(chunk) {
                *value = exp_not_above_zero(*value - maximum);
                *lane += *value;
            }
        }
        let mut tail = 0.0;
        for value in remainder.iter_mut() {
            *value = exp_not_above_zero(*value - maximum);
            tail += *value;
        }
        lanes.iter().sum::<f32>() + tail
    }
}

/// `exp` for non-positive arguments (softmax after subtracting the maximum) with relative
/// error near 1e-7; arguments below -87 underflow to zero-like values.
#[inline(always)]
fn exp_not_above_zero(value: f32) -> f32 {
    const ROUNDING: f32 = 12_582_912.0;
    const COEFFICIENTS: [f32; 8] = [
        1.0,
        1.0,
        0.5,
        1.0 / 6.0,
        1.0 / 24.0,
        1.0 / 120.0,
        1.0 / 720.0,
        1.0 / 5040.0,
    ];
    let clamped = value.clamp(-87.0, 0.0);
    let shifted = clamped * std::f32::consts::LOG2_E + ROUNDING;
    let exponent = shifted - ROUNDING;
    let remainder = clamped - exponent * 0.693_359_4 + exponent * 2.121_944_4e-4;
    let polynomial = COEFFICIENTS
        .iter()
        .rev()
        .fold(0.0, |sum, term| sum * remainder + term);
    let power_of_two = shifted
        .to_bits()
        .wrapping_sub(0x4B40_0000)
        .wrapping_add(127)
        << 23;
    polynomial * f32::from_bits(power_of_two)
}
