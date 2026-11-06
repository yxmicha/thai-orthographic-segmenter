//! Public types are `Send + Sync` and `Debug`, and their `Debug` output is never empty.

use thai_line_segmenter::{Error, LineSegmenter, icu_dictionary::IcuDictionary};

fn assert_send_sync_debug<T: Send + Sync + std::fmt::Debug>() {}

#[test]
fn public_types_are_thread_safe_and_inspectable() {
    assert_send_sync_debug::<LineSegmenter>();
    assert_send_sync_debug::<IcuDictionary>();
    assert_send_sync_debug::<Error>();
}

#[test]
fn debug_output_is_never_empty() -> Result<(), Error> {
    assert_ne!(format!("{:?}", LineSegmenter::new()?), "");
    Ok(())
}
