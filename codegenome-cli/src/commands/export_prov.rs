use std::path::Path;

use codegenome_identity::graph::overlay::OverlayKind;
use codegenome_identity::graph::prov::to_prov_json;
use codegenome_identity::store::backend::StoreBackend;
use codegenome_identity::store::ondisk::OnDiskStore;

pub fn run(store_dir: &str, output: &str) {
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

    let doc = to_prov_json(&nodes, &edges);
    let json = serde_json::to_string_pretty(&doc).unwrap_or_else(|_| "{}".into());
    if output == "-" {
        println!("{json}");
        return;
    }
    match std::fs::write(output, json) {
        Ok(()) => eprintln!(
            "PROV-JSON export: {} entities from {} nodes / {} edges -> {output}",
            doc["entity"].as_object().map(|o| o.len()).unwrap_or(0),
            nodes.len(),
            edges.len()
        ),
        Err(e) => eprintln!("Failed to write {output}: {e}"),
    }
}
