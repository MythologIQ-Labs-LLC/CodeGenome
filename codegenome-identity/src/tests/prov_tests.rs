use crate::graph::edge::{Edge, Relation};
use crate::graph::node::{Node, NodeKind, Provenance, Source, Timestamp};
use crate::graph::prov::to_prov_json;
use crate::identity::address_of;

fn node(kind: NodeKind, source: Source, actor: &str) -> Node {
    Node {
        address: address_of(format!("{kind:?}:{actor}").as_bytes()),
        kind,
        provenance: Provenance {
            source,
            actor: actor.into(),
            timestamp: Timestamp(42),
            justification: None,
        },
        confidence: 1.0,
        created_at: Timestamp(42),
        content_hash: address_of(b"content"),
        span: None,
    }
}

#[test]
fn nodes_become_entities_with_generation_and_attribution() {
    let n = node(NodeKind::Symbol, Source::ToolOutput, "tree-sitter-rust");
    let doc = to_prov_json(&[n.clone()], &[]);

    let entity_id = format!("cg:node:{}", n.address);
    assert!(doc["entity"][&entity_id].is_object(), "entity present");
    assert_eq!(doc["entity"][&entity_id]["cg:kind"], "Symbol");

    let gens = doc["wasGeneratedBy"].as_object().unwrap();
    assert_eq!(gens.len(), 1);
    let attr = doc["wasAttributedTo"].as_object().unwrap();
    assert_eq!(attr.len(), 1);

    let agent = &doc["agent"]["cg:agent:tree-sitter-rust"];
    assert_eq!(agent["prov:type"], "prov:SoftwareAgent");
    let activity = &doc["activity"]["cg:activity:tree-sitter-rust"];
    assert_eq!(activity["prov:type"], "cg:ToolExtraction");
}

#[test]
fn edges_become_claim_entities_with_confidence_and_evidence() {
    let a = address_of(b"a");
    let b = address_of(b"b");
    let ev = address_of(b"evidence");
    let edge = Edge {
        source: a,
        target: b,
        relation: Relation::Calls,
        confidence: 0.94,
        provenance: Provenance {
            source: Source::Inferred,
            actor: "heuristic-resolver".into(),
            timestamp: Timestamp(7),
            justification: None,
        },
        evidence: vec![ev],
    };
    let doc = to_prov_json(&[], &[edge]);

    let entities = doc["entity"].as_object().unwrap();
    assert_eq!(entities.len(), 1);
    let (_, claim) = entities.iter().next().unwrap();
    assert_eq!(claim["cg:relation"], "Calls");
    assert_eq!(claim["cg:confidence"], 0.94);
    assert_eq!(claim["cg:evidence"][0], ev.to_string());
    assert_eq!(
        claim["cg:aiGenerated"], true,
        "inferred claims must be labelled AI-generated"
    );
}

#[test]
fn belief_nodes_are_labelled_ai_generated() {
    let n = node(NodeKind::Belief, Source::UserStated, "claude-code");
    let doc = to_prov_json(&[n.clone()], &[]);
    let entity_id = format!("cg:node:{}", n.address);
    assert_eq!(doc["entity"][&entity_id]["cg:aiGenerated"], true);
    // user-asserted beliefs attribute to a person, not software
    assert_eq!(
        doc["agent"]["cg:agent:claude-code"]["prov:type"],
        "prov:Person"
    );
}

#[test]
fn compiler_grade_facts_are_not_labelled_ai_generated() {
    let n = node(NodeKind::File, Source::ToolOutput, "tree-sitter-rust");
    let doc = to_prov_json(&[n.clone()], &[]);
    let entity_id = format!("cg:node:{}", n.address);
    assert!(doc["entity"][&entity_id].get("cg:aiGenerated").is_none());
}
