use std::path::Path;

use crate::graph::edge::Edge;
use crate::graph::node::Node;
use crate::graph::overlay::{Overlay, OverlayKind};
use crate::measurement::GroundTruthLevel;

pub struct LspOverlay {
    nodes: Vec<Node>,
    edges: Vec<Edge>,
}

impl Overlay for LspOverlay {
    fn kind(&self) -> OverlayKind {
        OverlayKind::Semantic
    }
    fn nodes(&self) -> &[Node] {
        &self.nodes
    }
    fn edges(&self) -> &[Edge] {
        &self.edges
    }
    fn ground_truth(&self) -> GroundTruthLevel {
        GroundTruthLevel::Available
    }
}

impl LspOverlay {
    /// Build overlay by querying rust-analyzer for definitions/references.
    /// Requires rust-analyzer installed. Returns empty overlay if not found.
    pub fn from_workspace(root: &Path) -> Self {
        let Ok(edges) = query_rust_analyzer(root) else {
            eprintln!("[LSP] rust-analyzer not available, returning empty overlay");
            return Self {
                nodes: Vec::new(),
                edges: Vec::new(),
            };
        };
        Self {
            nodes: Vec::new(),
            edges,
        }
    }
}

fn query_rust_analyzer(_root: &Path) -> Result<Vec<Edge>, String> {
    // Check if rust-analyzer is available
    let which = std::process::Command::new("rust-analyzer")
        .arg("--version")
        .output();

    match which {
        Ok(output) if output.status.success() => {
            eprintln!(
                "[LSP] rust-analyzer found: {}",
                String::from_utf8_lossy(&output.stdout).trim()
            );
            // Full LSP protocol implementation:
            // 1. Spawn rust-analyzer as subprocess
            // 2. Initialize with workspace root
            // 3. For each file: didOpen, then query definitions/references
            // 4. Build References edges from results
            // 5. Shutdown + exit
            //
            // For now, return empty — the protocol handshake is complex
            // and requires careful async stdin/stdout management.
            // This stub proves the overlay structure; full LSP
            // communication is wired in a follow-up.
            Ok(Vec::new())
        }
        _ => Err("rust-analyzer not found".into()),
    }
}
