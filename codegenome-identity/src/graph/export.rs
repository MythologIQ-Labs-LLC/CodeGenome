use crate::graph::edge::{Edge, Relation};
use crate::graph::node::Node;

/// Export nodes and edges to Cytoscape.js elements JSON.
pub fn to_cytoscape_json(
    nodes: &[Node], edges: &[Edge],
) -> serde_json::Value {
    to_cytoscape_filtered(nodes, edges, 0.0, None)
}

/// Export with optional filtering by min confidence and relation types.
pub fn to_cytoscape_filtered(
    nodes: &[Node],
    edges: &[Edge],
    min_confidence: f64,
    relations: Option<&[Relation]>,
) -> serde_json::Value {
    let cy_nodes: Vec<serde_json::Value> = nodes
        .iter()
        .map(|n| {
            serde_json::json!({
                "data": {
                    "id": format!("{:?}", n.address),
                    "kind": format!("{:?}", n.kind),
                    "confidence": n.confidence,
                    "label": node_label(n),
                }
            })
        })
        .collect();

    let cy_edges: Vec<serde_json::Value> = edges
        .iter()
        .filter(|e| {
            e.confidence >= min_confidence
                && relations
                    .is_none_or(|r| r.contains(&e.relation))
        })
        .map(|e| {
            serde_json::json!({
                "data": {
                    "source": format!("{:?}", e.source),
                    "target": format!("{:?}", e.target),
                    "relation": format!("{:?}", e.relation),
                    "confidence": e.confidence,
                }
            })
        })
        .collect();

    serde_json::json!({
        "nodes": cy_nodes,
        "edges": cy_edges,
    })
}

fn node_label(n: &Node) -> String {
    if let Some(span) = &n.span {
        format!("{:?} L{}:{}", n.kind, span.start_line, span.end_line)
    } else {
        format!("{:?}", n.kind)
    }
}
