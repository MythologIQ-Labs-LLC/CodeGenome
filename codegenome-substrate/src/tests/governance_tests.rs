use crate::governance::ledger::{self, LedgerEntry};
use crate::governance::policy::{Decision, PolicyContext, PolicyEngine};

fn sample_entry(seq: u64) -> LedgerEntry {
    LedgerEntry {
        sequence: seq,
        timestamp: 1000 + seq,
        operation: "index".into(),
        actor: "test".into(),
        input_hash: "aaa".into(),
        output_hash: "bbb".into(),
        chain_hash: String::new(),
    }
}

fn temp_path(name: &str) -> std::path::PathBuf {
    let id = std::thread::current().id();
    let ts = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    let dir = std::env::temp_dir().join(format!("cg_gov_{name}_{id:?}_{ts}"));
    let _ = std::fs::remove_dir_all(&dir);
    let _ = std::fs::create_dir_all(&dir);
    dir.join("ledger.tsv")
}

// Ledger tests

#[test]
fn ledger_append_and_read() {
    let path = temp_path("append");
    let mut prev = ledger::genesis_hash();
    for i in 0..5 {
        prev = ledger::append(&path, &sample_entry(i), &prev).unwrap();
    }
    let entries = ledger::read_ledger(&path).unwrap();
    assert_eq!(entries.len(), 5);
    assert_eq!(entries[0].sequence, 0);
    assert_eq!(entries[4].operation, "index");
}

#[test]
fn ledger_chain_verifies() {
    let path = temp_path("verify");
    let mut prev = ledger::genesis_hash();
    for i in 0..3 {
        prev = ledger::append(&path, &sample_entry(i), &prev).unwrap();
    }
    assert!(ledger::read_ledger(&path).is_ok());
}

#[test]
fn ledger_tamper_detected() {
    let path = temp_path("tamper");
    let mut prev = ledger::genesis_hash();
    for i in 0..3 {
        prev = ledger::append(&path, &sample_entry(i), &prev).unwrap();
    }
    let content = std::fs::read_to_string(&path).unwrap();
    let tampered = content.replacen("index", "HACKED", 1);
    std::fs::write(&path, tampered).unwrap();
    assert!(ledger::read_ledger(&path).is_err());
}

// Policy tests

fn ctx(op: &str, impact: usize, files: usize) -> PolicyContext {
    PolicyContext {
        operation: op.into(),
        impact_nodes: impact,
        changed_files: files,
    }
}

#[test]
fn default_policy_allows_all() {
    let engine = PolicyEngine::load(std::path::Path::new("nonexistent.toml")).unwrap();
    assert_eq!(engine.evaluate(&ctx("index", 100, 50)), Decision::Allow);
}

#[test]
fn deny_rule_blocks_operation() {
    let path = temp_path("deny").with_extension("toml");
    std::fs::write(
        &path,
        r#"
[[rules]]
operation = "index"
condition = "always"
action = "deny"
"#,
    )
    .unwrap();
    let engine = PolicyEngine::load(&path).unwrap();
    assert!(matches!(
        engine.evaluate(&ctx("index", 0, 0)),
        Decision::Deny(_)
    ));
}

#[test]
fn require_approval_on_high_impact() {
    let path = temp_path("approval").with_extension("toml");
    std::fs::write(
        &path,
        r#"
[[rules]]
operation = "query"
condition = "impact_nodes > 10"
action = "require-approval"
"#,
    )
    .unwrap();
    let engine = PolicyEngine::load(&path).unwrap();
    assert!(matches!(
        engine.evaluate(&ctx("query", 15, 0)),
        Decision::RequireApproval(_)
    ));
    assert_eq!(engine.evaluate(&ctx("query", 5, 0)), Decision::Allow);
}

#[test]
fn condition_always_matches() {
    let path = temp_path("always").with_extension("toml");
    std::fs::write(
        &path,
        r#"
[[rules]]
operation = "index"
condition = "always"
action = "allow"
"#,
    )
    .unwrap();
    let engine = PolicyEngine::load(&path).unwrap();
    assert_eq!(engine.evaluate(&ctx("index", 0, 0)), Decision::Allow);
}

#[test]
fn unmatched_operation_allows() {
    let path = temp_path("unmatched").with_extension("toml");
    std::fs::write(
        &path,
        r#"
[[rules]]
operation = "index"
condition = "always"
action = "deny"
"#,
    )
    .unwrap();
    let engine = PolicyEngine::load(&path).unwrap();
    assert_eq!(engine.evaluate(&ctx("query", 0, 0)), Decision::Allow);
}
