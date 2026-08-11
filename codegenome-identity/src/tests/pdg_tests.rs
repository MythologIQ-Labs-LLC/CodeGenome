use std::path::PathBuf;

use crate::graph::edge::Relation;
use crate::graph::overlay::Overlay;
use crate::overlay::flow::FlowOverlay;
use crate::overlay::pdg::PdgOverlay;

fn pdg_from_snippet(code: &str) -> PdgOverlay {
    let files = vec![(PathBuf::from("test.rs"), code.as_bytes().to_vec())];
    let flow = FlowOverlay::from_source(&files);
    PdgOverlay::from_flow(&flow)
}

#[test]
fn if_creates_control_dependence() {
    let pdg = pdg_from_snippet(
        r#"
fn f() {
    if true {
        let a = 1;
    } else {
        let b = 2;
    }
}
"#,
    );
    let cd: Vec<_> = pdg
        .edges()
        .iter()
        .filter(|e| e.relation == Relation::ControlDependence)
        .collect();
    assert!(
        !cd.is_empty(),
        "If-else should produce ControlDependence edges"
    );
}

#[test]
fn sequential_no_control_dependence() {
    let pdg = pdg_from_snippet("fn f() { let a = 1; let b = 2; }");
    let cd: Vec<_> = pdg
        .edges()
        .iter()
        .filter(|e| e.relation == Relation::ControlDependence)
        .collect();
    assert!(
        cd.is_empty(),
        "Sequential code should have no ControlDependence"
    );
}

#[test]
fn pdg_includes_data_flow() {
    let pdg = pdg_from_snippet("fn f() { let x = 1; let y = x; }");
    let df: Vec<_> = pdg
        .edges()
        .iter()
        .filter(|e| e.relation == Relation::DataFlow)
        .collect();
    assert!(!df.is_empty(), "PDG should include DataFlow edges");
}

#[test]
fn self_index_pdg_non_empty() {
    let files = collect_rs_files(std::path::Path::new("src"));
    let flow = FlowOverlay::from_source(&files);
    let pdg = PdgOverlay::from_flow(&flow);
    let cd = pdg
        .edges()
        .iter()
        .filter(|e| e.relation == Relation::ControlDependence)
        .count();
    eprintln!("Self-index PDG: {} ControlDependence edges", cd);
    assert!(cd > 0, "CODEGENOME should have ControlDependence edges");
}

fn collect_rs_files(dir: &std::path::Path) -> Vec<(PathBuf, Vec<u8>)> {
    let mut files = Vec::new();
    if let Ok(entries) = std::fs::read_dir(dir) {
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
    }
    files
}
