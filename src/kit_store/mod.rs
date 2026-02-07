//! Types and persistence helpers for the git-based kit store.

use std::path::PathBuf;

use serde::{Deserialize, Serialize};

pub mod manifest;
pub mod storage;

/// Metadata describing a kit repository and its installable content.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct KitManifest {
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub author: String,
    #[serde(default)]
    pub version: String,
    #[serde(default)]
    pub repo_url: String,
    #[serde(default)]
    pub scripts: Vec<String>,
    #[serde(default)]
    pub scriptlets: Vec<String>,
}

/// Registry entry for a locally installed kit.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct InstalledKit {
    pub name: String,
    pub path: PathBuf,
    pub repo_url: String,
    pub git_hash: String,
    pub installed_at: String,
}
