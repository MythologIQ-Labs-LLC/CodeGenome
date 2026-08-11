use crate::graph::edge::{Edge, Relation};
use crate::graph::export::{to_cytoscape_filtered, to_cytoscape_json};
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

fn edge(src: &str, tgt: &str, conf: f64) -> Edge {
    Edge {
        source: addr(src),
        target: addr(tgt),
        relation: Relation::Calls,
        confidence: conf,
        provenance: Provenance::tool("test", Timestamp(0)),
        evidence: vec![],
    }
}

#[test]
fn exports_correct_element_counts() {
    let nodes = vec![node("A"), node("B"), node("C")];
    let edges = vec![edge("A", "B", 1.0), edge("B", "C", 0.8)];
    let json = to_cytoscape_json(&nodes, &edges);
    assert_eq!(json["nodes"].as_array().unwrap().len(), 3);
    assert_eq!(json["edges"].as_array().unwrap().len(), 2);
}

#[test]
fn node_json_has_required_fields() {
    let nodes = vec![node("X")];
    let json = to_cytoscape_json(&nodes, &[]);
    let n = &json["nodes"][0]["data"];
    assert!(n["id"].is_string());
    assert!(n["kind"].is_string());
    assert!(n["confidence"].is_number());
}

#[test]
fn edge_json_has_required_fields() {
    let nodes = vec![node("A"), node("B")];
    let edges = vec![edge("A", "B", 0.9)];
    let json = to_cytoscape_json(&nodes, &edges);
    let e = &json["edges"][0]["data"];
    assert!(e["source"].is_string());
    assert!(e["target"].is_string());
    assert!(e["relation"].is_string());
    assert!(e["confidence"].is_number());
}

#[test]
fn filtered_export_excludes_low_confidence() {
    let nodes = vec![node("A"), node("B"), node("C")];
    let edges = vec![edge("A", "B", 0.9), edge("B", "C", 0.5)];
    let json = to_cytoscape_filtered(&nodes, &edges, 0.8, None);
    assert_eq!(json["edges"].as_array().unwrap().len(), 1);
}
