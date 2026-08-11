use crate::graph::edge::{Edge, Relation};
use crate::graph::node::{Node, NodeKind, Provenance, Timestamp};
use crate::graph::query::{Direction, Query};
use crate::graph::query_context::LocalQueryContext;
use crate::graph::traversal::execute;
use crate::identity::address_of;

fn addr(name: &str) -> crate::identity::UorAddress {
    address_of(name.as_bytes())
}

fn make_node(name: &str) -> Node {
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

fn make_edge(src: &str, tgt: &str, conf: f64, rel: Relation) -> Edge {
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
fn linear_chain_downstream() {
    let nodes = vec![make_node("A"), make_node("B"), make_node("C")];
    let edges = vec![
        make_edge("A", "B", 1.0, Relation::Calls),
        make_edge("B", "C", 1.0, Relation::Calls),
    ];
    let q = Query::downstream(addr("A"), 10);
    let result = execute(&q, &LocalQueryContext::new(&nodes, &edges));

    assert_eq!(result.nodes.len(), 3);
    assert_eq!(result.edges.len(), 2);
    assert!(!result.paths.is_empty());
}

#[test]
fn linear_chain_upstream() {
    let nodes = vec![make_node("A"), make_node("B"), make_node("C")];
    let edges = vec![
        make_edge("A", "B", 1.0, Relation::Calls),
        make_edge("B", "C", 1.0, Relation::Calls),
    ];
    let q = Query {
        target: addr("C"),
        direction: Direction::Upstream,
        max_depth: 10,
        min_confidence: 0.0,
        relation_filter: None,
    };
    let result = execute(&q, &LocalQueryContext::new(&nodes, &edges));

    assert_eq!(result.nodes.len(), 3);
}

#[test]
fn max_depth_limits_traversal() {
    let nodes = vec![make_node("A"), make_node("B"), make_node("C")];
    let edges = vec![
        make_edge("A", "B", 1.0, Relation::Calls),
        make_edge("B", "C", 1.0, Relation::Calls),
    ];
    let q = Query {
        target: addr("A"),
        direction: Direction::Downstream,
        max_depth: 1,
        min_confidence: 0.0,
        relation_filter: None,
    };
    let result = execute(&q, &LocalQueryContext::new(&nodes, &edges));

    // depth=1: A + B reachable, C not
    assert_eq!(result.nodes.len(), 2);
}

#[test]
fn min_confidence_prunes_path() {
    let nodes = vec![make_node("A"), make_node("B"), make_node("C")];
    let edges = vec![
        make_edge("A", "B", 0.3, Relation::Calls),
        make_edge("B", "C", 1.0, Relation::Calls),
    ];
    let q = Query {
        target: addr("A"),
        direction: Direction::Downstream,
        max_depth: 10,
        min_confidence: 0.5,
        relation_filter: None,
    };
    let result = execute(&q, &LocalQueryContext::new(&nodes, &edges));

    // A→B has conf 0.3, below threshold — only A reachable
    assert_eq!(result.nodes.len(), 1);
}

#[test]
fn relation_filter_excludes_non_matching() {
    let nodes = vec![make_node("A"), make_node("B"), make_node("C")];
    let edges = vec![
        make_edge("A", "B", 1.0, Relation::Calls),
        make_edge("A", "C", 1.0, Relation::Imports),
    ];
    let q = Query {
        target: addr("A"),
        direction: Direction::Downstream,
        max_depth: 10,
        min_confidence: 0.0,
        relation_filter: Some(vec![Relation::Calls]),
    };
    let result = execute(&q, &LocalQueryContext::new(&nodes, &edges));

    // Only Calls edges: A→B. C not reachable via Calls.
    assert_eq!(result.nodes.len(), 2);
    assert!(result
        .nodes
        .iter()
        .all(|n| { n.address == addr("A") || n.address == addr("B") }));
}

#[test]
fn diamond_graph_both_paths() {
    let nodes = vec![
        make_node("A"),
        make_node("B"),
        make_node("C"),
        make_node("D"),
    ];
    let edges = vec![
        make_edge("A", "B", 1.0, Relation::Calls),
        make_edge("A", "C", 1.0, Relation::Calls),
        make_edge("B", "D", 1.0, Relation::Calls),
        make_edge("C", "D", 1.0, Relation::Calls),
    ];
    let q = Query::downstream(addr("A"), 10);
    let result = execute(&q, &LocalQueryContext::new(&nodes, &edges));

    assert_eq!(result.nodes.len(), 4);
}
