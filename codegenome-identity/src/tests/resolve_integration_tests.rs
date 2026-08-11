
use crate::graph::query::Query;
use crate::graph::resolve::FileIndex;
use crate::graph::traversal;
use crate::index::run_pipeline;
use crate::store::backend::StoreBackend;
use crate::store::ondisk::OnDiskStore;

#[test]
fn end_to_end_index_resolve_traverse() {
    let dir = std::env::temp_dir().join("cg_e2e_test");
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let store_dir = dir.join("store");

    // Write test source files
    std::fs::write(
        dir.join("lib.rs"),
        "fn helper() {}\n\nfn caller() { helper(); }\n",
    )
    .unwrap();

    // Index
    let result = run_pipeline(&dir, &store_dir).unwrap();
    assert!(result.node_count > 0);
    assert!(result.edge_count > 0);

    // Load overlay
    let store = OnDiskStore::new(&store_dir);
    let (nodes, edges) = store
        .read_overlay(&crate::graph::overlay::OverlayKind::Custom(
            "fused".into(),
        ))
        .unwrap()
        .unwrap();

    // Build FileIndex + resolve
    let index = FileIndex::build(&dir, &nodes, &edges);
    let addr = index.resolve("lib.rs", 1);
    assert!(addr.is_some(), "Should resolve helper at lib.rs:1");

    // Traverse from resolved address
    let query = Query::downstream(addr.unwrap(), 5);
    let ctx = crate::graph::query_context::LocalQueryContext::new(&nodes, &edges);
    let tr = traversal::execute(&query, &ctx);
    assert!(
        !tr.nodes.is_empty(),
        "Traversal should find reachable nodes"
    );

    // Verify provenance exists on result nodes
    for node in &tr.nodes {
        assert!(
            !node.provenance.actor.is_empty(),
            "Every node should have non-empty provenance actor"
        );
        assert!(
            node.confidence > 0.0,
            "Every node should have confidence > 0"
        );
    }

    let _ = std::fs::remove_dir_all(&dir);
}
