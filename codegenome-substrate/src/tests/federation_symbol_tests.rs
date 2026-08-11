use std::collections::HashMap;
use std::path::PathBuf;

use crate::federation::symbol_resolve::{build_export_table, resolve_cross_repo};
use codegenome_identity::graph::edge::Relation;
use codegenome_identity::lang::rust::RustLanguage;

#[test]
fn cross_repo_import_produces_edge_at_0_7() {
    let lang = RustLanguage;
    // Exporter: defines pub fn helper()
    let exporter_files: Vec<(PathBuf, Vec<u8>)> = vec![(
        PathBuf::from("lib.rs"),
        b"pub fn helper() {}".to_vec(),
    )];
    let exports = build_export_table(&lang, &exporter_files);
    assert!(exports.contains_key("helper"));

    // Importer: imports helper
    let importer_files: Vec<(PathBuf, Vec<u8>)> = vec![(
        PathBuf::from("main.rs"),
        b"use crate_b::helper;".to_vec(),
    )];
    let edges = resolve_cross_repo(&lang, &importer_files, &exports);
    assert!(!edges.is_empty(), "Expected at least one cross-repo edge");
    assert_eq!(edges[0].relation, Relation::Imports);
    assert!((edges[0].confidence - 0.7).abs() < 0.01);
}

#[test]
fn missing_symbol_produces_no_edge() {
    let lang = RustLanguage;
    let exporter_files: Vec<(PathBuf, Vec<u8>)> = vec![(
        PathBuf::from("lib.rs"),
        b"pub fn other() {}".to_vec(),
    )];
    let exports = build_export_table(&lang, &exporter_files);

    let importer_files: Vec<(PathBuf, Vec<u8>)> = vec![(
        PathBuf::from("main.rs"),
        b"use crate_b::nonexistent;".to_vec(),
    )];
    let edges = resolve_cross_repo(&lang, &importer_files, &exports);
    assert!(edges.is_empty(), "No edge for missing symbol");
}

#[test]
fn empty_export_table_produces_no_edges() {
    let lang = RustLanguage;
    let exports: HashMap<String, codegenome_identity::identity::UorAddress> = HashMap::new();
    let importer_files: Vec<(PathBuf, Vec<u8>)> = vec![(
        PathBuf::from("main.rs"),
        b"use crate_b::anything;".to_vec(),
    )];
    let edges = resolve_cross_repo(&lang, &importer_files, &exports);
    assert!(edges.is_empty());
}
