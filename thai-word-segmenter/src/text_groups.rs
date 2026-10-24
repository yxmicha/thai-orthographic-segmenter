/// Groups of texts each worker thread claims, per worker, so uneven cores finish together.
const GROUPS_PER_WORKER: usize = 4;
