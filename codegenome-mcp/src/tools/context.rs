use codegenome_identity::graph::overlay::Overlay;
use codegenome_identity::graph::query::{Direction, Query};
use codegenome_identity::graph::query_context::LocalQueryContext;
use codegenome_identity::graph::traversal;

use crate::tools::inputs::ContextInput;
use crate::tools::CodegenomeTools;

impl CodegenomeTools {
    /// Retrieve context around a file:line via graph traversal.
    pub fn context(&self, input: &ContextInput) -> String {
        let Some((overlay, index)) = self.load_with_index() else {
            return r#"{"error":"no index found"}"#.into();
        };
        let Some(addr) = index.resolve(&input.file, input.line) else {
            return format!(
                r#"{{"error":"no symbol at {}:{}"}}"#,
                input.file, input.line
            );
        };

        let direction = parse_direction(&input.direction);
        let query = Query {
            target: addr,
            direction,
            max_depth: input.depth,
            min_confidence: 0.0,
            relation_filter: None,
        };
        let ctx = LocalQueryContext::new(overlay.nodes(), overlay.edges());
        let result = traversal::execute(&query, &ctx);

        let nodes: Vec<_> = result
            .nodes
            .iter()
            .map(|n| {
                serde_json::json!({
                    "kind": format!("{:?}", n.kind),
                    "confidence": n.confidence,
                    "provenance": format!("{:?}", n.provenance.source),
                    "span": n.span.as_ref().map(|s| format!("{}:{}", s.start_line, s.end_line)),
                })
            })
            .collect();

        let mut resp = serde_json::json!({
            "target": format!("{addr:?}"),
            "nodes": nodes,
            "edges": result.edges.len(),
            "paths": result.paths.len(),
        });
        resp["meta"] = self.response_meta();
        serde_json::to_string_pretty(&resp).unwrap_or_default()
    }
}

fn parse_direction(s: &str) -> Direction {
    match s {
        "upstream" => Direction::Upstream,
        "both" => Direction::Both,
        _ => Direction::Downstream,
    }
}
