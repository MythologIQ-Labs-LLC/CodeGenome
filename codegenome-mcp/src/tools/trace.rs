use codegenome_identity::graph::edge::Relation;
use codegenome_identity::graph::node::NodeKind;
use codegenome_identity::graph::overlay::Overlay;
use codegenome_identity::graph::query::{Direction, Query};
use codegenome_identity::graph::query_context::LocalQueryContext;
use codegenome_identity::graph::traversal;

use crate::tools::inputs::TraceInput;
use crate::tools::CodegenomeTools;

impl CodegenomeTools {
    /// Process trace: find an entrypoint and traverse its call chain.
    pub fn trace(&self, input: &TraceInput) -> String {
        let Some((overlay, index)) = self.load_with_index() else {
            return r#"{"error":"no index found"}"#.into();
        };

        // Try resolving as file:line first, fall back to process node scan
        let target_addr = if let Some(addr) = try_resolve_entrypoint(&input.entrypoint, &index) {
            addr
        } else {
            // Fall back to process node matching
            let proc_node = overlay.nodes.iter().find(|n| n.kind == NodeKind::Process);
            match proc_node {
                Some(p) => p.address,
                None => {
                    return format!(r#"{{"error":"no process for '{}'"}}"#, input.entrypoint);
                }
            }
        };

        let query = Query {
            target: target_addr,
            direction: Direction::Downstream,
            max_depth: input.max_depth,
            min_confidence: 0.0,
            relation_filter: Some(vec![Relation::Calls, Relation::PartOfProcess]),
        };
        let ctx = LocalQueryContext::new(overlay.nodes(), overlay.edges());
        let result = traversal::execute(&query, &ctx);

        let chain: Vec<_> = result
            .nodes
            .iter()
            .map(|n| {
                let loc = n
                    .span
                    .as_ref()
                    .map(|s| format!("line {}:{}", s.start_line, s.end_line))
                    .unwrap_or_else(|| format!("{:?}", n.address));
                serde_json::json!({
                    "node": loc,
                    "kind": format!("{:?}", n.kind),
                    "confidence": n.confidence,
                })
            })
            .collect();

        let mut resp = serde_json::json!({
            "entrypoint": input.entrypoint,
            "chain": chain,
        });
        resp["meta"] = self.response_meta();
        serde_json::to_string_pretty(&resp).unwrap_or_default()
    }
}

fn try_resolve_entrypoint(
    ep: &str,
    index: &codegenome_identity::graph::resolve::FileIndex,
) -> Option<codegenome_identity::identity::UorAddress> {
    // Try "file:line" format
    let parts: Vec<&str> = ep.rsplitn(2, ':').collect();
    if parts.len() == 2 {
        if let Ok(line) = parts[0].parse::<u32>() {
            return index.resolve(parts[1], line);
        }
    }
    None
}
