use std::path::Path;

use codegenome_identity::graph::export::to_cytoscape_filtered;
use codegenome_identity::graph::overlay::OverlayKind;
use codegenome_identity::store::backend::StoreBackend;
use codegenome_identity::store::ondisk::OnDiskStore;

const TEMPLATE: &str = include_str!("../../templates/graph.html");

pub fn run(store_dir: &str, output: &str, min_confidence: f64) {
    let store = OnDiskStore::new(Path::new(store_dir));
    let (nodes, edges) = match store.read_overlay(&OverlayKind::Custom("fused".into())) {
        Ok(Some(data)) => data,
        Ok(None) => {
            eprintln!("No fused overlay found. Run `codegenome index` first.");
            return;
        }
        Err(e) => {
            eprintln!("Error reading overlay: {e}");
            return;
        }
    };

    let graph_json = to_cytoscape_filtered(&nodes, &edges, min_confidence, None);
    let json_str = serde_json::to_string(&graph_json).unwrap_or_else(|_| "{}".into());

    let html = TEMPLATE.replace("{{GRAPH_DATA}}", &json_str);
    if let Err(e) = std::fs::write(output, html) {
        eprintln!("Failed to write {output}: {e}");
        return;
    }

    println!(
        "Wrote {} nodes, {} edges to {output}",
        nodes.len(),
        edges.len()
    );
}
