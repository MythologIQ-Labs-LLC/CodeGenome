use crate::graph::edge::Relation;
use crate::graph::overlay::Overlay;
use crate::overlay::scip::{Occurrence, ScipDocument, ScipOverlay, SymbolRole};

#[test]
fn empty_scip_produces_empty_overlay() {
    let overlay = ScipOverlay::from_documents(&[]);
    assert_eq!(overlay.nodes().len(), 0);
    assert_eq!(overlay.edges().len(), 0);
}

#[test]
fn scip_reference_produces_edge() {
    let doc = ScipDocument {
        relative_path: "main.rs".into(),
        occurrences: vec![
            Occurrence {
                symbol: "foo".into(),
                role: SymbolRole::Definition,
                line: 1,
            },
            Occurrence {
                symbol: "foo".into(),
                role: SymbolRole::Reference,
                line: 10,
            },
        ],
    };
    let overlay = ScipOverlay::from_documents(&[doc]);
    let refs: Vec<_> = overlay
        .edges()
        .iter()
        .filter(|e| e.relation == Relation::References)
        .collect();
    assert_eq!(
        refs.len(),
        1,
        "One reference should produce one References edge"
    );
}

#[test]
fn scip_confidence_is_one() {
    let doc = ScipDocument {
        relative_path: "lib.rs".into(),
        occurrences: vec![
            Occurrence {
                symbol: "bar".into(),
                role: SymbolRole::Definition,
                line: 5,
            },
            Occurrence {
                symbol: "bar".into(),
                role: SymbolRole::Reference,
                line: 20,
            },
        ],
    };
    let overlay = ScipOverlay::from_documents(&[doc]);
    for edge in overlay.edges() {
        assert!(
            (edge.confidence - 1.0).abs() < f64::EPSILON,
            "SCIP edges should have confidence 1.0"
        );
    }
}
