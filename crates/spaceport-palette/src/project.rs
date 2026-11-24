//! A project folder the palette offers, and the one it would create.
//!
//! Only the two values live here: discovery, validation and the folders
//! themselves stay in the app (`crates/spaceport-gpui/src/palette/projects.rs`).

use std::path::PathBuf;

/// A project folder discovered under one of the configured roots.
#[derive(Clone, Debug)]
pub struct Project {
    pub path: PathBuf,
    pub label: String,
}

/// A project folder the palette can create under a configured root.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NewProject {
    pub path: PathBuf,
    pub label: String,
}
