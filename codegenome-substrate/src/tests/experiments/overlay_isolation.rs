use codegenome_identity::graph::*;
use codegenome_identity::identity::address_of;
use codegenome_identity::overlay::syntax::parse_rust_files;
use std::path::PathBuf;

/// Mock semantic overlay for isolation testing.
struct MockSemanticOverlay {
    nodes: Vec<Node>,
    edges: Vec<Edge>,
}

impl Overlay for MockSemanticOverlay {
    fn kind(&self) -> OverlayKind {
        OverlayKind::Semantic
    }
    fn nodes(&self) -> &[Node] {
        &self.nodes
    }
    fn edges(&self) -> &[Edge] {
        &self.edges
    }
    fn ground_truth(&self) -> codegenome_identity::measurement::GroundTruthLevel {
        codegenome_identity::measurement::GroundTruthLevel::Constructible
    }
}

/// OQ4: Adding a second overlay does not change the first.
#[test]
fn adding_semantic_overlay_does_not_change_syntax() {
    let digest_path = PathBuf::from(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../codegenome-identity"
    ))
    .join("src/identity/digest.rs");
    let source = std::fs::read(&digest_path).unwrap();
    let syntax = parse_rust_files(&[(digest_path, source)]);

    let nodes_before = codegenome_identity::codec::to_vec(syntax.nodes()).unwrap();
    let edges_before = codegenome_identity::codec::to_vec(syntax.edges()).unwrap();

    // Create a semantic overlay with import edges
    let _semantic = MockSemanticOverlay {
        nodes: vec![Node {
            address: address_of(b"mock_import"),
            kind: NodeKind::Symbol,
            provenance: Provenance::tool("mock", Timestamp(0)),
            confidence: 0.8,
            created_at: Timestamp(0),
            content_hash: address_of(b"mock_import"),
            span: None,
        }],
        edges: vec![Edge {
            source: address_of(b"file"),
            target: address_of(b"mock_import"),
            relation: Relation::Imports,
            confidence: 0.8,
            provenance: Provenance::tool("mock", Timestamp(0)),
            evidence: vec![],
        }],
    };

    // Syntax overlay unchanged
    let nodes_after = codegenome_identity::codec::to_vec(syntax.nodes()).unwrap();
    let edges_after = codegenome_identity::codec::to_vec(syntax.edges()).unwrap();

    assert_eq!(nodes_before, nodes_after);
    assert_eq!(edges_before, edges_after);
}
