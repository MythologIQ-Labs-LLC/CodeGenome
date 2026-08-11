use std::collections::HashSet;

use crate::graph::edge::{Edge, Relation};
use crate::graph::node::{Node, NodeKind, Provenance, Timestamp};
use crate::graph::query::Direction;
use crate::graph::query_context::{LocalQueryContext, QueryContext};
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
fn neighbors_returns_correct_targets() {
    let nodes = vec![node("A"), node("B"), node("C")];
    let edges = vec![edge("A", "B", 0.9), edge("A", "C", 0.7)];
    let ctx = LocalQueryContext::new(&nodes, &edges);

    let nbrs = ctx.neighbors(addr("A"), &Direction::Downstream, &None, 0.0);
    assert_eq!(nbrs.len(), 2);
    let targets: Vec<_> = nbrs.iter().map(|(a, _, _)| *a).collect();
    assert!(targets.contains(&addr("B")));
    assert!(targets.contains(&addr("C")));
}

#[test]
fn neighbors_filters_by_confidence() {
    let nodes = vec![node("A"), node("B"), node("C")];
    let edges = vec![edge("A", "B", 0.9), edge("A", "C", 0.5)];
    let ctx = LocalQueryContext::new(&nodes, &edges);

    let nbrs = ctx.neighbors(addr("A"), &Direction::Downstream, &None, 0.8);
    assert_eq!(nbrs.len(), 1);
    assert_eq!(nbrs[0].0, addr("B"));
}

#[test]
fn collect_nodes_returns_only_matching() {
    let nodes = vec![node("A"), node("B"), node("C")];
    let edges = vec![];
    let ctx = LocalQueryContext::new(&nodes, &edges);

    let set: HashSet<_> = [addr("A"), addr("C")].into();
    let collected = ctx.collect_nodes(&set);
    assert_eq!(collected.len(), 2);
}

#[test]
fn node_returns_none_for_unknown() {
    let nodes = vec![node("A")];
    let edges = vec![];
    let ctx = LocalQueryContext::new(&nodes, &edges);

    assert!(ctx.node(addr("A")).is_some());
    assert!(ctx.node(addr("Z")).is_none());
}
