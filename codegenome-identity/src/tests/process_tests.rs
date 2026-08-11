use std::path::PathBuf;

use crate::graph::edge::Relation;
use crate::graph::node::NodeKind;
use crate::graph::overlay::Overlay;
use crate::overlay::process::ProcessOverlay;
use crate::overlay::semantic::SemanticOverlay;
use crate::overlay::syntax::parse_rust_files;

fn build_process(code: &str) -> ProcessOverlay {
    let files = vec![(PathBuf::from("test.rs"), code.as_bytes().to_vec())];
    let syntax = parse_rust_files(&files);
    let semantic = SemanticOverlay::from_syntax(&syntax, &files);
    ProcessOverlay::from_semantic(&semantic, &syntax, &files)
}

#[test]
fn main_detected_as_entrypoint() {
    let overlay = build_process("fn main() {}");
    let procs: Vec<_> = overlay
        .nodes()
        .iter()
        .filter(|n| n.kind == NodeKind::Process)
        .collect();
    assert!(!procs.is_empty(), "main should be detected as entrypoint");
}

#[test]
fn test_fn_detected_as_entrypoint() {
    let overlay = build_process("#[test]\nfn my_test() {}");
    let procs: Vec<_> = overlay
        .nodes()
        .iter()
        .filter(|n| n.kind == NodeKind::Process)
        .collect();
    assert!(
        !procs.is_empty(),
        "test fn should be detected as entrypoint"
    );
}

#[test]
fn pub_fn_detected_as_entrypoint() {
    let overlay = build_process("pub fn api_call() {}");
    let procs: Vec<_> = overlay
        .nodes()
        .iter()
        .filter(|n| n.kind == NodeKind::Process)
        .collect();
    assert!(!procs.is_empty(), "pub fn should be detected as entrypoint");
}

#[test]
fn private_fn_not_entrypoint() {
    let overlay = build_process("fn helper() {}");
    let procs: Vec<_> = overlay
        .nodes()
        .iter()
        .filter(|n| n.kind == NodeKind::Process)
        .collect();
    assert!(procs.is_empty(), "private fn should NOT be an entrypoint");
}

#[test]
fn process_traces_call_chain() {
    let code = r#"
pub fn entry() { middle(); }
fn middle() { leaf(); }
fn leaf() {}
"#;
    let overlay = build_process(code);
    let pop_edges: Vec<_> = overlay
        .edges()
        .iter()
        .filter(|e| e.relation == Relation::PartOfProcess)
        .collect();
    // entry → entry symbol, entry → middle, entry → leaf = 3 edges
    assert!(
        pop_edges.len() >= 2,
        "Should trace at least 2 PartOfProcess edges, got {}",
        pop_edges.len()
    );
}

#[test]
fn self_index_has_process_nodes() {
    let files = collect_rs_files(std::path::Path::new("src"));
    let syntax = parse_rust_files(&files);
    let semantic = SemanticOverlay::from_syntax(&syntax, &files);
    let overlay = ProcessOverlay::from_semantic(&semantic, &syntax, &files);

    let proc_count = overlay
        .nodes()
        .iter()
        .filter(|n| n.kind == NodeKind::Process)
        .count();
    eprintln!("Self-index process nodes: {proc_count}");
    assert!(
        proc_count > 0,
        "CODEGENOME should have at least one entrypoint"
    );
}

fn collect_rs_files(dir: &std::path::Path) -> Vec<(PathBuf, Vec<u8>)> {
    let mut files = Vec::new();
    if let Ok(entries) = std::fs::read_dir(dir) {
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
    }
    files
}
