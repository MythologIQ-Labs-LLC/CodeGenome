use std::path::PathBuf;

use crate::graph::edge::Relation;
use crate::graph::overlay::Overlay;
use crate::overlay::runtime::RuntimeOverlay;

fn write_trace(path: &std::path::Path, lines: &[&str]) {
    let mut content = "caller\tcallee\tcount\tduration_ns\n".to_string();
    for line in lines {
        content.push_str(line);
        content.push('\n');
    }
    std::fs::write(path, content).unwrap();
}

fn source_files() -> Vec<(PathBuf, Vec<u8>)> {
    let code = b"fn caller_fn() {}\nfn callee_fn() {}";
    vec![(PathBuf::from("test.rs"), code.to_vec())]
}

#[test]
fn parse_trace_produces_calls_edges() {
    let dir = temp_dir("parse");
    let trace = dir.join("trace.tsv");
    write_trace(&trace, &["caller_fn\tcallee_fn\t50\t1000"]);
    let files = source_files();
    let overlay = RuntimeOverlay::from_trace_file(&trace, &files).unwrap();
    let calls: Vec<_> = overlay
        .edges()
        .iter()
        .filter(|e| e.relation == Relation::Calls)
        .collect();
    assert!(!calls.is_empty(), "Trace should produce Calls edges");
}

#[test]
fn high_count_gives_high_confidence() {
    let dir = temp_dir("high");
    let trace = dir.join("trace.tsv");
    write_trace(&trace, &["caller_fn\tcallee_fn\t100\t1000"]);
    let files = source_files();
    let overlay = RuntimeOverlay::from_trace_file(&trace, &files).unwrap();
    let edge = overlay.edges().first().unwrap();
    assert!(
        (edge.confidence - 1.0).abs() < 0.01,
        "100 calls should give confidence 1.0"
    );
}

#[test]
fn low_count_gives_low_confidence() {
    let dir = temp_dir("low");
    let trace = dir.join("trace.tsv");
    write_trace(&trace, &["caller_fn\tcallee_fn\t2\t1000"]);
    let files = source_files();
    let overlay = RuntimeOverlay::from_trace_file(&trace, &files).unwrap();
    let edge = overlay.edges().first().unwrap();
    assert!(
        (edge.confidence - 0.2).abs() < 0.01,
        "2 calls should give confidence 0.2"
    );
}

#[test]
fn unresolved_names_skipped() {
    let dir = temp_dir("unresolved");
    let trace = dir.join("trace.tsv");
    write_trace(&trace, &["nonexistent\talso_nonexistent\t10\t1000"]);
    let files = source_files();
    let overlay = RuntimeOverlay::from_trace_file(&trace, &files).unwrap();
    assert!(
        overlay.edges().is_empty(),
        "Unresolved names should produce no edges"
    );
}

fn temp_dir(name: &str) -> PathBuf {
    let id = std::thread::current().id();
    let ts = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    let dir = std::env::temp_dir().join(format!("cg_rt_{name}_{id:?}_{ts}"));
    let _ = std::fs::create_dir_all(&dir);
    dir
}
