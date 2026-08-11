use std::path::PathBuf;

use codegenome_identity::graph::node::NodeKind;
use codegenome_identity::graph::overlay::Overlay;
use codegenome_identity::overlay::syntax::parse_rust_files;

fn load_all_source() -> Vec<(PathBuf, Vec<u8>)> {
    let src_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src");
    collect_rs_files(&src_dir)
}

fn collect_rs_files(dir: &std::path::Path) -> Vec<(PathBuf, Vec<u8>)> {
    let mut files = Vec::new();
    let Ok(entries) = std::fs::read_dir(dir) else {
        return files;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            files.extend(collect_rs_files(&path));
        } else if path.extension().is_some_and(|e| e == "rs") {
            if let Ok(content) = std::fs::read(&path) {
                files.push((path, content));
            }
        }
    }
    files
}

/// OQ8: Is self-referential indexing non-trivial?
#[test]
fn self_index_produces_non_trivial_graph() {
    let files = load_all_source();
    let overlay = parse_rust_files(&files);

    let total_nodes = overlay.nodes().len();
    let total_edges = overlay.edges().len();

    let file_count = overlay
        .nodes()
        .iter()
        .filter(|n| n.kind == NodeKind::File)
        .count();
    let symbol_count = overlay
        .nodes()
        .iter()
        .filter(|n| n.kind == NodeKind::Symbol)
        .count();

    // CODEGENOME is now ~1200+ LOC across 20+ files
    assert!(total_nodes > 50, "Expected >50 nodes, got {total_nodes}");
    assert!(total_edges > 30, "Expected >30 edges, got {total_edges}");
    assert!(file_count > 10, "Expected >10 files, got {file_count}");
    assert!(
        symbol_count > 20,
        "Expected >20 symbols, got {symbol_count}"
    );

    eprintln!("--- Self-Index Metrics ---");
    eprintln!("Total nodes: {total_nodes}");
    eprintln!("  Files:   {file_count}");
    eprintln!("  Symbols: {symbol_count}");
    eprintln!("Total edges: {total_edges}");
    eprintln!("--------------------------");
}

/// Baseline: measure cycle time for self-indexing.
#[test]
fn self_index_cycle_time_under_5_seconds() {
    let files = load_all_source();

    let start = std::time::Instant::now();
    let _overlay = parse_rust_files(&files);
    let elapsed = start.elapsed();

    eprintln!("Self-index cycle time: {:?}", elapsed);
    assert!(
        elapsed.as_secs() < 5,
        "Cycle time exceeded 5s: {:?}",
        elapsed,
    );
}
