use pyo3::{exceptions::PyValueError, prelude::*};

/// Texts scored together in one call to the model, which holds activations for all of them.
const BATCH_BYTES: usize = 256 * 1024;
