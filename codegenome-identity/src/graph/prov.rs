//! W3C PROV-DM export (PROV-JSON serialization).
//!
//! Maps the graph's intrinsic provenance onto the standard model,
//! following the PROV-AGENT pattern for AI-agent systems:
//!
//! - every graph node and every edge-claim is a `prov:Entity`
//! - every distinct provenance actor implies a `prov:Activity`
//!   (the extraction / inference / assertion that produced the fact)
//!   and a `prov:Agent` associated with it
//! - facts link to their producer via `wasGeneratedBy` and
//!   `wasAttributedTo`; edge-claims link to the code entities they
//!   are about via `cg:source` / `cg:target` and carry their noisy-OR
//!   confidence and supporting evidence addresses
//! - facts whose provenance is inference or belief assertion are
//!   explicitly labelled `cg:aiGenerated: true`, keeping LLM-derived
//!   claims distinguishable from compiler-grade observations
//!
//! The output is interoperable with PROV tooling and with
//! provenance-carrying memory interchange formats (e.g. MIF).

use std::collections::BTreeMap;

use crate::graph::edge::Edge;
use crate::graph::node::{Node, NodeKind, Source};

const PREFIX_CG: &str = "urn:codegenome:";
const PREFIX_PROV: &str = "http://www.w3.org/ns/prov#";

/// Serialize nodes and edges as a PROV-JSON document.
pub fn to_prov_json(nodes: &[Node], edges: &[Edge]) -> serde_json::Value {
    let mut entities = BTreeMap::new();
    let mut activities = BTreeMap::new();
    let mut agents = BTreeMap::new();
    let mut generated_by = BTreeMap::new();
    let mut attributed_to = BTreeMap::new();
    let mut associated_with = BTreeMap::new();

    for node in nodes {
        let entity_id = format!("cg:node:{}", node.address);
        let mut attrs = serde_json::json!({
            "cg:kind": format!("{:?}", node.kind),
            "cg:confidence": node.confidence,
            "cg:contentHash": node.content_hash.to_string(),
        });
        if let Some(span) = &node.span {
            attrs["cg:startLine"] = span.start_line.into();
            attrs["cg:endLine"] = span.end_line.into();
        }
        if is_ai_generated(&node.provenance.source, node.kind == NodeKind::Belief) {
            attrs["cg:aiGenerated"] = true.into();
        }
        entities.insert(entity_id.clone(), attrs);
        link_provenance(
            &entity_id,
            &node.provenance,
            &mut activities,
            &mut agents,
            &mut generated_by,
            &mut attributed_to,
            &mut associated_with,
        );
    }

    for (i, edge) in edges.iter().enumerate() {
        let entity_id = format!(
            "cg:claim:{}:{:?}:{}",
            edge.source.short_hex(),
            edge.relation,
            edge.target.short_hex()
        );
        let mut attrs = serde_json::json!({
            "cg:relation": format!("{:?}", edge.relation),
            "cg:confidence": edge.confidence,
            "cg:source": format!("cg:node:{}", edge.source),
            "cg:target": format!("cg:node:{}", edge.target),
        });
        if !edge.evidence.is_empty() {
            attrs["cg:evidence"] = edge
                .evidence
                .iter()
                .map(|a| serde_json::Value::String(a.to_string()))
                .collect::<Vec<_>>()
                .into();
        }
        if is_ai_generated(&edge.provenance.source, false) {
            attrs["cg:aiGenerated"] = true.into();
        }
        // Distinct claims can share source/relation/target across
        // overlays; suffix with the index to keep entity ids unique.
        let entity_id = format!("{entity_id}:{i}");
        entities.insert(entity_id.clone(), attrs);
        link_provenance(
            &entity_id,
            &edge.provenance,
            &mut activities,
            &mut agents,
            &mut generated_by,
            &mut attributed_to,
            &mut associated_with,
        );
    }

    serde_json::json!({
        "prefix": { "cg": PREFIX_CG, "prov": PREFIX_PROV },
        "entity": entities,
        "activity": activities,
        "agent": agents,
        "wasGeneratedBy": generated_by,
        "wasAttributedTo": attributed_to,
        "wasAssociatedWith": associated_with,
    })
}

/// LLM-derived or human-asserted beliefs are AI/assertion provenance;
/// tool output and consolidation are observation-grade.
fn is_ai_generated(source: &Source, is_belief: bool) -> bool {
    is_belief || matches!(source, Source::Inferred)
}

#[allow(clippy::too_many_arguments)]
fn link_provenance(
    entity_id: &str,
    prov: &crate::graph::node::Provenance,
    activities: &mut BTreeMap<String, serde_json::Value>,
    agents: &mut BTreeMap<String, serde_json::Value>,
    generated_by: &mut BTreeMap<String, serde_json::Value>,
    attributed_to: &mut BTreeMap<String, serde_json::Value>,
    associated_with: &mut BTreeMap<String, serde_json::Value>,
) {
    let slug: String = prov
        .actor
        .chars()
        .map(|c| if c.is_alphanumeric() { c } else { '-' })
        .collect();
    let activity_id = format!("cg:activity:{slug}");
    let agent_id = format!("cg:agent:{slug}");

    activities.entry(activity_id.clone()).or_insert_with(|| {
        serde_json::json!({
            "prov:type": activity_type(&prov.source),
        })
    });
    agents.entry(agent_id.clone()).or_insert_with(|| {
        serde_json::json!({
            "prov:type": agent_type(&prov.source),
            "cg:actor": prov.actor,
        })
    });
    associated_with
        .entry(format!("_:assoc-{slug}"))
        .or_insert_with(|| {
            serde_json::json!({
                "prov:activity": activity_id,
                "prov:agent": agent_id,
            })
        });

    generated_by.insert(
        format!("_:gen-{}", generated_by.len()),
        serde_json::json!({
            "prov:entity": entity_id,
            "prov:activity": activity_id,
            "prov:time": prov.timestamp.0,
        }),
    );
    attributed_to.insert(
        format!("_:attr-{}", attributed_to.len()),
        serde_json::json!({
            "prov:entity": entity_id,
            "prov:agent": agent_id,
        }),
    );
}

fn activity_type(source: &Source) -> &'static str {
    match source {
        Source::ToolOutput => "cg:ToolExtraction",
        Source::Inferred => "cg:Inference",
        Source::UserStated => "cg:Assertion",
        Source::Consolidated => "cg:Consolidation",
    }
}

fn agent_type(source: &Source) -> &'static str {
    match source {
        Source::UserStated => "prov:Person",
        _ => "prov:SoftwareAgent",
    }
}
