
use crate::tools::gate::check_write_privilege;

#[test]
fn valid_actor_with_fresh_source_allowed() {
    // Use the codegenome-core source dir as a "fresh" repo
    // (store doesn't exist, so freshness check returns stale)
    // This tests the enforcement flow, not the policy itself
    let dir = std::env::temp_dir().join("cg_gate_test_allow");
    let _ = std::fs::create_dir_all(&dir);
    let store = dir.join("store");
    let _ = std::fs::create_dir_all(&store);

    // No meta.json → freshness is_fresh=false → Deny
    let result = check_write_privilege(
        &dir, &store, "claude-code",
    );
    assert!(
        result.is_err(),
        "Should deny: no index meta means stale"
    );
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn empty_actor_denied_with_provenance() {
    let dir = std::env::temp_dir().join("cg_gate_test_deny");
    let _ = std::fs::create_dir_all(&dir);
    let store = dir.join("store");
    let _ = std::fs::create_dir_all(&store);

    let result = check_write_privilege(&dir, &store, "");
    let _ = std::fs::remove_dir_all(&dir);

    assert!(result.is_err());
    let reason = result.unwrap_err();
    assert!(
        reason.contains("provenance"),
        "Should mention provenance, got: {reason}"
    );
}
