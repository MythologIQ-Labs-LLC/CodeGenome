use crate::graph::edge::{Edge, Relation};
use crate::graph::node::{Node, NodeKind, Provenance, Timestamp};
use crate::graph::overlay::{Overlay, OverlayKind};
use crate::identity::address_of;
use crate::index::merger::fuse;
use crate::measurement::GroundTruthLevel;

struct TestOverlay {
    nodes: Vec<Node>,
    edges: Vec<Edge>,
}

impl Overlay for TestOverlay {
    fn kind(&self) -> OverlayKind {
        OverlayKind::Custom("test".into())
    }
    fn nodes(&self) -> &[Node] {
        &self.nodes
    }
    fn edges(&self) -> &[Edge] {
        &self.edges
    }
    fn ground_truth(&self) -> GroundTruthLevel {
        GroundTruthLevel::Constructible
    }
}

fn make_prov() -> Provenance {
    Provenance::tool("test", Timestamp(0))
}

fn make_node(name: &str) -> Node {
    let addr = address_of(name.as_bytes());
    Node {
        address: addr,
        kind: NodeKind::Symbol,
        provenance: make_prov(),
        confidence: 1.0,
        created_at: Timestamp(0),
        content_hash: addr,
        span: None,
    }
}

fn make_edge(src: &str, tgt: &str, conf: f64) -> Edge {
    Edge {
        source: address_of(src.as_bytes()),
        target: address_of(tgt.as_bytes()),
        relation: Relation::Calls,
        confidence: conf,
        provenance: make_prov(),
        evidence: vec![],
    }
}

#[test]
fn dedup_nodes_by_address() {
    let n = make_node("foo");
    let o1 = TestOverlay {
        nodes: vec![n.clone()],
        edges: vec![],
    };
    let o2 = TestOverlay {
        nodes: vec![n],
        edges: vec![],
    };
    let fused = fuse(&[&o1, &o2]);
    assert_eq!(fused.nodes().len(), 1);
}

#[test]
fn noisy_or_merges_same_edge() {
    let e1 = make_edge("a", "b", 0.6);
    let e2 = make_edge("a", "b", 0.4);
    let o1 = TestOverlay {
        nodes: vec![],
        edges: vec![e1],
    };
    let o2 = TestOverlay {
        nodes: vec![],
        edges: vec![e2],
    };
    let fused = fuse(&[&o1, &o2]);

    assert_eq!(fused.edges().len(), 1);
    // noisy-OR: 1 - (1-0.6)(1-0.4) = 1 - 0.24 = 0.76
    let expected = 0.76;
    assert!(
        (fused.edges()[0].confidence - expected).abs() < 1e-10,
        "Expected {}, got {}",
        expected,
        fused.edges()[0].confidence
    );
}

#[test]
fn distinct_edges_preserved() {
    let e1 = make_edge("a", "b", 0.5);
    let e2 = make_edge("b", "c", 0.8);
    let o = TestOverlay {
        nodes: vec![],
        edges: vec![e1, e2],
    };
    let fused = fuse(&[&o]);
    assert_eq!(fused.edges().len(), 2);
}
