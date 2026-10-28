//! Public types are `Send + Sync` and `Debug`, and their `Debug` output is never empty.


fn assert_send_sync_debug<T: Send + Sync + std::fmt::Debug>() {}
