use std::path::Path;

/// Initialize CODEGENOME for a repository:
/// 1. Run initial index if store doesn't exist
/// 2. Write .mcp.json for Claude Code auto-discovery
pub fn run(source_dir: &str, store_dir: &str) {
    let src = Path::new(source_dir);
    let store = Path::new(store_dir);

    // Run initial index
    if !store.exists() {
        eprintln!("Indexing {source_dir} -> {store_dir}...");
        match codegenome_identity::index::run_pipeline(src, store) {
            Ok(r) => {
                eprintln!(
                    "Indexed: {} files, {} nodes, {} edges ({}ms)",
                    r.file_count, r.node_count, r.edge_count, r.elapsed_ms,
                );
            }
            Err(e) => {
                eprintln!("Index failed: {e}");
                std::process::exit(1);
            }
        }
    } else {
        eprintln!("Store exists at {store_dir}, skipping index.");
    }

    // Write .mcp.json
    let mcp_path = Path::new(".mcp.json");
    let mcp_content = serde_json::json!({
        "mcpServers": {
            "codegenome": {
                "command": "codegenome",
                "args": [
                    "serve",
                    "--source-dir", source_dir,
                    "--store-dir", store_dir
                ]
            }
        }
    });
    let json = serde_json::to_string_pretty(&mcp_content).expect("Failed to serialize .mcp.json");

    if let Err(e) = std::fs::write(mcp_path, json) {
        eprintln!("Failed to write .mcp.json: {e}");
        std::process::exit(1);
    }
    eprintln!("Created .mcp.json — Claude Code will auto-discover CODEGENOME tools.");
}
