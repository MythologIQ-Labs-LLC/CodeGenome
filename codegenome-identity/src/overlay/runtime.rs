use std::path::{Path, PathBuf};

use crate::graph::edge::Edge;
use crate::graph::node::Node;
use crate::graph::overlay::{Overlay, OverlayKind};
use crate::index::dynamic::TraceResult;
use crate::measurement::GroundTruthLevel;

pub struct RuntimeOverlay {
    nodes: Vec<Node>,
    edges: Vec<Edge>,
}

impl Overlay for RuntimeOverlay {
    fn kind(&self) -> OverlayKind {
        OverlayKind::Runtime
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

impl RuntimeOverlay {
    pub fn from_trace(result: &TraceResult) -> Self {
        Self {
            nodes: Vec::new(),
            edges: result.edges.clone(),
        }
    }

    /// Backward-compatible wrapper. Delegates to index::dynamic.
    pub fn from_trace_file(
        trace_path: &Path,
        source_files: &[(PathBuf, Vec<u8>)],
    ) -> Result<Self, String> {
        let result = crate::index::dynamic::ingest_trace(trace_path, source_files)?;
        Ok(Self::from_trace(&result))
    }
}
