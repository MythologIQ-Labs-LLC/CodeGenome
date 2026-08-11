use std::path::PathBuf;

use crate::index::parser::parse_files_multi;
use crate::lang::all_languages;
use crate::lang::detect::group_by_language;

#[test]
fn mixed_language_files_produce_nodes_from_all() {
    let rust_src = b"fn hello() {}\nstruct World;".to_vec();
    let ts_src = b"function greet() {}\nclass App {}".to_vec();
    let py_src = b"def run():\n    pass\n\nclass Config:\n    pass\n".to_vec();

    let files: Vec<(PathBuf, Vec<u8>)> = vec![
        (PathBuf::from("lib.rs"), rust_src),
        (PathBuf::from("app.ts"), ts_src),
        (PathBuf::from("util.py"), py_src),
    ];

    let languages = all_languages();
    let groups = group_by_language(&files);

    assert!(groups.contains_key("rust"), "Rust group missing");
    assert!(groups.contains_key("typescript"), "TS group missing");
    assert!(groups.contains_key("python"), "Python group missing");

    let parsed = parse_files_multi(&groups, &languages);
    assert_eq!(parsed.len(), 3, "Expected 3 parsed files");

    let total_nodes: usize = parsed.iter().map(|p| p.nodes.len()).sum();
    // Each file produces at least 1 file node + symbol nodes
    assert!(
        total_nodes >= 3,
        "Expected at least 3 nodes (one per file), got {total_nodes}"
    );
}

#[test]
fn unsupported_extension_is_dropped() {
    let files: Vec<(PathBuf, Vec<u8>)> = vec![
        (PathBuf::from("readme.md"), b"# Hello".to_vec()),
        (PathBuf::from("lib.rs"), b"fn x() {}".to_vec()),
    ];

    let groups = group_by_language(&files);
    assert_eq!(groups.len(), 1, "Only Rust should be grouped");
    assert!(groups.contains_key("rust"));
}
