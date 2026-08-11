use std::path::Path;

use codegenome_identity::store::meta;
use codegenome_substrate::governance::policy::Decision;
use codegenome_substrate::governance::write_gate::{WriteGatePolicy, WriteRequest};

/// Check write privilege at the MCP boundary.
/// Returns Ok(()) if allowed, Err(reason) if denied.
pub fn check_write_privilege(
    source_dir: &Path,
    store_dir: &Path,
    actor: &str,
) -> Result<(), String> {
    let freshness = meta::check_freshness(store_dir, source_dir);

    let request = WriteRequest {
        actor: actor.to_string(),
        toolchain_version: env!("CARGO_PKG_VERSION").to_string(),
        source_freshness: freshness,
        min_edge_confidence: 1.0, // reindex produces full-confidence edges
    };

    let policy = WriteGatePolicy::default_policy();
    match policy.evaluate(&request) {
        Decision::Allow => Ok(()),
        Decision::Deny(reason) => Err(reason),
        Decision::RequireApproval(reason) => Err(format!("approval required: {reason}")),
    }
}
