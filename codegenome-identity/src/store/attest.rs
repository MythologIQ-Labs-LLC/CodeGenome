//! In-toto attestation of index snapshots (backlog W4/W5, step 1).
//!
//! Emits an in-toto Statement (v1) whose subjects are the store's
//! artifact files with BLAKE3 digests, and whose predicate captures
//! the index metadata. This makes the graph snapshot tamper-evident:
//! any modification of stored overlays after attestation is detectable
//! by re-running verification — the substrate-level answer to the
//! agent-memory-poisoning threat model.
//!
//! Signing: the emitted statement is unsigned JSON. Wrapping it in a
//! Sigstore/cosign DSSE envelope is a release-pipeline concern
//! (`cosign attest --predicate ...`), deliberately kept out of the
//! library so local use needs no keys or OIDC.

use std::path::Path;

use crate::store::meta;

pub const STATEMENT_TYPE: &str = "https://in-toto.io/Statement/v1";
pub const PREDICATE_TYPE: &str = "urn:codegenome:attestation:index-snapshot:v1";

/// Build an in-toto Statement over every file in the store directory.
pub fn attest_store(store_dir: &Path) -> Result<serde_json::Value, String> {
    let subjects = store_subjects(store_dir)?;
    if subjects.is_empty() {
        return Err(format!(
            "no store artifacts found in {}",
            store_dir.display()
        ));
    }

    let mut predicate = serde_json::json!({
        "generator": format!("codegenome {}", env!("CARGO_PKG_VERSION")),
        "digestAlgorithm": "blake3",
        "storeDir": store_dir.display().to_string(),
    });
    if let Ok(Some(m)) = meta::load(store_dir) {
        predicate["index"] = serde_json::json!({
            "timestamp": m.timestamp,
            "fileCount": m.file_count,
            "nodeCount": m.node_count,
            "edgeCount": m.edge_count,
        });
    }

    Ok(serde_json::json!({
        "_type": STATEMENT_TYPE,
        "subject": subjects,
        "predicateType": PREDICATE_TYPE,
        "predicate": predicate,
    }))
}

/// Verify a previously emitted statement against the store on disk.
/// Returns the list of mismatched or missing subject names (empty =
/// verified).
pub fn verify_statement(
    statement: &serde_json::Value,
    store_dir: &Path,
) -> Result<Vec<String>, String> {
    let subjects = statement["subject"]
        .as_array()
        .ok_or("statement has no subject array")?;
    let mut failures = Vec::new();
    for subject in subjects {
        let name = subject["name"].as_str().unwrap_or_default();
        let expected = subject["digest"]["blake3"].as_str().unwrap_or_default();
        let path = store_dir.join(name);
        match std::fs::read(&path) {
            Ok(bytes) => {
                let actual = blake3::hash(&bytes).to_hex().to_string();
                if actual != expected {
                    failures.push(format!("{name}: digest mismatch"));
                }
            }
            Err(_) => failures.push(format!("{name}: missing")),
        }
    }
    Ok(failures)
}

fn store_subjects(store_dir: &Path) -> Result<Vec<serde_json::Value>, String> {
    let mut subjects = Vec::new();
    let entries = std::fs::read_dir(store_dir).map_err(|e| e.to_string())?;
    let mut paths: Vec<_> = entries
        .flatten()
        .map(|e| e.path())
        .filter(|p| p.is_file())
        .collect();
    paths.sort();
    for path in paths {
        let bytes = std::fs::read(&path).map_err(|e| e.to_string())?;
        let name = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or_default()
            .to_string();
        subjects.push(serde_json::json!({
            "name": name,
            "digest": { "blake3": blake3::hash(&bytes).to_hex().to_string() },
        }));
    }
    Ok(subjects)
}
