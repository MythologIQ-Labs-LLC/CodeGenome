use crate::diff::propagator::propagate;
use crate::diff::{DiffStatus, OwnedDiff, OwnedDiffFile, OwnedHunk};
use crate::graph::edge::{Edge, Relation};
use crate::graph::node::{Node, NodeKind, Provenance, Span, Timestamp};
use crate::graph::overlay::{Overlay, OverlayKind};
use crate::identity::address_of;
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

fn make_node_at(name: &str, line: u32) -> Node {
    let a = address_of(name.as_bytes());
    Node {
        address: a,
        kind: NodeKind::Symbol,
        provenance: Provenance::tool("test", Timestamp(0)),
        confidence: 1.0,
        created_at: Timestamp(0),
        content_hash: a,
        span: Some(Span {
            start_byte: 0,
            end_byte: 100,
            start_line: line,
            end_line: line + 5,
        }),
    }
}

#[test]
fn diff_touching_function_appears_in_changed_nodes() {
    let node_a = make_node_at("func_a", 1);
    let node_b = make_node_at("func_b", 20);
    let addr_a = node_a.address;

    let overlay = TestOverlay {
        nodes: vec![node_a, node_b],
        edges: vec![],
    };

    let diff = OwnedDiff {
        files: vec![OwnedDiffFile {
            path: "test.rs".into(),
            status: DiffStatus::Modified,
            hunks: vec![OwnedHunk {
                new_start: 3,
                new_lines: 2,
                old_start: 3,
                old_lines: 2,
            }],
        }],
    };

    let result = propagate(&diff, &[&overlay]);
    assert!(
        result.changed_nodes.contains(&addr_a),
        "func_a spans lines 1-6, hunk at 3-5 should match"
    );
}

#[test]
fn impact_attenuates_by_edge_confidence() {
    let node_a = make_node_at("a", 1);
    let node_b = make_node_at("b", 20);
    let addr_a = node_a.address;
    let addr_b = node_b.address;

    let overlay = TestOverlay {
        nodes: vec![node_a, node_b],
        edges: vec![Edge {
            source: addr_a,
            target: addr_b,
            relation: Relation::Calls,
            confidence: 0.5,
            provenance: Provenance::tool("test", Timestamp(0)),
            evidence: vec![],
        }],
    };

    let diff = OwnedDiff {
        files: vec![OwnedDiffFile {
            path: "test.rs".into(),
            status: DiffStatus::Modified,
            hunks: vec![OwnedHunk {
                new_start: 3,
                new_lines: 2,
                old_start: 3,
                old_lines: 2,
            }],
        }],
    };

    let result = propagate(&diff, &[&overlay]);
    let impact_b = result.impact.get(&addr_b).copied().unwrap_or(0.0);
    assert!(
        impact_b <= 0.5 + f64::EPSILON,
        "Impact on b should be attenuated by 0.5 edge confidence, got {}",
        impact_b
    );
}
