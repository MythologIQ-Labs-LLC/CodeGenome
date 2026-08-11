use crate::belief::create::{create_belief, BeliefSpec};
use codegenome_identity::graph::edge::{Edge, Relation};
use codegenome_identity::graph::node::{Node, NodeKind, Provenance, Timestamp};
use codegenome_identity::graph::query::{Direction, Query};
use codegenome_identity::graph::query_context::LocalQueryContext;
use codegenome_identity::graph::traversal;
use codegenome_identity::identity::address_of;

fn addr(name: &str) -> codegenome_identity::identity::UorAddress {
    address_of(name.as_bytes())
}

fn code_node(name: &str) -> Node {
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

fn make_belief() -> (Node, Vec<Edge>) {
    create_belief(&BeliefSpec {
        claim: "fn_x is dead code".into(),
        subject: addr("fn_x"),
        confidence: 0.8,
        supporting_evidence: vec![addr("ev_no_calls")],
        contradicting_evidence: vec![],
        actor: "analysis-agent".into(),
    })
}

#[test]
fn upstream_from_subject_reaches_belief() {
    let (bnode, bedges) = make_belief();
    let nodes = vec![code_node("fn_x"), code_node("ev_no_calls"), bnode];
    let edges = bedges;

    let ctx = LocalQueryContext::new(&nodes, &edges);
    let q = Query {
        target: addr("fn_x"),
        direction: Direction::Upstream,
        max_depth: 5,
        min_confidence: 0.0,
        relation_filter: None,
    };
    let result = traversal::execute(&q, &ctx);

    let has_belief = result.nodes.iter().any(|n| n.kind == NodeKind::Belief);
    assert!(
        has_belief,
        "Upstream from fn_x should reach the Belief node"
    );
}

#[test]
fn downstream_from_belief_reaches_subject() {
    let (bnode, bedges) = make_belief();
    let nodes = vec![code_node("fn_x"), code_node("ev_no_calls"), bnode];
    let edges = bedges;

    let ctx = LocalQueryContext::new(&nodes, &edges);
    let q = Query::downstream(addr(&format!("belief:{}", "fn_x is dead code")), 5);
    let result = traversal::execute(&q, &ctx);

    let has_subject = result.nodes.iter().any(|n| n.address == addr("fn_x"));
    assert!(has_subject, "Downstream from belief should reach fn_x");
}

#[test]
fn supports_edge_traversable_from_belief() {
    let (bnode, bedges) = make_belief();
    let nodes = vec![code_node("fn_x"), code_node("ev_no_calls"), bnode];
    let edges = bedges;

    let ctx = LocalQueryContext::new(&nodes, &edges);
    let belief_addr = address_of(b"belief:fn_x is dead code");
    let q = Query {
        target: belief_addr,
        direction: Direction::Downstream,
        max_depth: 5,
        min_confidence: 0.0,
        relation_filter: Some(vec![Relation::Supports]),
    };
    let result = traversal::execute(&q, &ctx);

    let has_evidence = result
        .nodes
        .iter()
        .any(|n| n.address == addr("ev_no_calls"));
    assert!(has_evidence, "Supports edge should reach evidence node");
}

#[test]
fn confidence_filtering_excludes_low_belief() {
    let (bnode, bedges) = create_belief(&BeliefSpec {
        claim: "weak claim".into(),
        subject: addr("fn_y"),
        confidence: 0.5,
        supporting_evidence: vec![],
        contradicting_evidence: vec![],
        actor: "agent".into(),
    });
    let nodes = vec![code_node("fn_y"), bnode];
    let edges = bedges;

    let ctx = LocalQueryContext::new(&nodes, &edges);
    let q = Query {
        target: addr("fn_y"),
        direction: Direction::Upstream,
        max_depth: 5,
        min_confidence: 0.6,
        relation_filter: None,
    };
    let result = traversal::execute(&q, &ctx);

    let has_belief = result.nodes.iter().any(|n| n.kind == NodeKind::Belief);
    assert!(
        !has_belief,
        "0.5 belief should be excluded by 0.6 threshold"
    );
}
