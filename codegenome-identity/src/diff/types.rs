use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::identity::UorAddress;

/// Owned representation of a git diff. No lifetimes from git2.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct OwnedDiff {
    pub files: Vec<OwnedDiffFile>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct OwnedDiffFile {
    pub path: PathBuf,
    pub status: DiffStatus,
    pub hunks: Vec<OwnedHunk>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct OwnedHunk {
    pub new_start: u32,
    pub new_lines: u32,
    pub old_start: u32,
    pub old_lines: u32,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum DiffStatus {
    Added,
    Modified,
    Deleted,
    Renamed,
}

/// Result of detect_changes: affected nodes, edges, and
/// propagated impact/staleness signals.
#[derive(Clone, Debug, Default)]
pub struct ChangeSet {
    pub changed_nodes: Vec<UorAddress>,
    pub affected_edges: Vec<UorAddress>,
    pub impact: std::collections::HashMap<UorAddress, f64>,
    pub staleness: std::collections::HashMap<UorAddress, f64>,
}
