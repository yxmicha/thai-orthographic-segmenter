use thai_syllable_segmenter::inference::{
    Activations, InputBlocks, PackedMatrix, ShapeError, run_on_cpu,
};

/// Deterministic values in `[-1, 1)`.
fn values(count: usize, seed: u64) -> Vec<f32> {
    let mut state = seed;
    (0..count)
        .map(|_| {
            state = state
                .wrapping_mul(6_364_136_223_846_793_005)
                .wrapping_add(1_442_695_040_888_963_407);
            #[allow(clippy::cast_precision_loss, reason = "both values fit in 24 bits")]
            let unit = ((state >> 40) as f32) / (1u64 << 23) as f32;
            unit - 1.0
        })
        .collect()
}

#[test]
fn packed_multiplication_matches_a_naive_product_for_every_shape() -> Result<(), ShapeError> {
    // Row counts around the micro-kernel block, and output widths around the panel width.
    for (rows, inputs, outputs) in [
        (1, 3, 2),
        (5, 7, 16),
        (6, 8, 17),
        (7, 5, 33),
        (13, 9, 31),
        (2, 1, 1),
        (12, 64, 48),
    ] {
        let weight = values(inputs * outputs, 1);
        let bias = values(outputs, 2);
        let input = values(rows * inputs, 3);
        let matrix = PackedMatrix::new(&weight, inputs, outputs, bias.clone())?;
        let expected = (0..rows * outputs)
            .map(|cell| {
                let (row, column) = (cell / outputs, cell % outputs);
                let product = (0..inputs)
                    .map(|index| {
                        f64::from(input[row * inputs + index])
                            * f64::from(weight[index * outputs + column])
                    })
                    .sum::<f64>();
                #[allow(
                    clippy::cast_possible_truncation,
                    reason = "the reference rounds to f32"
                )]
                let rounded = (product + f64::from(bias[column])) as f32;
                rounded
            })
            .collect::<Vec<_>>();
        let check = |output: Vec<f32>| {
            assert_eq!(output.len(), expected.len());
            for (value, reference) in output.iter().zip(&expected) {
                assert!(
                    (value - reference).abs() < 1.0e-5,
                    "{rows}x{inputs}x{outputs}: {value} differs from {reference}"
                );
            }
        };
        let multiply = |fused: bool| {
            let mut blocks = InputBlocks::default();
            let mut output = vec![0.0; rows * outputs];
            let result = if fused {
                matrix.multiply_into::<true>(&input, &mut blocks, &mut output)
            } else {
                matrix.multiply_into::<false>(&input, &mut blocks, &mut output)
            };
            assert!(result.is_ok());
            output
        };
        check(multiply(false));
        check(multiply(true));
        check(run_on_cpu(
            #[inline(always)]
            || multiply(true),
            #[inline(always)]
            || multiply(false),
        ));
    }
    Ok(())
}

#[test]
fn gelu_matches_the_exact_error_function_form() {
    // erf by its Maclaurin series, accurate in f64 for the range checked here.
    let erf = |value: f64| {
        let mut term = value;
        let mut sum = value;
        for n in 1..80 {
            term *= -value * value / f64::from(n);
            sum += term / f64::from(2 * n + 1);
        }
        sum * 2.0 / std::f64::consts::PI.sqrt()
    };
    for step in -500_i16..=500 {
        let x = f32::from(step) / 100.0;
        let expected = f64::from(x) * 0.5 * (1.0 + erf(f64::from(x) / std::f64::consts::SQRT_2));
        let mut value = [x];
        Activations::new(&mut value).gelu();
        assert!(
            (f64::from(value[0]) - expected).abs() < 1.0e-6,
            "gelu({x}) = {} but expected {expected}",
            value[0]
        );
    }
    let mut tails = [-8.0, 8.0];
    Activations::new(&mut tails).gelu();
    assert!(tails[0].abs() < 1.0e-6);
    assert!((tails[1] - 8.0).abs() < 1.0e-5);
}
