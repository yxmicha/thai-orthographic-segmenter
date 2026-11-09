use pyo3::{PyErr, create_exception, exceptions::PyValueError};

create_exception!(
    thai_orthographic_segmenter,
    SegmenterError,
    PyValueError,
    "A segmenter could not load its model or segment the text."
);
