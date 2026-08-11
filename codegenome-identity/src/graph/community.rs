use std::collections::HashMap;

use crate::graph::edge::{Edge, Relation};
use crate::graph::node::Node;
use crate::identity::UorAddress;

/// A connected component: nodes reachable from each other
/// via edges matching the filter criteria.
pub struct Component {
    pub members: Vec<UorAddress>,
    pub edge_count: usize,
    pub avg_confidence: f64,
}

/// Find connected components filtered by relation types
/// and minimum confidence. Uses union-find.
pub fn find_components(
    nodes: &[Node],
    edges: &[Edge],
    relations: &[Relation],
    min_confidence: f64,
) -> Vec<Component> {
    let mut parent: HashMap<UorAddress, UorAddress> =
        nodes.iter().map(|n| (n.address, n.address)).collect();

    let filtered: Vec<&Edge> = edges
        .iter()
        .filter(|e| e.confidence >= min_confidence && relations.contains(&e.relation))
        .collect();

    for edge in &filtered {
        let a = find(&parent, edge.source);
        let b = find(&parent, edge.target);
        if a != b {
            parent.insert(a, b);
        }
    }

    // Group by root
    let mut groups: HashMap<UorAddress, Vec<UorAddress>> = HashMap::new();
    for &addr in parent.keys() {
        let root = find(&parent, addr);
        groups.entry(root).or_default().push(addr);
    }

    // Build components with edge counts and avg confidence
    groups
        .into_values()
        .filter(|members| members.len() > 1)
        .map(|members| {
            let member_set: std::collections::HashSet<_> = members.iter().copied().collect();
            let component_edges: Vec<&Edge> = filtered
                .iter()
                .filter(|e| member_set.contains(&e.source) && member_set.contains(&e.target))
                .copied()
                .collect();
            let avg = if component_edges.is_empty() {
                0.0
            } else {
                component_edges.iter().map(|e| e.confidence).sum::<f64>()
                    / component_edges.len() as f64
            };
            Component {
                members,
                edge_count: component_edges.len(),
                avg_confidence: avg,
            }
        })
        .collect()
}

/// Convenience: find components using Calls + Imports edges.
pub fn module_clusters(nodes: &[Node], edges: &[Edge], min_confidence: f64) -> Vec<Component> {
    find_components(
        nodes,
        edges,
        &[Relation::Calls, Relation::Imports],
        min_confidence,
    )
}

fn find(parent: &HashMap<UorAddress, UorAddress>, mut addr: UorAddress) -> UorAddress {
    while let Some(&p) = parent.get(&addr) {
        if p == addr {
            break;
        }
        addr = p;
    }
    addr
}
