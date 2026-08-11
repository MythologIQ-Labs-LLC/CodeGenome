use std::path::PathBuf;

use crate::graph::edge::Relation;
use crate::graph::node::NodeKind;
use crate::graph::overlay::Overlay;
use crate::overlay::syntax::parse_rust_files;
use crate::store::backend::StoreBackend;
use crate::store::ondisk::OnDiskStore;

pub(crate) fn load_own_source() -> Vec<(PathBuf, Vec<u8>)> {
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

#[test]
fn self_parse_finds_address_of() {
    let digest_path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src/identity/digest.rs");
    let source = std::fs::read(&digest_path).unwrap();
    let files = vec![(digest_path, source)];
    let overlay = parse_rust_files(&files);

    let symbol_names: Vec<_> = overlay
        .nodes()
        .iter()
        .filter(|n| n.kind == NodeKind::Symbol)
        .collect();
    assert!(!symbol_names.is_empty(), "Should find symbols");
}

#[test]
fn each_file_produces_one_file_node() {
    let files = load_own_source();
    let file_count = files.len();
    let overlay = parse_rust_files(&files);

    let file_nodes = overlay
        .nodes()
        .iter()
        .filter(|n| n.kind == NodeKind::File)
        .count();
    assert_eq!(file_nodes, file_count);
}

#[test]
fn every_symbol_has_contains_edge() {
    let files = load_own_source();
    let overlay = parse_rust_files(&files);

    let symbol_addrs: std::collections::HashSet<_> = overlay
        .nodes()
        .iter()
        .filter(|n| n.kind == NodeKind::Symbol)
        .map(|n| n.address)
        .collect();

    let contained: std::collections::HashSet<_> = overlay
        .edges()
        .iter()
        .filter(|e| e.relation == Relation::Contains)
        .map(|e| e.target)
        .collect();

    for addr in &symbol_addrs {
        assert!(contained.contains(addr), "Symbol missing Contains edge");
    }
}

#[test]
fn parse_is_deterministic() {
    let files = load_own_source();
    let a = parse_rust_files(&files);
    let b = parse_rust_files(&files);

    let a_bytes = bincode::serialize(a.nodes()).unwrap();
    let b_bytes = bincode::serialize(b.nodes()).unwrap();
    assert_eq!(a_bytes, b_bytes, "Parse should be deterministic");
}

#[test]
fn symbols_have_spans() {
    let files = load_own_source();
    let overlay = parse_rust_files(&files);

    for node in overlay.nodes() {
        if node.kind == NodeKind::Symbol {
            assert!(node.span.is_some(), "Symbol nodes must have spans");
        }
    }
}

#[test]
fn store_roundtrip() {
    let dir = std::env::temp_dir().join("codegenome_test_store");
    let _ = std::fs::remove_dir_all(&dir);

    let files = load_own_source();
    let overlay = parse_rust_files(&files);

    let store = OnDiskStore::new(&dir);
    store
        .write_overlay(&overlay.kind(), overlay.nodes(), overlay.edges())
        .unwrap();

    let (read_nodes, read_edges) = store
        .read_overlay(&overlay.kind())
        .unwrap()
        .expect("Should read back overlay");

    assert_eq!(overlay.nodes(), &read_nodes[..]);
    assert_eq!(overlay.edges(), &read_edges[..]);

    let _ = std::fs::remove_dir_all(&dir);
}
