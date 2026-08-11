use std::path::PathBuf;

use crate::graph::edge::Edge;
use crate::graph::node::Node;
use crate::graph::overlay::{Overlay, OverlayKind};
use crate::index::parser::ParsedFile;
use crate::measurement::GroundTruthLevel;

/// Syntax overlay: tree-sitter parse results as graph nodes
/// and edges. Implements the Overlay trait.
pub struct SyntaxOverlay {
    nodes: Vec<Node>,
    edges: Vec<Edge>,
}

impl SyntaxOverlay {
    pub fn from_parsed(parsed: &[ParsedFile]) -> Self {
        let mut nodes = Vec::new();
        let mut edges = Vec::new();
        for pf in parsed {
            nodes.extend(pf.nodes.iter().cloned());
            edges.extend(pf.edges.iter().cloned());
        }
        Self { nodes, edges }
    }

    pub fn node_count(&self) -> usize {
        self.nodes.len()
    }

    pub fn edge_count(&self) -> usize {
        self.edges.len()
    }
}

impl Overlay for SyntaxOverlay {
    fn kind(&self) -> OverlayKind {
        OverlayKind::Syntax
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

/// Backward-compatible wrapper. Delegates to index::parser.
pub fn parse_rust_files(files: &[(PathBuf, Vec<u8>)]) -> SyntaxOverlay {
    SyntaxOverlay::from_parsed(&crate::index::parser::parse_files(files))
}
