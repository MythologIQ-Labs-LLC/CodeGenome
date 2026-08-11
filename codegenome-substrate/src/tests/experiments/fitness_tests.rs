use std::path::PathBuf;

use crate::experiments::config::ExperimentParams;
use crate::experiments::fitness;
use codegenome_identity::graph::overlay::Overlay;
use codegenome_identity::overlay::flow::FlowOverlay;
use codegenome_identity::overlay::semantic::SemanticOverlay;
use codegenome_identity::overlay::syntax::parse_rust_files;

fn src_dir() -> PathBuf {
    PathBuf::from(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../codegenome-identity"
    ))
    .join("src")
}

#[test]
fn impact_accuracy_baseline_non_degenerate() {
    let params = ExperimentParams::default();
    let accuracy = fitness::impact_accuracy(&src_dir(), &params);
    eprintln!("Impact accuracy baseline (3-layer): {accuracy:.4}");
    assert!(accuracy.is_finite(), "Accuracy must be finite");
}

#[test]
fn stability_baseline_above_threshold() {
    let params = ExperimentParams::default();
    let stab = fitness::stability(&src_dir(), &params);
    eprintln!("Stability baseline (3-layer): {stab:.4}");
    assert!(stab > 0.3, "Stability should be > 0.3");
    assert!(stab.is_finite(), "Stability must be finite");
}

#[test]
fn three_layer_fitness_differs_from_syntax_only() {
    let dir = src_dir();
    let files = collect_rs_files(&dir);
    assert!(!files.is_empty());

    let syntax = parse_rust_files(&files);
    let semantic = SemanticOverlay::from_syntax(&syntax, &files);
    let flow = FlowOverlay::from_source(&files);

    // Syntax-only: count edges reachable from first file
    let syntax_only: Vec<&dyn Overlay> = vec![&syntax];
    let three_layer: Vec<&dyn Overlay> = vec![&syntax, &semantic, &flow];

    let syntax_edges: usize = syntax_only.iter().map(|o| o.edges().len()).sum();
    let three_edges: usize = three_layer.iter().map(|o| o.edges().len()).sum();

    eprintln!("Syntax-only edges: {syntax_edges}");
    eprintln!("Three-layer edges: {three_edges}");

    assert!(
        three_edges > syntax_edges,
        "Three layers ({three_edges}) should have more edges than syntax-only ({syntax_edges})"
    );
}

#[test]
fn propagation_depth_produces_finite_score() {
    let params = ExperimentParams::default();
    let score = crate::experiments::fitness_fns::propagation_depth(&src_dir(), &params);
    eprintln!("PropagationDepth: {score:.4}");
    assert!(score.is_finite() && (0.0..=1.0).contains(&score));
}

#[test]
fn cycle_time_produces_finite_score() {
    let params = ExperimentParams::default();
    let score = crate::experiments::fitness_fns::cycle_time(&src_dir(), &params);
    eprintln!("CycleTime: {score:.4}");
    assert!(score.is_finite() && (0.0..=1.0).contains(&score));
}

#[test]
fn graph_density_produces_finite_score() {
    let params = ExperimentParams::default();
    let score = crate::experiments::fitness_fns::graph_density(&src_dir(), &params);
    eprintln!("GraphDensity: {score:.4}");
    assert!(score.is_finite() && (0.0..=1.0).contains(&score));
}

#[test]
fn graph_density_three_layer_greater_than_syntax() {
    let dir = src_dir();
    let files = collect_rs_files(&dir);
    let syntax = parse_rust_files(&files);
    let syntax_density = syntax.edges().len() as f64 / syntax.nodes().len().max(1) as f64;
    let three_density =
        crate::experiments::fitness_fns::graph_density(&dir, &ExperimentParams::default());
    eprintln!("Syntax density: {syntax_density:.4}, Three-layer: {three_density:.4}");
    assert!(three_density >= syntax_density.min(1.0));
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
