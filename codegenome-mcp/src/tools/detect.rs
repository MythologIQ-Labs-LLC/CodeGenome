use codegenome_identity::diff::detect_changes;
use codegenome_identity::diff::git_bridge;
use codegenome_identity::graph::overlay::Overlay;

use crate::tools::inputs::DetectInput;
use crate::tools::CodegenomeTools;

impl CodegenomeTools {
    /// Map git diff to affected symbols + blast radius with
    /// process-level impact propagation.
    pub fn detect(&self, input: &DetectInput) -> String {
        let Some(overlay) = self.load_overlay() else {
            return r#"{"error":"no index found"}"#.into();
        };

        let diff = match git_bridge::git_diff(
            &self.source_dir,
            Some(&input.from_ref),
            input.to_ref.as_deref(),
        ) {
            Ok(d) => d,
            Err(e) => return serde_json::json!({"error": e}).to_string(),
        };

        let overlays: Vec<&dyn Overlay> = vec![&overlay];
        let changeset = detect_changes(&diff, &overlays);

        let blast_radius = blast_radius_json(&changeset.impact, &overlay);

        let mut resp = serde_json::json!({
            "from_ref": input.from_ref,
            "to_ref": input.to_ref.as_deref().unwrap_or("workdir"),
            "changed_nodes": changeset.changed_nodes.len(),
            "affected_edges": changeset.affected_edges.len(),
            "impact_nodes": changeset.impact.len(),
            "staleness_nodes": changeset.staleness.len(),
            "blast_radius": blast_radius,
        });
        resp["meta"] = self.response_meta();
        serde_json::to_string_pretty(&resp).unwrap_or_default()
    }
}

fn blast_radius_json(
    impact: &std::collections::HashMap<codegenome_identity::identity::UorAddress, f64>,
    overlay: &crate::tools::StoredOverlay,
) -> Vec<serde_json::Value> {
    let mut items: Vec<_> = impact.iter().filter(|(_, &score)| score > 0.01).collect();
    items.sort_by(|a, b| b.1.partial_cmp(a.1).unwrap_or(std::cmp::Ordering::Equal));

    items
        .iter()
        .take(50)
        .map(|(addr, score)| {
            let node = overlay.nodes.iter().find(|n| n.address == **addr);
            let loc = node
                .and_then(|n| n.span.as_ref())
                .map(|s| format!("line {}:{}", s.start_line, s.end_line))
                .unwrap_or_else(|| format!("{addr:?}"));
            let kind = node
                .map(|n| format!("{:?}", n.kind))
                .unwrap_or_else(|| "Unknown".into());
            serde_json::json!({
                "node": loc,
                "kind": kind,
                "impact_score": score,
            })
        })
        .collect()
}
