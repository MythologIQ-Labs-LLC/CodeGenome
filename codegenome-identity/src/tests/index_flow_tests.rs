use std::path::PathBuf;

use crate::graph::edge::Relation;
use crate::index::flow::extract_flow;
use crate::index::flow_cfg::{extract_control_flow, CfgKind};
use crate::index::flow_dfg::extract_data_flow;

fn parse_tree(code: &[u8]) -> tree_sitter::Tree {
    let mut parser = tree_sitter::Parser::new();
    parser
        .set_language(&tree_sitter_rust::LANGUAGE.into())
        .unwrap();
    parser.parse(code, None).unwrap()
}

#[test]
fn if_else_produces_branch_cfg_edges() {
    let code = b"fn check(x: bool) { if x { let a = 1; } else { let b = 2; } }";
    let tree = parse_tree(code);
    let edges = extract_control_flow(code, &tree);

    let branches: Vec<_> = edges.iter().filter(|e| e.kind == CfgKind::Branch).collect();
    assert!(
        branches.len() >= 2,
        "Expected at least 2 Branch edges for if/else, got {}",
        branches.len()
    );
}

#[test]
fn let_and_use_produces_data_flow_edge() {
    let code = b"fn demo() { let x = 1; let y = x + 1; }";
    let tree = parse_tree(code);
    let edges = extract_data_flow(code, &tree);

    assert!(
        !edges.is_empty(),
        "Expected at least one DataFlow edge for let+use"
    );
    assert_eq!(edges[0].var_name, "x");
}

#[test]
fn extract_flow_produces_control_and_data_edges() {
    let code = r#"
fn demo() {
    let x = 1;
    let y = x + 1;
    if y > 0 { let z = 3; }
}
"#;
    let files = vec![(PathBuf::from("test.rs"), code.as_bytes().to_vec())];
    let result = extract_flow(&files);

    let cf: Vec<_> = result
        .edges
        .iter()
        .filter(|e| e.relation == Relation::ControlFlow)
        .collect();
    let df: Vec<_> = result
        .edges
        .iter()
        .filter(|e| e.relation == Relation::DataFlow)
        .collect();

    assert!(!cf.is_empty(), "Expected ControlFlow edges");
    assert!(!df.is_empty(), "Expected DataFlow edges");
}
