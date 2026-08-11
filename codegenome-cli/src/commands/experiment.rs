use std::path::{Path, PathBuf};

use codegenome_identity::graph::overlay::OverlayKind;
use codegenome_substrate::experiments::config::*;
use codegenome_substrate::experiments::runner;

pub fn run(source_dir: &str, log_file: &str, max_iterations: Option<u64>, model: Option<String>) {
    let infra = ExperimentInfra {
        source_dir: PathBuf::from(source_dir),
        overlays: vec![
            OverlayKind::Syntax,
            OverlayKind::Semantic,
            OverlayKind::Flow,
        ],
        fitness_fn: FitnessFunction::ImpactAccuracy,
        model_id: model.clone(),
    };
    let params = ExperimentParams::default();

    println!("=== CODEGENOME Experiment Loop ===");
    println!("Source: {source_dir}");
    println!("Log:    {log_file}");
    match max_iterations {
        Some(n) => println!("Iterations: {n}"),
        None => println!("Iterations: infinite (Ctrl+C to stop)"),
    }
    match &model {
        Some(m) => println!("Model:  {m} (Tier 2 enabled)"),
        None => println!("Model:  none (Tier 1.5 only)"),
    }
    println!("==================================");

    runner::run_continuous(&infra, params, Path::new(log_file), max_iterations);

    println!("Experiment loop complete.");
}
