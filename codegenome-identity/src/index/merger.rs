use std::collections::{HashMap, HashSet};

use crate::confidence::multi_path_confidence;
use crate::graph::edge::{Edge, Relation};
use crate::graph::node::{Node, Provenance, Source, Timestamp};
use crate::graph::overlay::Overlay;
use crate::identity::UorAddress;
use crate::overlay::fused::FusedOverlay;

type EdgeKey = (UorAddress, UorAddress, Relation);

/// Fuse multiple overlays into one. Deduplicates nodes by address.
/// Merges edges with same (source, target, relation) using noisy-OR.
pub fn fuse(overlays: &[&dyn Overlay]) -> FusedOverlay {
    let nodes = dedup_nodes(overlays);
    let edges = fuse_edges(overlays);
    FusedOverlay::new(nodes, edges)
}

fn dedup_nodes(overlays: &[&dyn Overlay]) -> Vec<Node> {
    let mut seen = HashSet::new();
    let mut nodes = Vec::new();
    for overlay in overlays {
        for node in overlay.nodes() {
            if seen.insert(node.address) {
                nodes.push(node.clone());
            }
        }
    }
    nodes
}

fn fuse_edges(overlays: &[&dyn Overlay]) -> Vec<Edge> {
    let mut groups: HashMap<EdgeKey, Vec<&Edge>> = HashMap::new();
    for overlay in overlays {
        for edge in overlay.edges() {
            let key = (edge.source, edge.target, edge.relation.clone());
            groups.entry(key).or_default().push(edge);
        }
    }
    groups
        .into_values()
        .map(|edges| merge_edge_group(&edges))
        .collect()
}

fn merge_edge_group(edges: &[&Edge]) -> Edge {
    if edges.len() == 1 {
        return edges[0].clone();
    }
    let confidences: Vec<f64> = edges.iter().map(|e| e.confidence).collect();
    let fused_conf = multi_path_confidence(&confidences);
    let mut evidence: Vec<UorAddress> = Vec::new();
    for edge in edges {
        evidence.extend_from_slice(&edge.evidence);
    }
    Edge {
        source: edges[0].source,
        target: edges[0].target,
        relation: edges[0].relation.clone(),
        confidence: fused_conf,
        provenance: Provenance {
            source: Source::Consolidated,
            actor: "fusion".into(),
            timestamp: Timestamp(0),
            justification: None,
        },
        evidence,
    }
}
