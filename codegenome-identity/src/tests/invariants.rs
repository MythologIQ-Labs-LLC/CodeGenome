use proptest::prelude::*;

use crate::confidence;
use crate::graph::*;
use crate::identity::address_of;

// --- UOR Determinism ---

proptest! {
    #[test]
    fn uor_same_bytes_same_address(bytes in proptest::collection::vec(any::<u8>(), 0..1024)) {
        let a = address_of(&bytes);
        let b = address_of(&bytes);
        prop_assert_eq!(a, b);
    }

    #[test]
    fn uor_different_bytes_different_address(
        a_bytes in proptest::collection::vec(any::<u8>(), 1..512),
        b_bytes in proptest::collection::vec(any::<u8>(), 1..512),
    ) {
        prop_assume!(a_bytes != b_bytes);
        let a = address_of(&a_bytes);
        let b = address_of(&b_bytes);
        prop_assert_ne!(a, b);
    }
}

// --- Node/Edge Value Semantics ---

fn sample_provenance() -> Provenance {
    Provenance::tool("test-actor", Timestamp(1000))
}

fn sample_node(content: &[u8]) -> Node {
    Node {
        address: address_of(content),
        kind: NodeKind::Symbol,
        provenance: sample_provenance(),
        confidence: 0.95,
        created_at: Timestamp(1000),
        content_hash: address_of(content),
        span: None,
    }
}

fn sample_node_with_span(content: &[u8], span: Span) -> Node {
    Node {
        address: address_of(content),
        kind: NodeKind::Symbol,
        provenance: sample_provenance(),
        confidence: 0.95,
        created_at: Timestamp(1000),
        content_hash: address_of(content),
        span: Some(span),
    }
}

fn sample_edge(src: &[u8], tgt: &[u8]) -> Edge {
    Edge {
        source: address_of(src),
        target: address_of(tgt),
        relation: Relation::Calls,
        confidence: 0.9,
        provenance: sample_provenance(),
        evidence: vec![],
    }
}

#[test]
fn node_roundtrip_serialization() {
    let node = sample_node(b"fn main()");
    let encoded = bincode::serialize(&node).unwrap();
    let decoded: Node = bincode::deserialize(&encoded).unwrap();
    assert_eq!(node, decoded);
}

#[test]
fn edge_roundtrip_serialization() {
    let edge = sample_edge(b"caller", b"callee");
    let encoded = bincode::serialize(&edge).unwrap();
    let decoded: Edge = bincode::deserialize(&encoded).unwrap();
    assert_eq!(edge, decoded);
}

#[test]
fn node_clone_is_identical() {
    let node = sample_node(b"struct Foo");
    let cloned = node.clone();
    assert_eq!(node, cloned);
}

#[test]
fn edge_clone_is_identical() {
    let edge = sample_edge(b"a", b"b");
    let cloned = edge.clone();
    assert_eq!(edge, cloned);
}

// --- Overlay Isolation ---

struct TestOverlay {
    overlay_kind: OverlayKind,
    node_list: Vec<Node>,
    edge_list: Vec<Edge>,
}

impl Overlay for TestOverlay {
    fn kind(&self) -> OverlayKind {
        self.overlay_kind.clone()
    }
    fn nodes(&self) -> &[Node] {
        &self.node_list
    }
    fn edges(&self) -> &[Edge] {
        &self.edge_list
    }
    fn ground_truth(&self) -> crate::measurement::GroundTruthLevel {
        crate::measurement::GroundTruthLevel::Available
    }
}

#[test]
fn overlay_isolation_adding_b_does_not_change_a() {
    let overlay_a = TestOverlay {
        overlay_kind: OverlayKind::Syntax,
        node_list: vec![sample_node(b"file_a")],
        edge_list: vec![],
    };

    let nodes_before: Vec<Node> = overlay_a.nodes().to_vec();
    let edges_before: Vec<Edge> = overlay_a.edges().to_vec();

    // "Add" overlay B (just create it — overlays are independent)
    let _overlay_b = TestOverlay {
        overlay_kind: OverlayKind::Semantic,
        node_list: vec![sample_node(b"symbol_x")],
        edge_list: vec![sample_edge(b"file_a", b"symbol_x")],
    };

    // Overlay A unchanged
    assert_eq!(overlay_a.nodes().to_vec(), nodes_before);
    assert_eq!(overlay_a.edges().to_vec(), edges_before);
}

// --- Confidence Monotonicity ---

proptest! {
    #[test]
    fn path_confidence_decreases_with_more_edges(
        confidences in proptest::collection::vec(0.0f64..=1.0, 1..10),
    ) {
        let provenance = sample_provenance();
        let edges: Vec<Edge> = confidences.iter().enumerate().map(|(i, &c)| {
            Edge {
                source: address_of(format!("s{i}").as_bytes()),
                target: address_of(format!("t{i}").as_bytes()),
                relation: Relation::Calls,
                confidence: c,
                provenance: provenance.clone(),
                evidence: vec![],
            }
        }).collect();

        let edge_refs: Vec<&Edge> = edges.iter().collect();

        for len in 1..edge_refs.len() {
            let shorter = confidence::path_confidence(&edge_refs[..len]);
            let longer = confidence::path_confidence(&edge_refs[..=len]);
            prop_assert!(longer <= shorter + f64::EPSILON);
        }
    }

    #[test]
    fn multi_path_increases_with_more_paths(
        path_confs in proptest::collection::vec(0.0f64..=1.0, 1..10),
    ) {
        for len in 1..path_confs.len() {
            let fewer = confidence::multi_path_confidence(&path_confs[..len]);
            let more = confidence::multi_path_confidence(&path_confs[..=len]);
            prop_assert!(more >= fewer - f64::EPSILON);
        }
    }
}

// --- Span and New Relations ---

#[test]
fn span_equality() {
    let a = Span {
        start_byte: 0,
        end_byte: 10,
        start_line: 1,
        end_line: 1,
    };
    let b = Span {
        start_byte: 0,
        end_byte: 10,
        start_line: 1,
        end_line: 1,
    };
    let c = Span {
        start_byte: 5,
        end_byte: 15,
        start_line: 2,
        end_line: 3,
    };
    assert_eq!(a, b);
    assert_ne!(a, c);
}

#[test]
fn node_with_span_roundtrip() {
    let span = Span {
        start_byte: 10,
        end_byte: 50,
        start_line: 3,
        end_line: 7,
    };
    let node = sample_node_with_span(b"fn example()", span);
    let encoded = bincode::serialize(&node).unwrap();
    let decoded: Node = bincode::deserialize(&encoded).unwrap();
    assert_eq!(node, decoded);
    assert_eq!(decoded.span, Some(span));
}

#[test]
fn flow_relation_variants_roundtrip() {
    for relation in [
        Relation::ControlFlow,
        Relation::ControlDependence,
        Relation::DataFlow,
        Relation::Dominates,
    ] {
        let edge = Edge {
            source: address_of(b"src"),
            target: address_of(b"tgt"),
            relation: relation.clone(),
            confidence: 1.0,
            provenance: sample_provenance(),
            evidence: vec![],
        };
        let encoded = bincode::serialize(&edge).unwrap();
        let decoded: Edge = bincode::deserialize(&encoded).unwrap();
        assert_eq!(edge.relation, decoded.relation);
    }
}

// --- Provenance Completeness ---

#[test]
fn provenance_actor_is_never_empty() {
    let prov = Provenance::tool("tree-sitter", Timestamp(1000));
    assert!(!prov.actor.is_empty());
}

#[test]
fn uor_address_display_is_64_hex_chars() {
    let addr = address_of(b"test");
    let display = format!("{addr}");
    assert_eq!(display.len(), 64);
}
