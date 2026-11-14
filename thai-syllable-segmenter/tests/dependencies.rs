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

/// Whether `path` lies below `directory`. Both are canonicalized, because on Windows
/// `canonicalize` returns `\\?\`-prefixed paths while `cargo metadata` does not.
fn is_inside(path: &Path, directory: &Path) -> bool {
    path.canonicalize()
        .is_ok_and(|canonical| canonical.starts_with(directory))
}

#[test]
fn orthographic_segmenter_crates_depend_only_on_each_other()
-> Result<(), Box<dyn std::error::Error>> {
    let crate_directory = Path::new(env!("CARGO_MANIFEST_DIR"));
    let segmenter_directory = crate_directory
        .parent()
        .ok_or("crate directory has no parent")?
        .canonicalize()?;
    let output = Command::new(env!("CARGO"))
        .current_dir(crate_directory)
        .args([
            "metadata",
            "--format-version",
            "1",
            "--no-deps",
            "--offline",
        ])
        .output()?;
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let metadata: Metadata = serde_json::from_slice(&output.stdout)?;
    let members = metadata
        .packages
        .iter()
        .filter(|package| is_inside(&package.manifest_path, &segmenter_directory))
        .collect::<Vec<_>>();
    assert_eq!(
        members.len(),
        5,
        "expected the four segmenter crates and the Python bindings"
    );
    let mut outside = Vec::new();
    for package in members {
        for dependency in &package.dependencies {
            if let Some(path) = &dependency.path
                && !is_inside(path, &segmenter_directory)
            {
                outside.push(format!(
                    "{package} -> {dependency}",
                    package = package.name,
                    dependency = dependency.name
                ));
            }
        }
    }
    assert!(outside.is_empty(), "repository dependencies: {outside:?}");
    Ok(())
}
