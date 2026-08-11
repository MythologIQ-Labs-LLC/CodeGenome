use crate::federation::query_context::FederatedQueryContext;
use codegenome_identity::graph::edge::{Edge, Relation};
use codegenome_identity::graph::node::{Node, NodeKind, Provenance, Timestamp};
use codegenome_identity::graph::query::Query;
use codegenome_identity::graph::traversal;
use codegenome_identity::identity::address_of;

fn addr(name: &str) -> codegenome_identity::identity::UorAddress {
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
fn downstream_crosses_repo_boundary_via_symbol_edge() {
    // Repo A: A_fn → A_caller (local Calls edge)
    // Cross-repo: A_caller → B_helper (Imports at 0.7)
    // Repo B: B_helper (isolated node)
    let nodes = vec![node("A_fn"), node("A_caller"), node("B_helper")];
    let local = vec![edge("A_fn", "A_caller", 1.0, Relation::Calls)];
    let cross = vec![edge("A_caller", "B_helper", 0.7, Relation::Imports)];

    let ctx = FederatedQueryContext::from_parts(nodes, local, cross);
    let q = Query::downstream(addr("A_fn"), 10);
    let result = traversal::execute(&q, &ctx);

    assert_eq!(
        result.nodes.len(),
        3,
        "Should traverse A_fn → A_caller → B_helper"
    );
}

#[test]
fn min_confidence_blocks_cross_repo_bridge() {
    let nodes = vec![node("A_fn"), node("A_caller"), node("B_helper")];
    let local = vec![edge("A_fn", "A_caller", 1.0, Relation::Calls)];
    let cross = vec![edge("A_caller", "B_helper", 0.7, Relation::Imports)];

    let ctx = FederatedQueryContext::from_parts(nodes, local, cross);
    let q = Query {
        target: addr("A_fn"),
        direction: codegenome_identity::graph::query::Direction::Downstream,
        max_depth: 10,
        min_confidence: 0.8, // 1.0 * 0.7 = 0.7 < 0.8
        relation_filter: None,
    };
    let result = traversal::execute(&q, &ctx);

    assert_eq!(
        result.nodes.len(),
        2,
        "Should NOT cross 0.7 bridge with min_confidence 0.8"
    );
}

#[test]
fn upstream_propagates_into_importing_repo() {
    let nodes = vec![node("A_caller"), node("B_helper")];
    let local = vec![];
    let cross = vec![edge("A_caller", "B_helper", 0.7, Relation::Imports)];

    let ctx = FederatedQueryContext::from_parts(nodes, local, cross);
    let q = Query {
        target: addr("B_helper"),
        direction: codegenome_identity::graph::query::Direction::Upstream,
        max_depth: 10,
        min_confidence: 0.0,
        relation_filter: None,
    };
    let result = traversal::execute(&q, &ctx);

    assert_eq!(
        result.nodes.len(),
        2,
        "Upstream from B_helper should reach A_caller"
    );
}

#[test]
fn no_cross_edges_stays_within_repo() {
    let nodes = vec![node("A"), node("B"), node("C")];
    let local = vec![edge("A", "B", 1.0, Relation::Calls)];
    let cross: Vec<Edge> = vec![];

    let ctx = FederatedQueryContext::from_parts(nodes, local, cross);
    let q = Query::downstream(addr("A"), 10);
    let result = traversal::execute(&q, &ctx);

    assert_eq!(result.nodes.len(), 2, "Only A and B reachable");
}
