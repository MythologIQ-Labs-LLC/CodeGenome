use std::path::{Path, PathBuf};

use crate::graph::node::Span;
use crate::identity::{address_of, UorAddress};

/// Lightweight reference to a node's address and span.
#[derive(Clone, Debug)]
pub struct NodeRef {
    pub address: UorAddress,
    pub span: Span,
}

/// Maps source files to their graph nodes for fast resolution.
pub struct FileIndex {
    entries: Vec<(PathBuf, UorAddress, Vec<NodeRef>)>,
}

impl FileIndex {
    /// Build a file index by walking source_dir and matching
    /// file addresses against stored overlay nodes via Contains edges.
    pub fn build(
        source_dir: &Path,
        nodes: &[crate::graph::node::Node],
        edges: &[crate::graph::edge::Edge],
    ) -> Self {
        let files = collect_rs_paths(source_dir);
        let mut entries = Vec::new();

        for path in &files {
            let file_addr = file_address(path);
            // Find nodes connected by Contains edges from this file
            let child_addrs: std::collections::HashSet<UorAddress> = edges
                .iter()
                .filter(|e| {
                    e.source == file_addr && e.relation == crate::graph::edge::Relation::Contains
                })
                .map(|e| e.target)
                .collect();

            let children: Vec<NodeRef> = nodes
                .iter()
                .filter(|n| n.span.is_some() && child_addrs.contains(&n.address))
                .map(|n| NodeRef {
                    address: n.address,
                    span: n.span.unwrap(),
                })
                .collect();
            if !children.is_empty() {
                entries.push((path.clone(), file_addr, children));
            }
        }

        Self { entries }
    }

    /// Resolve a file path + line number to a UorAddress.
    /// Matches by path suffix for ergonomics.
    pub fn resolve(&self, file: &str, line: u32) -> Option<UorAddress> {
        let matching = self
            .entries
            .iter()
            .find(|(path, _, _)| path_matches_suffix(path, file))?;
        matching
            .2
            .iter()
            .find(|nr| nr.span.start_line <= line && nr.span.end_line >= line)
            .map(|nr| nr.address)
    }

    pub fn file_count(&self) -> usize {
        self.entries.len()
    }
}

/// Check if a path ends with the given suffix.
/// e.g. "src/lib.rs" matches "/full/path/to/src/lib.rs"
fn path_matches_suffix(path: &Path, suffix: &str) -> bool {
    let path_str = path.to_string_lossy();
    let normalized = path_str.replace('\\', "/");
    let suffix_normalized = suffix.replace('\\', "/");
    normalized.ends_with(&suffix_normalized)
}

fn file_address(path: &Path) -> UorAddress {
    let content = format!("file:{}", path.display());
    address_of(content.as_bytes())
}

fn collect_rs_paths(dir: &Path) -> Vec<PathBuf> {
    let mut paths = Vec::new();
    let Ok(entries) = std::fs::read_dir(dir) else {
        return paths;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            paths.extend(collect_rs_paths(&path));
        } else if path.extension().is_some_and(|e| e == "rs") {
            paths.push(path);
        }
    }
    paths
}
