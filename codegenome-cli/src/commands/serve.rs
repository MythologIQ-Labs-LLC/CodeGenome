pub fn run(source_dir: &str, store_dir: &str) {
    let rt = tokio::runtime::Runtime::new().expect("Failed to create tokio runtime");
    if let Err(e) = rt.block_on(codegenome_mcp::server::run_stdio(
        source_dir.to_string(),
        store_dir.to_string(),
    )) {
        eprintln!("MCP server error: {e}");
        std::process::exit(1);
    }
}
