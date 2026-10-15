//! Longest accepted matches with source-syllable alignment.


use std::{collections::BTreeMap, io::BufRead};

#[derive(Default)]
struct Node {
    children: BTreeMap<char, usize>,
    terminal: bool,
}
