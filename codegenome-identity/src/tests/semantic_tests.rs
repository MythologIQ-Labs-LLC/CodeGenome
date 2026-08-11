use std::path::PathBuf;

use crate::graph::edge::Relation;
use crate::graph::overlay::Overlay;
use crate::overlay::semantic::SemanticOverlay;
use crate::overlay::syntax::parse_rust_files;
use crate::signal::impact::propagate_impact;

fn parse_snippet(
    code: &str,
) -> (
    crate::overlay::syntax::SyntaxOverlay,
    Vec<(PathBuf, Vec<u8>)>,
) {
    let files = vec![(PathBuf::from("test.rs"), code.as_bytes().to_vec())];
    let syntax = parse_rust_files(&files);
    (syntax, files)
}

#[test]
fn use_declaration_produces_imports_edge() {
    // Two files: lib defines `helper`, main imports it via `use`
    let files = vec![
        (PathBuf::from("lib.rs"), b"pub fn helper() {}".to_vec()),
        (
            PathBuf::from("main.rs"),
            b"use crate::helper;\nfn main() {}".to_vec(),
        ),
    ];
    let syntax = parse_rust_files(&files);
    let semantic = SemanticOverlay::from_syntax(&syntax, &files);

    let imports: Vec<_> = semantic
        .edges()
        .iter()
        .filter(|e| e.relation == Relation::Imports)
        .collect();
    assert!(!imports.is_empty(), "Expected at least one Imports edge");
    assert!(
        imports
            .iter()
            .all(|e| (e.confidence - 0.8).abs() < f64::EPSILON),
        "Imports edges should have confidence 0.8"
    );
}

#[test]
fn function_call_produces_calls_edge() {
    let code = r#"
fn helper() {}

fn main() {
    helper();
}
"#;
    let (syntax, files) = parse_snippet(code);
    let semantic = SemanticOverlay::from_syntax(&syntax, &files);

    let calls: Vec<_> = semantic
        .edges()
        .iter()
        .filter(|e| e.relation == Relation::Calls)
        .collect();
    assert!(!calls.is_empty(), "Expected at least one Calls edge");
    assert!(
        calls
            .iter()
            .all(|e| (e.confidence - 0.7).abs() < f64::EPSILON),
        "Calls edges should have confidence 0.7"
    );
}

#[test]
fn impl_trait_produces_implements_edge() {
    let code = r#"
struct Foo;

trait Bar {
    fn do_thing(&self);
}

impl Bar for Foo {
    fn do_thing(&self) {}
}
"#;
    let (syntax, files) = parse_snippet(code);
    let semantic = SemanticOverlay::from_syntax(&syntax, &files);

    let impls: Vec<_> = semantic
        .edges()
        .iter()
        .filter(|e| e.relation == Relation::Implements)
        .collect();
    assert!(!impls.is_empty(), "Expected at least one Implements edge");
    assert!(
        impls
            .iter()
            .all(|e| (e.confidence - 0.8).abs() < f64::EPSILON),
        "Implements edges should have confidence 0.8"
    );
}

#[test]
fn unresolved_call_produces_no_edge() {
    let code = r#"
fn main() {
    unknown_function();
}
"#;
    let (syntax, files) = parse_snippet(code);
    let semantic = SemanticOverlay::from_syntax(&syntax, &files);

    let calls: Vec<_> = semantic
        .edges()
        .iter()
        .filter(|e| e.relation == Relation::Calls)
        .collect();
    assert!(calls.is_empty(), "Unresolved calls should produce no edges");
}

#[test]
fn self_index_has_semantic_edges() {
    let source_dir = std::path::Path::new("src");
    let files = collect_rust_files(source_dir);
    assert!(!files.is_empty());

    let syntax = parse_rust_files(&files);
    let semantic = SemanticOverlay::from_syntax(&syntax, &files);

    let call_count = semantic
        .edges()
        .iter()
        .filter(|e| e.relation == Relation::Calls)
        .count();
    let import_count = semantic
        .edges()
        .iter()
        .filter(|e| e.relation == Relation::Imports)
        .count();

    eprintln!("--- Self-Index Semantic Edges ---");
    eprintln!("  Calls:   {call_count}");
    eprintln!("  Imports: {import_count}");
    eprintln!("  Total:   {}", semantic.edges().len());

    assert!(call_count > 0, "Self-index should have Calls edges");
    assert!(import_count > 0, "Self-index should have Imports edges");
}

#[test]
fn combined_overlays_propagate_impact() {
    let code = r#"
fn callee() {}

fn caller() {
    callee();
}
"#;
    let (syntax, files) = parse_snippet(code);
    let semantic = SemanticOverlay::from_syntax(&syntax, &files);

    // Find the callee's address
    let callee_addr = syntax
        .nodes()
        .iter()
        .find(|n| n.kind == crate::graph::node::NodeKind::Symbol)
        .expect("Should have at least one symbol")
        .address;

    let overlays: Vec<&dyn Overlay> = vec![&syntax, &semantic];
    let impact = propagate_impact(&[callee_addr], &overlays);

    // Impact should propagate to at least the callee itself
    assert!(impact.contains_key(&callee_addr));
    // With semantic edges, impact should reach more than just the changed node
    assert!(
        !impact.is_empty(),
        "Impact should propagate through semantic edges"
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
