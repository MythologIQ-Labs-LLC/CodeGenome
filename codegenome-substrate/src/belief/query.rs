use codegenome_identity::graph::edge::{Edge, Relation};
use codegenome_identity::graph::node::{Node, NodeKind};
use codegenome_identity::identity::UorAddress;

/// Find all beliefs about a given subject address.
/// Returns each belief node paired with its reasoning edges.
pub fn beliefs_about(
    subject: UorAddress,
    nodes: &[Node],
    edges: &[Edge],
) -> Vec<(Node, Vec<Edge>)> {
    let belief_addrs: Vec<UorAddress> = edges
        .iter()
        .filter(|e| e.relation == Relation::AboutSubject && e.target == subject)
        .map(|e| e.source)
        .collect();

    belief_addrs
        .iter()
        .filter_map(|&addr| {
            let node = nodes.iter().find(|n| n.address == addr)?;
            let related: Vec<Edge> = edges.iter().filter(|e| e.source == addr).cloned().collect();
            Some((node.clone(), related))
        })
        .collect()
}

/// Find all beliefs created by a given actor.
pub fn beliefs_by_actor(actor: &str, nodes: &[Node]) -> Vec<Node> {
    nodes
        .iter()
        .filter(|n| n.kind == NodeKind::Belief && n.provenance.actor == actor)
        .cloned()
        .collect()
}

/// Find supporting evidence addresses for a belief.
pub fn supporting_evidence(belief: UorAddress, edges: &[Edge]) -> Vec<UorAddress> {
    edges
        .iter()
        .filter(|e| e.source == belief && e.relation == Relation::Supports)
        .map(|e| e.target)
        .collect()
}

/// Find contradicting evidence addresses for a belief.
pub fn contradicting_evidence(belief: UorAddress, edges: &[Edge]) -> Vec<UorAddress> {
    edges
        .iter()
        .filter(|e| e.source == belief && e.relation == Relation::Contradicts)
        .map(|e| e.target)
        .collect()
}
