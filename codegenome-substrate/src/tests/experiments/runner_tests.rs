use std::path::PathBuf;

use crate::experiments::config::*;
use crate::experiments::log;
use crate::experiments::runner;

fn test_infra() -> ExperimentInfra {
    ExperimentInfra {
        source_dir: PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src"),
        overlays: vec![codegenome_identity::graph::overlay::OverlayKind::Syntax],
        fitness_fn: FitnessFunction::GraphDensity,
        model_id: None,
    }
}

#[test]
fn single_experiment_produces_valid_result() {
    let infra = test_infra();
    let params = ExperimentParams::default();
    let result = runner::run_experiment(&infra, &params, &infra.fitness_fn);

    assert_eq!(result.status, ExperimentStatus::Pass);
    assert!(result.fitness < 1.0, "Fitness must not be trivially 1.0");
    assert!(result.cycle_time_ms < 10000);
}

#[test]
fn hill_climb_step_keeps_or_discards() {
    let infra = test_infra();
    let params = ExperimentParams::default();
    let baseline = runner::run_experiment(&infra, &params, &infra.fitness_fn);

    let (_, result, _kept) = runner::hill_climb_step(
        &infra,
        &params,
        &infra.fitness_fn,
        baseline.fitness,
        baseline.stability,
        0.1,
    );
    assert!(result.fitness.is_finite());
}

#[test]
fn tsv_logging_roundtrip() {
    let dir = std::env::temp_dir().join("codegenome_exp_test");
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let log_path = dir.join("results.tsv");

    let mut params = std::collections::HashMap::new();
    params.insert("max_depth".into(), 5.0);
    let result = log::ExperimentResult {
        iteration: 1,
        params,
        fitness: 0.876543,
        stability: 0.95,
        status: ExperimentStatus::Pass,
        cycle_time_ms: 42,
        description: "test run".into(),
        chain_hash: String::new(),
    };

    let h1 = log::log_result(&log_path, &result, &log::genesis_hash()).unwrap();
    let h2 = log::log_result(&log_path, &result, &h1).unwrap();
    let _ = log::log_result(&log_path, &result, &h2).unwrap();

    let results = log::read_log(&log_path).unwrap();
    assert_eq!(results.len(), 3);
    assert!((results[0].fitness - 0.876543).abs() < 0.001);
    assert_eq!(results[0].params.get("max_depth"), Some(&5.0));

    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn continuous_loop_runs_n_iterations() {
    let id = std::thread::current().id();
    let dir = std::env::temp_dir().join(format!("codegenome_loop_{id:?}"));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let log_path = dir.join("results.tsv");

    let infra = test_infra();
    let params = ExperimentParams::default();

    runner::run_continuous(&infra, params, &log_path, Some(3));

    let results = log::read_log(&log_path).unwrap();
    assert!(results.len() >= 3);

    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn adaptive_loop_runs_without_panic() {
    let id = std::thread::current().id();
    let dir = std::env::temp_dir().join(format!("codegenome_adaptive_{id:?}"));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let log_path = dir.join("results.tsv");

    let infra = test_infra();
    let params = ExperimentParams::default();

    runner::run_continuous(&infra, params, &log_path, Some(20));

    let results = log::read_log(&log_path).unwrap();
    assert!(results.len() >= 20, "Should have at least 20 results");

    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn run_experiment_carries_active_params() {
    let infra = test_infra();
    let params = ExperimentParams::default();
    let result = runner::run_experiment(&infra, &params, &infra.fitness_fn);
    assert_eq!(result.params, params.values);
    assert!(!result.params.is_empty());
}
