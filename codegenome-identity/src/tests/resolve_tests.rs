use crate::graph::overlay::Overlay;
use crate::graph::resolve::FileIndex;
use crate::index::parser::parse_files;
use crate::overlay::syntax::SyntaxOverlay;

fn build_index(code: &[(&str, &str)]) -> (FileIndex, SyntaxOverlay) {
    let dir = std::env::temp_dir().join("cg_resolve_test");
    let _ = std::fs::create_dir_all(&dir);
    let mut files = Vec::new();
    for (name, content) in code {
        let path = dir.join(name);
        std::fs::write(&path, content).unwrap();
        files.push((path, content.as_bytes().to_vec()));
    }
    let parsed = parse_files(&files);
    let syntax = SyntaxOverlay::from_parsed(&parsed);
    let index = FileIndex::build(&dir, syntax.nodes(), syntax.edges());
    let _ = std::fs::remove_dir_all(&dir);
    (index, syntax)
}

#[test]
fn resolve_known_file_and_line() {
    let (index, _syntax) = build_index(&[
        ("a.rs", "fn alpha() {}\nfn beta() {}"),
        ("b.rs", "fn gamma() {}"),
    ]);
    // alpha is at line 1, beta at line 2
    let addr = index.resolve("a.rs", 1);
    assert!(addr.is_some(), "Should resolve alpha at a.rs:1");
}

#[test]
fn suffix_match_works() {
    let (index, _) = build_index(&[("lib.rs", "fn foo() {}")]);
    // The file is stored as temp_dir/cg_resolve_test/lib.rs
    // but we query with just "lib.rs"
    let addr = index.resolve("lib.rs", 1);
    assert!(addr.is_some(), "Should resolve via suffix match");
}

#[test]
fn unknown_file_returns_none() {
    let (index, _) = build_index(&[("a.rs", "fn alpha() {}")]);
    let addr = index.resolve("nonexistent.rs", 1);
    assert!(addr.is_none());
}

#[test]
fn line_outside_span_returns_none() {
    let (index, _) = build_index(&[("a.rs", "fn alpha() {}")]);
    // alpha is at line 1, line 999 doesn't exist
    let addr = index.resolve("a.rs", 999);
    assert!(addr.is_none());
}
