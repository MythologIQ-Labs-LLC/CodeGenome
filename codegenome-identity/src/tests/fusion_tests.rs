use std::path::PathBuf;

use crate::graph::edge::{Edge, Relation};
use crate::graph::node::{Node, NodeKind, Provenance, Timestamp};
use crate::graph::overlay::Overlay;
use crate::identity::address_of;
use crate::overlay::fused::fuse;
use crate::overlay::flow::FlowOverlay;
use crate::overlay::semantic::SemanticOverlay;
use crate::overlay::syntax::parse_rust_files;

fn addr(s: &str) -> crate::identity::UorAddress {
    address_of(s.as_bytes())
}

fn prov() -> Provenance {
    Provenance::tool("test", Timestamp(0))
}

fn make_edge(src: &str, tgt: &str, rel: Relation, conf: f64) -> Edge {
    Edge {
        source: addr(src),
        target: addr(tgt),
        relation: rel,
        confidence: conf,
        provenance: prov(),
        evidence: vec![],
    }
}

fn make_node(name: &str) -> Node {
    Node {
        address: addr(name),
        kind: NodeKind::Symbol,
        provenance: prov(),
        confidence: 1.0,
        created_at: Timestamp(0),
        content_hash: addr(name),
        span: None,
    }
}

struct SimpleOverlay {
    nodes: Vec<Node>,
    edges: Vec<Edge>,
}

impl Overlay for SimpleOverlay {
    fn kind(&self) -> crate::graph::overlay::OverlayKind {
        crate::graph::overlay::OverlayKind::Custom("test".into())
    }
    fn nodes(&self) -> &[Node] { &self.nodes }
    fn edges(&self) -> &[Edge] { &self.edges }
    fn ground_truth(&self) -> crate::measurement::GroundTruthLevel {
        crate::measurement::GroundTruthLevel::Available
    }
}

#[test]
fn single_overlay_passes_through() {
    let ov = SimpleOverlay {
        nodes: vec![make_node("a"), make_node("b")],
        edges: vec![make_edge("a", "b", Relation::Calls, 0.7)],
    };
    let fused = fuse(&[&ov]);
    assert_eq!(fused.nodes().len(), 2);
    assert_eq!(fused.edges().len(), 1);
    assert!((fused.edges()[0].confidence - 0.7).abs() < 0.001);
}

#[test]
fn duplicate_edges_fuse_noisy_or() {
    let ov1 = SimpleOverlay {
        nodes: vec![make_node("a"), make_node("b")],
        edges: vec![make_edge("a", "b", Relation::Calls, 0.7)],
    };
    let ov2 = SimpleOverlay {
        nodes: vec![make_node("a"), make_node("b")],
        edges: vec![make_edge("a", "b", Relation::Calls, 0.8)],
    };
    let fused = fuse(&[&ov1, &ov2]);
    assert_eq!(fused.edges().len(), 1);
    // noisy-OR: 1 - (1-0.7)(1-0.8) = 1 - 0.06 = 0.94
    let conf = fused.edges()[0].confidence;
    assert!((conf - 0.94).abs() < 0.01, "Expected ~0.94, got {conf}");
}

#[test]
fn different_relations_stay_separate() {
    let ov = SimpleOverlay {
        nodes: vec![make_node("a"), make_node("b")],
        edges: vec![
            make_edge("a", "b", Relation::Calls, 0.7),
            make_edge("a", "b", Relation::Contains, 1.0),
        ],
    };
    let fused = fuse(&[&ov]);
    assert_eq!(fused.edges().len(), 2);
}

#[test]
fn nodes_deduplicate_by_address() {
    let ov1 = SimpleOverlay {
        nodes: vec![make_node("a")],
        edges: vec![],
    };
    let ov2 = SimpleOverlay {
        nodes: vec![make_node("a"), make_node("b")],
        edges: vec![],
    };
    let fused = fuse(&[&ov1, &ov2]);
    assert_eq!(fused.nodes().len(), 2);
}

#[test]
fn fused_confidence_always_higher() {
    let ov1 = SimpleOverlay {
        nodes: vec![make_node("a"), make_node("b")],
        edges: vec![make_edge("a", "b", Relation::Calls, 0.5)],
    };
    let ov2 = SimpleOverlay {
        nodes: vec![make_node("a"), make_node("b")],
        edges: vec![make_edge("a", "b", Relation::Calls, 0.6)],
    };
    let fused = fuse(&[&ov1, &ov2]);
    let conf = fused.edges()[0].confidence;
    assert!(conf >= 0.6, "Fused {conf} should be >= max individual 0.6");
}

#[test]
fn self_index_fusion_reduces_edge_count() {
    let files = collect_rs_files(std::path::Path::new("src"));
    let syntax = parse_rust_files(&files);
    let semantic = SemanticOverlay::from_syntax(&syntax, &files);
    let flow = FlowOverlay::from_source(&files);

    let sum = syntax.edges().len() + semantic.edges().len() + flow.edges().len();
    let fused = fuse(&[&syntax, &semantic, &flow]);
    eprintln!("Sum: {sum}, Fused: {}", fused.edges().len());
    assert!(fused.edges().len() <= sum, "Fusion should not add edges");
}

fn collect_rs_files(dir: &std::path::Path) -> Vec<(PathBuf, Vec<u8>)> {
    let mut files = Vec::new();
    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                files.extend(collect_rs_files(&path));
            } else if path.extension().is_some_and(|e| e == "rs") {
                if let Ok(content) = std::fs::read(&path) {
                    files.push((path, content));
                }
            }
        }
    }
    files
}
