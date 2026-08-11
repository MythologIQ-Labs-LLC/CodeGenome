use crate::store::attest::{attest_store, verify_statement, PREDICATE_TYPE, STATEMENT_TYPE};

fn temp_store(name: &str) -> std::path::PathBuf {
    let dir = std::env::temp_dir().join(format!("codegenome-attest-{name}"));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

#[test]
fn attestation_covers_all_store_files_and_verifies() {
    let dir = temp_store("roundtrip");
    std::fs::write(dir.join("fused.bin"), b"overlay-bytes").unwrap();
    std::fs::write(dir.join("meta.json"), b"{}").unwrap();

    let statement = attest_store(&dir).unwrap();
    assert_eq!(statement["_type"], STATEMENT_TYPE);
    assert_eq!(statement["predicateType"], PREDICATE_TYPE);
    assert_eq!(statement["subject"].as_array().unwrap().len(), 2);

    let failures = verify_statement(&statement, &dir).unwrap();
    assert!(failures.is_empty(), "untouched store must verify");
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn tampering_is_detected() {
    let dir = temp_store("tamper");
    std::fs::write(dir.join("fused.bin"), b"original").unwrap();
    let statement = attest_store(&dir).unwrap();

    std::fs::write(dir.join("fused.bin"), b"poisoned").unwrap();
    let failures = verify_statement(&statement, &dir).unwrap();
    assert_eq!(failures.len(), 1);
    assert!(failures[0].contains("digest mismatch"));
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn missing_artifact_is_detected() {
    let dir = temp_store("missing");
    std::fs::write(dir.join("fused.bin"), b"original").unwrap();
    let statement = attest_store(&dir).unwrap();

    std::fs::remove_file(dir.join("fused.bin")).unwrap();
    let failures = verify_statement(&statement, &dir).unwrap();
    assert_eq!(failures.len(), 1);
    assert!(failures[0].contains("missing"));
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn empty_store_refuses_to_attest() {
    let dir = temp_store("empty");
    assert!(attest_store(&dir).is_err());
    let _ = std::fs::remove_dir_all(&dir);
}
