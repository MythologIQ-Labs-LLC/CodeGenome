use std::path::PathBuf;

use crate::graph::edge::Relation;
use crate::graph::overlay::Overlay;
use crate::overlay::flow::FlowOverlay;
use crate::overlay::semantic::SemanticOverlay;
use crate::overlay::syntax::parse_rust_files;
use crate::signal::impact::propagate_impact;

fn flow_from_snippet(code: &str) -> FlowOverlay {
    let files = vec![(PathBuf::from("test.rs"), code.as_bytes().to_vec())];
    FlowOverlay::from_source(&files)
}

#[test]
fn sequential_statements_produce_control_flow() {
    let flow = flow_from_snippet("fn f() { let a = 1; let b = 2; }");
    let cf: Vec<_> = flow
        .edges()
        .iter()
        .filter(|e| e.relation == Relation::ControlFlow)
        .collect();
    assert!(!cf.is_empty(), "Sequential statements should produce ControlFlow edges");
}

#[test]
fn if_expression_branches() {
    let code = r#"
fn f() {
    if true {
        let a = 1;
    } else {
        let b = 2;
    }
}
"#;
    let flow = flow_from_snippet(code);
    let cf: Vec<_> = flow
        .edges()
        .iter()
        .filter(|e| e.relation == Relation::ControlFlow)
        .collect();
    // if-else produces at least: entry→if, if→then, if→else
    assert!(cf.len() >= 2, "If-else should produce branch edges, got {}", cf.len());
}

#[test]
fn loop_produces_back_edge() {
    let code = r#"
fn f() {
    loop {
        let x = 1;
        break;
    }
}
"#;
    let flow = flow_from_snippet(code);
    let cf: Vec<_> = flow
        .edges()
        .iter()
        .filter(|e| e.relation == Relation::ControlFlow)
        .collect();
    assert!(!cf.is_empty(), "Loop should produce ControlFlow edges");
}

#[test]
fn match_produces_branch_per_arm() {
    let code = r#"
fn f(x: i32) {
    match x {
        1 => { let a = 1; }
        _ => { let b = 2; }
    }
}
"#;
    let flow = flow_from_snippet(code);
    let cf: Vec<_> = flow
        .edges()
        .iter()
        .filter(|e| e.relation == Relation::ControlFlow)
        .collect();
    assert!(cf.len() >= 2, "Match arms should produce branch edges, got {}", cf.len());
}

#[test]
fn let_binding_produces_data_flow() {
    let code = r#"
fn f() {
    let x = 1;
    let y = x;
}
"#;
    let flow = flow_from_snippet(code);
    let df: Vec<_> = flow
        .edges()
        .iter()
        .filter(|e| e.relation == Relation::DataFlow)
        .collect();
    assert!(!df.is_empty(), "Let binding with use should produce DataFlow edge");
}

#[test]
fn return_produces_exit_edge() {
    let code = r#"
fn f() -> i32 {
    return 1;
}
"#;
    let flow = flow_from_snippet(code);
    let cf: Vec<_> = flow
        .edges()
        .iter()
        .filter(|e| e.relation == Relation::ControlFlow)
        .collect();
    assert!(!cf.is_empty(), "Return should produce ControlFlow edge");
}

#[test]
fn self_index_flow_non_trivial() {
    let source_dir = std::path::Path::new("src");
    let files = collect_rust_files(source_dir);
    assert!(!files.is_empty());

    let flow = FlowOverlay::from_source(&files);
    let cf_count = flow
        .edges()
        .iter()
        .filter(|e| e.relation == Relation::ControlFlow)
        .count();
    let df_count = flow
        .edges()
        .iter()
        .filter(|e| e.relation == Relation::DataFlow)
        .count();

    eprintln!("--- Self-Index Flow Edges ---");
    eprintln!("  ControlFlow: {cf_count}");
    eprintln!("  DataFlow:    {df_count}");
    eprintln!("  Nodes:       {}", flow.nodes().len());
    eprintln!("  Total edges: {}", flow.edges().len());

    assert!(cf_count > 0, "Self-index should have ControlFlow edges");
    assert!(df_count > 0, "Self-index should have DataFlow edges");
}

#[test]
fn three_overlay_impact_propagation() {
    let code = r#"
fn callee() {
    let x = 1;
    let y = x;
}

fn caller() {
    callee();
}
"#;
    let files = vec![(PathBuf::from("test.rs"), code.as_bytes().to_vec())];
    let syntax = parse_rust_files(&files);
    let semantic = SemanticOverlay::from_syntax(&syntax, &files);
    let flow = FlowOverlay::from_source(&files);

    // Find a statement node inside callee
    let flow_node = flow
        .nodes()
        .iter()
        .find(|n| n.kind == crate::graph::node::NodeKind::Symbol)
        .expect("Flow overlay should have statement nodes");

    let overlays: Vec<&dyn Overlay> = vec![&syntax, &semantic, &flow];
    let impact = propagate_impact(&[flow_node.address], &overlays);

    assert!(
        !impact.is_empty(),
        "Impact should propagate across three overlays"
    );
}

fn collect_rust_files(dir: &std::path::Path) -> Vec<(PathBuf, Vec<u8>)> {
    let mut files = Vec::new();
    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                files.extend(collect_rust_files(&path));
            } else if path.extension().is_some_and(|e| e == "rs") {
                if let Ok(content) = std::fs::read(&path) {
                    files.push((path, content));
                }
            }
        }
    }
    files
}
