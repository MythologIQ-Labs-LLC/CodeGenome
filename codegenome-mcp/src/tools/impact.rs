use codegenome_identity::signal::impact::propagate_impact;

use crate::tools::inputs::ImpactInput;
use crate::tools::CodegenomeTools;

impl CodegenomeTools {
    /// Blast radius: propagate impact from a file:line location.
    pub fn impact(&self, input: &ImpactInput) -> String {
        let Some((overlay, index)) = self.load_with_index() else {
            return r#"{"error":"no index found"}"#.into();
        };
        let Some(addr) = index.resolve(&input.file, input.line) else {
            return format!(
                r#"{{"error":"no symbol at {}:{}"}}"#,
                input.file, input.line
            );
        };

        let overlays: Vec<&dyn codegenome_identity::graph::overlay::Overlay> = vec![&overlay];
        let impact = propagate_impact(&[addr], &overlays);
        let mut results: Vec<_> = impact.iter().filter(|(_, &s)| s > 0.01).collect();
        results.sort_by(|a, b| b.1.partial_cmp(a.1).unwrap_or(std::cmp::Ordering::Equal));

        let items: Vec<_> = results
            .iter()
            .take(50)
            .map(|(addr, score)| {
                let loc = overlay
                    .nodes
                    .iter()
                    .find(|n| n.address == **addr)
                    .and_then(|n| n.span.as_ref())
                    .map(|s| format!("line {}:{}", s.start_line, s.end_line))
                    .unwrap_or_else(|| format!("{addr:?}"));
                serde_json::json!({
                    "node": loc,
                    "impact_score": score,
                })
            })
            .collect();

        let mut resp = serde_json::json!({
            "source": format!("{}:{}", input.file, input.line),
            "affected_nodes": items,
        });
        resp["meta"] = self.response_meta();
        serde_json::to_string_pretty(&resp).unwrap_or_default()
    }
}
