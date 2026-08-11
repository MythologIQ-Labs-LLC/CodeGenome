use std::path::Path;

use codegenome_identity::graph::overlay::OverlayKind;
use codegenome_identity::graph::repo_map::render_map_nodes;
use codegenome_identity::index::orchestrator::collect_source_files;
use codegenome_identity::store::backend::StoreBackend;
use codegenome_identity::store::ondisk::OnDiskStore;

pub fn run(source_dir: &str, store_dir: &str, output: &str) {
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

    let files = collect_source_files(Path::new(source_dir));
    let map = render_map_nodes(&files, &nodes, &edges);
    if output == "-" {
        println!("{map}");
        return;
    }
    match std::fs::write(output, &map) {
        Ok(()) => eprintln!("Repo map for {} files -> {output}", files.len()),
        Err(e) => eprintln!("Failed to write {output}: {e}"),
    }
}
