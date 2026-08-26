//! Public types are `Send + Sync`, `Debug` and, where they are plain data, hashable and ordered.


fn assert_send_sync_debug<T: Send + Sync + std::fmt::Debug>() {}
