use crate::tools::CodegenomeTools;
use codegenome_identity::store::backend::StoreBackend;
use codegenome_identity::store::meta;
use codegenome_identity::store::ondisk::OnDiskStore;

impl CodegenomeTools {
    /// Report index status: overlay counts + freshness.
    pub fn status_report(&self, source_dir: &str) -> String {
        let src = if source_dir.is_empty() {
            "."
        } else {
            source_dir
        };
        let store = OnDiskStore::new(&self.store_dir);
        let freshness = meta::check_freshness(&self.store_dir, std::path::Path::new(src));

        let overlays: Vec<_> = store
            .list_overlays()
            .unwrap_or_default()
            .iter()
            .map(|k| {
                let (n, e) = store
                    .read_overlay(k)
                    .ok()
                    .flatten()
                    .map(|(nodes, edges)| (nodes.len(), edges.len()))
                    .unwrap_or((0, 0));
                serde_json::json!({"overlay": format!("{k:?}"), "nodes": n, "edges": e})
            })
            .collect();

        serde_json::json!({
            "fresh": freshness.is_fresh,
            "last_indexed": freshness.last_indexed,
            "files_changed": freshness.files_changed,
            "files_added": freshness.files_added,
            "files_removed": freshness.files_removed,
            "overlays": overlays,
        })
        .to_string()
    }
}
