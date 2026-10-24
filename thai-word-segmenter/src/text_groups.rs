/// Groups of texts each worker thread claims, per worker, so uneven cores finish together.
const GROUPS_PER_WORKER: usize = 4;
/// Smallest group of texts worth scoring together, in bytes of text.
const MINIMUM_GROUP_BYTES: usize = 1024;
