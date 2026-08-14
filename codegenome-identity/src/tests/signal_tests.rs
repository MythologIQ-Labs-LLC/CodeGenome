use crate::graph::*;
use crate::identity::address_of;
use crate::signal::impact::{propagate_impact, propagate_impact_directional};
use crate::signal::staleness::propagate_staleness;
use crate::signal::topo::topological_sort;

fn addr(name: &str) -> crate::identity::UorAddress {
    address_of(name.as_bytes())
}

fn prov() -> Provenance {
    Provenance::tool("test", Timestamp(0))
}

struct Chain {
    nodes: Vec<Node>,
    edges: Vec<Edge>,
}

impl Overlay for Chain {
    fn kind(&self) -> OverlayKind {
        OverlayKind::Syntax
    }
    fn nodes(&self) -> &[Node] {
        &self.nodes
    }
    fn edges(&self) -> &[Edge] {
        &self.edges
    }
    fn ground_truth(&self) -> crate::measurement::GroundTruthLevel {
        crate::measurement::GroundTruthLevel::Constructible
    }
}

fn make_edge(src: &str, tgt: &str, confidence: f64) -> Edge {
    Edge {
        source: addr(src),
        target: addr(tgt),
        relation: Relation::Calls,
        confidence,
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

fn linear_chain() -> Chain {
    Chain {
        nodes: vec![make_node("A"), make_node("B"), make_node("C")],
        edges: vec![make_edge("A", "B", 0.5), make_edge("B", "C", 0.5)],
    }
}

fn diamond() -> Chain {
    Chain {
        nodes: vec![
            make_node("A"),
            make_node("B"),
            make_node("C"),
            make_node("D"),
        ],
        edges: vec![
            make_edge("A", "B", 0.9),
            make_edge("A", "C", 0.5),
            make_edge("B", "D", 0.8),
            make_edge("C", "D", 0.3),
        ],
    }
}

// --- Topo Sort ---

#[test]
fn topo_sort_linear_forward() {
    let chain = linear_chain();
    let sorted = topological_sort(
        &[addr("A")],
        &[&chain as &dyn Overlay],
        Direction::Downstream,
    );
    let a_pos = sorted.iter().position(|a| *a == addr("A")).unwrap();
    let b_pos = sorted.iter().position(|a| *a == addr("B")).unwrap();
    let c_pos = sorted.iter().position(|a| *a == addr("C")).unwrap();
    assert!(a_pos < b_pos);
    assert!(b_pos < c_pos);
}

#[test]
fn topo_sort_diamond() {
    let d = diamond();
    let sorted = topological_sort(&[addr("A")], &[&d as &dyn Overlay], Direction::Downstream);
    assert_eq!(sorted.len(), 4);
    let a_pos = sorted.iter().position(|a| *a == addr("A")).unwrap();
    let d_pos = sorted.iter().position(|a| *a == addr("D")).unwrap();
    assert!(a_pos < d_pos);
}

// --- Impact ---

#[test]
fn impact_linear_attenuation() {
    let chain = linear_chain();
    let impact = propagate_impact(&[addr("A")], &[&chain as &dyn Overlay]);
    assert!((impact[&addr("A")] - 1.0).abs() < f64::EPSILON);
    assert!((impact[&addr("B")] - 0.5).abs() < f64::EPSILON);
    assert!((impact[&addr("C")] - 0.25).abs() < f64::EPSILON);
}

#[test]
fn impact_diamond_max_path() {
    let d = diamond();
    let impact = propagate_impact(&[addr("A")], &[&d as &dyn Overlay]);
    // Path A→B→D: 0.9*0.8 = 0.72
    // Path A→C→D: 0.5*0.3 = 0.15
    // max = 0.72
    assert!((impact[&addr("D")] - 0.72).abs() < 0.01);
}

#[test]
fn directional_impact_distinguishes_upstream_and_downstream() {
    let chain = linear_chain();
    let overlays: [&dyn Overlay; 1] = [&chain];

    let downstream = propagate_impact_directional(&[addr("B")], &overlays, Direction::Downstream);
    assert_eq!(downstream.get(&addr("A")), None);
    assert!((downstream[&addr("B")] - 1.0).abs() < f64::EPSILON);
    assert!((downstream[&addr("C")] - 0.5).abs() < f64::EPSILON);

    let upstream = propagate_impact_directional(&[addr("B")], &overlays, Direction::Upstream);
    assert!((upstream[&addr("A")] - 0.5).abs() < f64::EPSILON);
    assert!((upstream[&addr("B")] - 1.0).abs() < f64::EPSILON);
    assert_eq!(upstream.get(&addr("C")), None);
}

#[test]
fn directional_impact_both_is_max_union() {
    let chain = linear_chain();
    let overlays: [&dyn Overlay; 1] = [&chain];

    let both = propagate_impact_directional(&[addr("B")], &overlays, Direction::Both);
    assert!((both[&addr("A")] - 0.5).abs() < f64::EPSILON);
    assert!((both[&addr("B")] - 1.0).abs() < f64::EPSILON);
    assert!((both[&addr("C")] - 0.5).abs() < f64::EPSILON);
}

#[test]
fn impact_isolation_unreachable() {
    let chain = linear_chain();
    let isolated = make_node("X");
    let mut extended = Chain {
        nodes: chain.nodes.clone(),
        edges: chain.edges.clone(),
    };
    extended.nodes.push(isolated);

    let impact = propagate_impact(&[addr("A")], &[&extended as &dyn Overlay]);
    assert!(!impact.contains_key(&addr("X")));
}

#[test]
fn impact_empty_graph() {
    let empty = Chain {
        nodes: vec![],
        edges: vec![],
    };
    let impact = propagate_impact(&[addr("A")], &[&empty as &dyn Overlay]);
    assert!(impact.is_empty() || impact.len() == 1);
}

// --- Staleness ---

#[test]
fn staleness_backward() {
    let chain = linear_chain();
    let staleness = propagate_staleness(&[addr("C")], &[&chain as &dyn Overlay]);
    assert!((staleness[&addr("C")] - 1.0).abs() < f64::EPSILON);
    assert!((staleness[&addr("B")] - 0.5).abs() < f64::EPSILON);
    assert!((staleness[&addr("A")] - 0.25).abs() < f64::EPSILON);
}

#[test]
fn staleness_attenuation_matches_impact() {
    // Same graph, reversed direction, same math
    let chain = linear_chain();
    let impact = propagate_impact(&[addr("A")], &[&chain as &dyn Overlay]);
    let staleness = propagate_staleness(&[addr("C")], &[&chain as &dyn Overlay]);
    // Impact A→C = 0.25, Staleness C→A = 0.25
    assert!((impact[&addr("C")] - staleness[&addr("A")]).abs() < f64::EPSILON);
}

// --- Self-index integration ---

#[test]
fn impact_on_self_index_is_non_trivial() {
    let files = crate::tests::self_index::load_own_source();
    let overlay = crate::overlay::syntax::parse_rust_files(&files);

    let first_symbol = overlay
        .nodes()
        .iter()
        .find(|n| n.kind == NodeKind::Symbol)
        .expect("Should have symbols");

    let impact = propagate_impact(&[first_symbol.address], &[&overlay as &dyn Overlay]);
    // Impact map should contain at least the changed node
    assert!(!impact.is_empty());
}
