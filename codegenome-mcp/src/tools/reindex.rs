use crate::tools::gate;
use crate::tools::inputs::ReindexInput;
use crate::tools::CodegenomeTools;

impl CodegenomeTools {
    /// Write-gated re-index: check privilege, then run pipeline.
    pub fn reindex(&self, input: &ReindexInput) -> String {
        if let Err(reason) =
            gate::check_write_privilege(&self.source_dir, &self.store_dir, &input.actor)
        {
            return serde_json::json!({
                "error": "write denied",
                "reason": reason,
            })
            .to_string();
        }

        match codegenome_identity::index::run_pipeline(&self.source_dir, &self.store_dir) {
            Ok(r) if r.is_fresh => serde_json::json!({
                "status": "fresh",
                "files": r.file_count,
                "nodes": r.node_count,
                "edges": r.edge_count,
            })
            .to_string(),
            Ok(r) => serde_json::json!({
                "status": "reindexed",
                "actor": input.actor,
                "files": r.file_count,
                "nodes": r.node_count,
                "edges": r.edge_count,
                "elapsed_ms": r.elapsed_ms,
            })
            .to_string(),
            Err(e) => serde_json::json!({"error": e}).to_string(),
        }
    }
}
