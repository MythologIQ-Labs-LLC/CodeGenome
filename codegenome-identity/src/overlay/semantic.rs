use std::path::PathBuf;

use crate::graph::edge::Edge;
use crate::graph::node::Node;
use crate::graph::overlay::{Overlay, OverlayKind};
use crate::index::resolver::ResolvedEdges;
use crate::measurement::GroundTruthLevel;
use crate::overlay::syntax::SyntaxOverlay;

pub struct SemanticOverlay {
    nodes: Vec<Node>,
    edges: Vec<Edge>,
}

impl Overlay for SemanticOverlay {
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
        GroundTruthLevel::Constructible
    }
}

impl SemanticOverlay {
    pub fn from_resolved(resolved: &ResolvedEdges) -> Self {
        Self {
            nodes: Vec::new(),
            edges: resolved.edges.clone(),
        }
    }

    /// Backward-compatible wrapper. Delegates to index::resolver.
    pub fn from_syntax(_syntax: &SyntaxOverlay, files: &[(PathBuf, Vec<u8>)]) -> Self {
        let parsed = crate::index::parser::parse_files(files);
        Self::from_resolved(&crate::index::resolver::resolve(&parsed, files))
    }
}
