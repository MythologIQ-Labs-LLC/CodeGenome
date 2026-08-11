use crate::graph::community::{find_components, module_clusters};
use crate::graph::edge::{Edge, Relation};
use crate::graph::node::{Node, NodeKind, Provenance, Timestamp};
use crate::identity::address_of;

fn addr(name: &str) -> crate::identity::UorAddress {
    address_of(name.as_bytes())
}

fn node(name: &str) -> Node {
    let a = addr(name);
    Node {
        address: a,
        kind: NodeKind::Symbol,
        provenance: Provenance::tool("test", Timestamp(0)),
        confidence: 1.0,
        created_at: Timestamp(0),
        content_hash: a,
        span: None,
    }
}

fn edge(src: &str, tgt: &str, conf: f64, rel: Relation) -> Edge {
    Edge {
        source: addr(src),
        target: addr(tgt),
        relation: rel,
        confidence: conf,
        provenance: Provenance::tool("test", Timestamp(0)),
        evidence: vec![],
    }
}

#[test]
fn two_disconnected_pairs_produce_two_components() {
    let nodes = vec![node("A"), node("B"), node("C"), node("D")];
    let edges = vec![
        edge("A", "B", 1.0, Relation::Calls),
        edge("C", "D", 1.0, Relation::Calls),
    ];
    let comps = find_components(&nodes, &edges, &[Relation::Calls], 0.0);
    assert_eq!(comps.len(), 2);
    assert!(comps.iter().all(|c| c.members.len() == 2));
}

#[test]
fn bridge_edge_merges_components() {
    let nodes = vec![node("A"), node("B"), node("C"), node("D")];
    let edges = vec![
        edge("A", "B", 1.0, Relation::Calls),
        edge("B", "C", 1.0, Relation::Imports),
        edge("C", "D", 1.0, Relation::Calls),
    ];
    let comps = find_components(&nodes, &edges, &[Relation::Calls, Relation::Imports], 0.0);
    assert_eq!(comps.len(), 1);
    assert_eq!(comps[0].members.len(), 4);
}

#[test]
fn relation_filter_splits_components() {
    let nodes = vec![node("A"), node("B"), node("C"), node("D")];
    let edges = vec![
        edge("A", "B", 1.0, Relation::Calls),
        edge("B", "C", 1.0, Relation::Imports),
        edge("C", "D", 1.0, Relation::Calls),
    ];
    // Only Calls — B→C bridge is Imports, so 2 components
    let comps = find_components(&nodes, &edges, &[Relation::Calls], 0.0);
    assert_eq!(comps.len(), 2);
}

#[test]
fn confidence_filter_splits_component() {
    let nodes = vec![node("A"), node("B"), node("C")];
    let edges = vec![
        edge("A", "B", 0.9, Relation::Calls),
        edge("B", "C", 0.5, Relation::Calls),
    ];
    let comps = find_components(&nodes, &edges, &[Relation::Calls], 0.8);
    assert_eq!(comps.len(), 1); // Only A-B, C is isolated (size 1 filtered out)
    assert_eq!(comps[0].members.len(), 2);
}

#[test]
fn module_clusters_uses_calls_and_imports() {
    let nodes = vec![node("A"), node("B")];
    let edges = vec![edge("A", "B", 1.0, Relation::Imports)];
    let comps = module_clusters(&nodes, &edges, 0.0);
    assert_eq!(comps.len(), 1);
}
