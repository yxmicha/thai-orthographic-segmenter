//! The segmenter crates and their Python bindings depend on no other repository crate.

use std::{
    path::{Path, PathBuf},
    process::Command,
};

#[derive(serde::Deserialize)]
struct Metadata {
    packages: Vec<Package>,
}

#[derive(serde::Deserialize)]
struct Package {
    name: String,
    manifest_path: PathBuf,
    dependencies: Vec<Dependency>,
}

#[derive(serde::Deserialize)]
struct Dependency {
    name: String,
    path: Option<PathBuf>,
}
