use std::error::Error;

use pyo3::{PyErr, create_exception, exceptions::PyValueError};

create_exception!(
    thai_orthographic_segmenter,
    SegmenterError,
    PyValueError,
    "A segmenter could not load its model or segment the text."
);

/// A failure with the messages of its underlying causes, outermost first.
///
/// The message is a `String` so a failure can cross out of the code that runs without the
/// interpreter lock.
pub(crate) fn describe(error: &dyn Error) -> String {
    let mut message = error.to_string();
    let mut source = error.source();
    while let Some(cause) = source {
        message.push_str(": ");
        message.push_str(&cause.to_string());
        source = cause.source();
    }
    message
}

/// The Python exception for a failure described by [`describe`].
pub(crate) fn failure(message: String) -> PyErr {
    SegmenterError::new_err(message)
}
