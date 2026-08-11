use crate::confidence::{impact_score, multi_path_confidence, path_confidence};
use crate::graph::edge::{Edge, Relation};
use crate::graph::node::{Provenance, Timestamp};
use crate::identity::address_of;

fn make_edge(conf: f64) -> Edge {
    Edge {
        source: address_of(b"src"),
        target: address_of(b"tgt"),
        relation: Relation::Calls,
        confidence: conf,
        provenance: Provenance::tool("test", Timestamp(0)),
        evidence: vec![],
    }
}

#[test]
fn empty_path_confidence_is_identity() {
    assert!((path_confidence(&[]) - 1.0).abs() < f64::EPSILON);
}

#[test]
fn single_edge_path_confidence() {
    let e = make_edge(0.8);
    assert!((path_confidence(&[&e]) - 0.8).abs() < f64::EPSILON);
}

#[test]
fn multi_edge_path_confidence_is_product() {
    let e1 = make_edge(0.8);
    let e2 = make_edge(0.5);
    let expected = 0.8 * 0.5;
    assert!((path_confidence(&[&e1, &e2]) - expected).abs() < 1e-10);
}

#[test]
fn empty_multi_path_is_zero() {
    assert!((multi_path_confidence(&[]) - 0.0).abs() < f64::EPSILON);
}

#[test]
fn three_independent_paths_noisy_or() {
    let confs = vec![0.5, 0.3, 0.2];
    // 1 - (1-0.5)(1-0.3)(1-0.2) = 1 - 0.5*0.7*0.8 = 1 - 0.28 = 0.72
    let expected = 0.72;
    assert!(
        (multi_path_confidence(&confs) - expected).abs() < 1e-10,
        "Expected {}, got {}",
        expected,
        multi_path_confidence(&confs)
    );
}

#[test]
fn impact_score_clamps_inputs() {
    assert!((impact_score(1.5, 0.5) - 0.5).abs() < f64::EPSILON);
    assert!((impact_score(0.5, -0.1) - 0.0).abs() < f64::EPSILON);
    assert!((impact_score(0.8, 1.0) - 0.8).abs() < f64::EPSILON);
}
