/// Lookahead facts that decide whether the second base after a leading vowel joins its cluster.
#[allow(
    clippy::struct_excessive_bools,
    reason = "independent facts of one scan, each tested on its own"
)]
struct SecondBaseEvidence {
    inherent_final_cluster_has_following_onset: bool,
    cluster_has_following_nucleus: bool,
    cluster_has_terminal_implicit_candidate: bool,
    cluster_starts_unrelated_i_rime: bool,
    starts_leading_ao_pattern: bool,
}
