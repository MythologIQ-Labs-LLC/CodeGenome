use codegenome_identity::graph::edge::{Edge, Relation};
use codegenome_identity::graph::node::{Node, NodeKind, Provenance, Source, Timestamp};
use codegenome_identity::identity::{address_of, UorAddress};

/// A belief claim about a code artifact. Pure value.
pub struct BeliefSpec {
    pub claim: String,
    pub subject: UorAddress,
    pub confidence: f64,
    pub supporting_evidence: Vec<UorAddress>,
    pub contradicting_evidence: Vec<UorAddress>,
    pub actor: String,
}

/// Create a Belief node and its reasoning edges.
/// Returns pure values — does not write to storage.
/// Caller is responsible for write gating before persistence.
pub fn create_belief(spec: &BeliefSpec) -> (Node, Vec<Edge>) {
    let content = format!("belief:{}", spec.claim);
    let address = address_of(content.as_bytes());
    let prov = Provenance {
        source: Source::Inferred,
        actor: spec.actor.clone(),
        timestamp: now_timestamp(),
        justification: spec.supporting_evidence.first().copied(),
    };

    let node = Node {
        address,
        kind: NodeKind::Belief,
        provenance: prov.clone(),
        confidence: spec.confidence,
        created_at: prov.timestamp,
        content_hash: address_of(spec.claim.as_bytes()),
        span: None,
    };

    let mut edges = vec![about_subject(address, spec, &prov)];

    for &ev in &spec.supporting_evidence {
        edges.push(reasoning_edge(
            address,
            ev,
            Relation::Supports,
            spec.confidence,
            &prov,
        ));
    }
    for &ev in &spec.contradicting_evidence {
        edges.push(reasoning_edge(
            address,
            ev,
            Relation::Contradicts,
            spec.confidence,
            &prov,
        ));
    }

    (node, edges)
}

fn about_subject(belief: UorAddress, spec: &BeliefSpec, prov: &Provenance) -> Edge {
    Edge {
        source: belief,
        target: spec.subject,
        relation: Relation::AboutSubject,
        confidence: spec.confidence,
        provenance: prov.clone(),
        evidence: vec![belief, spec.subject],
    }
}

fn reasoning_edge(
    belief: UorAddress,
    target: UorAddress,
    relation: Relation,
    confidence: f64,
    prov: &Provenance,
) -> Edge {
    Edge {
        source: belief,
        target,
        relation,
        confidence,
        provenance: prov.clone(),
        evidence: vec![belief, target],
    }
}

fn now_timestamp() -> Timestamp {
    let ms = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64;
    Timestamp(ms)
}
