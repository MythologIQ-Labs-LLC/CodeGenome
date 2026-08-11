use std::path::PathBuf;

use crate::graph::edge::Relation;
use crate::graph::node::NodeKind;
use crate::index::parser::{parse_files, ParsedFile};

fn parse_snippet(code: &str) -> Vec<ParsedFile> {
    let files = vec![(PathBuf::from("test.rs"), code.as_bytes().to_vec())];
    parse_files(&files)
}

#[test]
fn three_functions_produce_four_nodes() {
    let code = r#"
fn alpha() {}
fn beta() {}
fn gamma() {}
"#;
    let parsed = parse_snippet(code);
    assert_eq!(parsed.len(), 1);
    let pf = &parsed[0];
    // 1 File node + 3 Symbol nodes
    assert_eq!(pf.nodes.len(), 4);
    let file_nodes: Vec<_> = pf
        .nodes
        .iter()
        .filter(|n| n.kind == NodeKind::File)
        .collect();
    assert_eq!(file_nodes.len(), 1);
    let symbol_nodes: Vec<_> = pf
        .nodes
        .iter()
        .filter(|n| n.kind == NodeKind::Symbol)
        .collect();
    assert_eq!(symbol_nodes.len(), 3);
}

#[test]
fn three_functions_produce_three_contains_edges() {
    let code = r#"
fn alpha() {}
fn beta() {}
fn gamma() {}
"#;
    let parsed = parse_snippet(code);
    let pf = &parsed[0];
    let contains: Vec<_> = pf
        .edges
        .iter()
        .filter(|e| e.relation == Relation::Contains)
        .collect();
    assert_eq!(contains.len(), 3);
}

#[test]
fn content_hash_is_deterministic() {
    let code = "fn foo() {}";
    let a = parse_snippet(code);
    let b = parse_snippet(code);
    assert_eq!(a[0].content_hash, b[0].content_hash);
}

#[test]
fn parser_reuse_across_files() {
    let files = vec![
        (PathBuf::from("a.rs"), b"fn a() {}".to_vec()),
        (PathBuf::from("b.rs"), b"fn b() {}".to_vec()),
    ];
    let parsed = parse_files(&files);
    assert_eq!(parsed.len(), 2);
    assert_ne!(parsed[0].file_address, parsed[1].file_address);
}
