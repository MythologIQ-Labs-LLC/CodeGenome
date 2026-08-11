use std::path::PathBuf;

use crate::graph::repo_map::render_map_nodes;
use crate::index::parser::parse_files;
use crate::index::resolver::resolve;

const SOURCE: &str = "pub fn entry() { helper(); }\nfn helper() {}\n";

type SourceFiles = Vec<(PathBuf, Vec<u8>)>;
type Fixture = (
    SourceFiles,
    Vec<crate::graph::node::Node>,
    Vec<crate::graph::edge::Edge>,
);

fn fixture() -> Fixture {
    let files = vec![(PathBuf::from("src/lib.rs"), SOURCE.as_bytes().to_vec())];
    let parsed = parse_files(&files);
    let resolved = resolve(&parsed, &files);
    let mut nodes = Vec::new();
    let mut edges = Vec::new();
    for pf in &parsed {
        nodes.extend(pf.nodes.clone());
        edges.extend(pf.edges.clone());
    }
    edges.extend(resolved.edges);
    (files, nodes, edges)
}

#[test]
fn map_lists_files_and_signature_lines() {
    let (files, nodes, edges) = fixture();
    let map = render_map_nodes(&files, &nodes, &edges);
    assert!(map.contains("## src/lib.rs"));
    assert!(map.contains("pub fn entry()"), "map: {map}");
    assert!(map.contains("fn helper()"));
    assert!(map.contains("(L1"), "line numbers present");
}

#[test]
fn map_carries_call_fan_in_out() {
    let (files, nodes, edges) = fixture();
    let map = render_map_nodes(&files, &nodes, &edges);
    // entry calls helper -> helper has fan-in, entry has fan-out
    assert!(map.contains("out:1"), "caller fan-out shown: {map}");
    assert!(map.contains("in:1"), "callee fan-in shown: {map}");
}

#[test]
fn unindexed_file_is_listed_without_symbols() {
    let files = vec![(PathBuf::from("src/other.rs"), b"fn x() {}".to_vec())];
    let map = render_map_nodes(&files, &[], &[]);
    assert!(map.contains("## src/other.rs"));
    assert!(map.contains("no symbols indexed"));
}
