use std::collections::HashMap;

use crate::graph::edge::{Edge, Relation};
use crate::graph::node::{Node, Provenance, Source, Timestamp};
use crate::graph::overlay::{Overlay, OverlayKind};
use crate::identity::UorAddress;
use crate::measurement::GroundTruthLevel;
use crate::overlay::flow::FlowOverlay;

pub struct PdgOverlay {
    nodes: Vec<Node>,
    edges: Vec<Edge>,
}

impl Overlay for PdgOverlay {
    fn kind(&self) -> OverlayKind {
        OverlayKind::Custom("pdg".into())
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

impl PdgOverlay {
    /// Build PDG from flow overlay: add ControlDependence edges + copy DataFlow.
    pub fn from_flow(flow: &FlowOverlay) -> Self {
        let prov = Provenance {
            source: Source::ToolOutput,
            actor: "pdg-builder".into(),
            timestamp: Timestamp(0),
            justification: None,
        };

        let mut edges: Vec<Edge> = flow
            .edges()
            .iter()
            .filter(|e| e.relation == Relation::DataFlow)
            .cloned()
            .collect();

        let branch_targets = find_branch_targets(flow);
        for (branch, dependents) in &branch_targets {
            for dep in dependents {
                edges.push(Edge {
                    source: *branch,
                    target: *dep,
                    relation: Relation::ControlDependence,
                    confidence: 1.0,
                    provenance: prov.clone(),
                    evidence: vec![],
                });
            }
        }

        Self {
            nodes: Vec::new(),
            edges,
        }
    }
}

/// Find nodes with multiple outgoing ControlFlow edges (branch points)
/// and their immediate targets (control-dependent nodes).
fn find_branch_targets(flow: &FlowOverlay) -> HashMap<UorAddress, Vec<UorAddress>> {
    let mut outgoing: HashMap<UorAddress, Vec<UorAddress>> = HashMap::new();
    for edge in flow.edges() {
        if edge.relation == Relation::ControlFlow {
            outgoing.entry(edge.source).or_default().push(edge.target);
        }
    }
    outgoing
        .into_iter()
        .filter(|(_, targets)| targets.len() > 1)
        .collect()
}
