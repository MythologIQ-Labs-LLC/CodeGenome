use std::path::Path;

pub fn run(source_dir: &str, store_dir: &str) {
    match codegenome_identity::index::run_pipeline(Path::new(source_dir), Path::new(store_dir)) {
        Ok(result) if result.is_fresh => {
            println!(
                "Index is fresh ({} files, {} nodes, {} edges)",
                result.file_count, result.node_count, result.edge_count
            );
        }
        Ok(result) => {
            println!(
                "Indexed {} files → {} nodes, {} edges ({}ms)",
                result.file_count, result.node_count, result.edge_count, result.elapsed_ms
            );
        }
        Err(e) => eprintln!("Index failed: {e}"),
    }
}
