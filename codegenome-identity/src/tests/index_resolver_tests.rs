use std::path::PathBuf;

use crate::graph::edge::Relation;
use crate::index::parser::parse_files;
use crate::index::resolver::resolve;

#[test]
fn use_declaration_produces_imports_edge() {
    let files = vec![
        (PathBuf::from("lib.rs"), b"pub fn helper() {}".to_vec()),
        (
            PathBuf::from("main.rs"),
            b"use crate::helper;\nfn main() {}".to_vec(),
        ),
    ];
    let parsed = parse_files(&files);
    let resolved = resolve(&parsed, &files);

    let imports: Vec<_> = resolved
        .edges
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
fn cross_file_call_produces_calls_edge() {
    let code = r#"
fn helper() {}

fn main() {
    helper();
}
"#;
    let files = vec![(PathBuf::from("test.rs"), code.as_bytes().to_vec())];
    let parsed = parse_files(&files);
    let resolved = resolve(&parsed, &files);

    let calls: Vec<_> = resolved
        .edges
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
trait Greet {
    fn hello(&self);
}

struct Bot;

impl Greet for Bot {
    fn hello(&self) {}
}
"#;
    let files = vec![(PathBuf::from("test.rs"), code.as_bytes().to_vec())];
    let parsed = parse_files(&files);
    let resolved = resolve(&parsed, &files);

    let impls: Vec<_> = resolved
        .edges
        .iter()
        .filter(|e| e.relation == Relation::Implements)
        .collect();
    assert!(!impls.is_empty(), "Expected at least one Implements edge");
}
