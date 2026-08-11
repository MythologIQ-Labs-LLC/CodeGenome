use crate::evidence::{self, EvidenceEntry, Operation};

fn sample_entry() -> EvidenceEntry {
    EvidenceEntry {
        timestamp: 1000,
        operation: Operation::Index,
        input_hash: "aaa".into(),
        output_hash: "bbb".into(),
        chain_hash: String::new(),
    }
}

#[test]
fn evidence_chain_is_deterministic() {
    let h1 = evidence::log_evidence(
        &temp_path("det1"),
        &sample_entry(),
        &evidence::genesis_hash(),
    )
    .unwrap();
    let h2 = evidence::log_evidence(
        &temp_path("det2"),
        &sample_entry(),
        &evidence::genesis_hash(),
    )
    .unwrap();
    assert_eq!(h1, h2);
}

#[test]
fn evidence_chain_verifies_clean() {
    let path = temp_path("clean");
    let mut prev = evidence::genesis_hash();
    for _ in 0..5 {
        prev = evidence::log_evidence(&path, &sample_entry(), &prev).unwrap();
    }
    let entries = evidence::read_evidence(&path).unwrap();
    assert_eq!(entries.len(), 5);
}

#[test]
fn tampered_evidence_fails() {
    let path = temp_path("tamper");
    let mut prev = evidence::genesis_hash();
    for _ in 0..3 {
        prev = evidence::log_evidence(&path, &sample_entry(), &prev).unwrap();
    }
    let content = std::fs::read_to_string(&path).unwrap();
    let tampered = content.replacen("aaa", "zzz", 1);
    std::fs::write(&path, tampered).unwrap();
    assert!(evidence::read_evidence(&path).is_err());
}

#[test]
fn evidence_genesis_is_stable() {
    assert_eq!(evidence::genesis_hash(), evidence::genesis_hash());
    assert!(!evidence::genesis_hash().is_empty());
}

fn temp_path(name: &str) -> std::path::PathBuf {
    let id = std::thread::current().id();
    let ts = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    let dir = std::env::temp_dir().join(format!("cg_ev_{name}_{id:?}_{ts}"));
    let _ = std::fs::remove_dir_all(&dir);
    let _ = std::fs::create_dir_all(&dir);
    dir.join("evidence.tsv")
}
