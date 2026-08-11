use std::path::PathBuf;

use crate::graph::edge::Edge;
use crate::graph::node::Node;
use crate::graph::overlay::{Overlay, OverlayKind};
use crate::index::flow::FlowResult;
use crate::measurement::GroundTruthLevel;

pub struct FlowOverlay {
    nodes: Vec<Node>,
    edges: Vec<Edge>,
}

impl Overlay for FlowOverlay {
    fn kind(&self) -> OverlayKind {
        OverlayKind::Flow
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

impl FlowOverlay {
    pub fn from_flow_result(result: &FlowResult) -> Self {
        Self {
            nodes: result.nodes.clone(),
            edges: result.edges.clone(),
        }
    }

    /// Backward-compatible wrapper. Delegates to index::flow.
    pub fn from_source(files: &[(PathBuf, Vec<u8>)]) -> Self {
        Self::from_flow_result(&crate::index::flow::extract_flow(files))
    }
}
