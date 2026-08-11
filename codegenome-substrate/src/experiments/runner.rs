use std::path::Path;

use crate::experiments::checkpoint::{self, Checkpoint};
use crate::experiments::config::*;
use crate::experiments::fitness;
use crate::experiments::log::{self, log_result, ExperimentResult};
use crate::experiments::review::{Action, ReviewState};
use crate::experiments::runner_helpers::*;

/// Run one experiment cycle. Returns fitness + stability.
pub fn run_experiment(
    infra: &ExperimentInfra,
    params: &ExperimentParams,
    fitness_fn: &FitnessFunction,
) -> ExperimentResult {
    let start = std::time::Instant::now();
    let accuracy = fitness::dispatch(fitness_fn, &infra.source_dir, params);
    let stab = fitness::stability(&infra.source_dir, params);
    let elapsed = start.elapsed();

    ExperimentResult {
        iteration: 0,
        params: params.values.clone(),
        fitness: accuracy,
        stability: stab,
        status: ExperimentStatus::Pass,
        cycle_time_ms: elapsed.as_millis() as u64,
        description: format_params(params),
        chain_hash: String::new(),
    }
}

/// Hill-climbing: perturb params, run, keep if better.
pub fn hill_climb_step(
    infra: &ExperimentInfra,
    current: &ExperimentParams,
    current_fitness: f64,
    current_stability: f64,
    perturbation_scale: f64,
) -> (ExperimentParams, ExperimentResult, bool) {
    let perturbed = perturb(current, perturbation_scale);
    let result = run_experiment(infra, &perturbed, &infra.fitness_fn);
    let dominated = result.fitness > current_fitness;
    let pareto =
        (result.fitness - current_fitness).abs() < 0.0001 && result.stability > current_stability;
    let kept = dominated || pareto;
    (perturbed, result, kept)
}

/// Mutable state for the continuous loop.
pub(crate) struct LoopState {
    pub params: ExperimentParams,
    pub fitness_fn: FitnessFunction,
    pub best_fitness: f64,
    pub best_stability: f64,
    pub scale: f64,
    pub reviewer: ReviewState,
    pub chain_hash: String,
}

/// Run experiments with adaptive review, checkpointing, and chain integrity.
pub fn run_continuous(
    infra: &ExperimentInfra,
    initial_params: ExperimentParams,
    log_path: &Path,
    max_iterations: Option<u64>,
) {
    let (mut state, start_iter) = init_or_resume(infra, initial_params, log_path);

    let limit = max_iterations.unwrap_or(u64::MAX);
    for i in start_iter..=limit {
        run_iteration(infra, &mut state, i, log_path);
        save_checkpoint(&state, i, log_path);
    }
}

fn init_or_resume(
    infra: &ExperimentInfra,
    initial_params: ExperimentParams,
    log_path: &Path,
) -> (LoopState, u64) {
    let cp_path = checkpoint::checkpoint_path(log_path);
    if let Ok(cp) = checkpoint::load(&cp_path) {
        if let Err(e) = log::read_log(log_path) {
            eprintln!("[ABORT] TSV integrity check failed: {e}");
            std::process::exit(1);
        }
        eprintln!("[RESUME] from iteration {}", cp.iteration);
        let state = restore_state(&cp);
        return (state, cp.iteration + 1);
    }
    let state = fresh_start(infra, initial_params, log_path);
    (state, 1)
}

fn fresh_start(
    infra: &ExperimentInfra,
    initial_params: ExperimentParams,
    log_path: &Path,
) -> LoopState {
    let reviewer = ReviewState::new(10, 3, 0.1);
    let mut state = LoopState {
        fitness_fn: infra.fitness_fn.clone(),
        scale: reviewer.base_scale(),
        params: initial_params,
        best_fitness: 0.0,
        best_stability: 0.0,
        reviewer,
        chain_hash: log::genesis_hash(),
    };
    let mut result = run_experiment(infra, &state.params, &state.fitness_fn);
    result.iteration = 0;
    result.description = "baseline".into();
    if let Ok(h) = log_result(log_path, &result, &state.chain_hash) {
        state.chain_hash = h;
    }
    state.best_fitness = result.fitness;
    state.best_stability = result.stability;
    state.reviewer.assess(result.fitness);
    log_iteration(0, "baseline", &result);
    state
}

fn restore_state(cp: &Checkpoint) -> LoopState {
    let reviewer = ReviewState::resume(
        10,
        3,
        0.1,
        cp.plateau_count,
        cp.widen_count,
        cp.best_fitness,
    );
    LoopState {
        params: ExperimentParams {
            values: cp.params.clone(),
        },
        fitness_fn: parse_fitness_fn(&cp.fitness_fn),
        best_fitness: cp.best_fitness,
        best_stability: cp.best_stability,
        scale: cp.scale,
        reviewer,
        chain_hash: cp.last_chain_hash.clone(),
    }
}

/// One iteration: hill-climb, review, adapt, log.
fn run_iteration(infra: &ExperimentInfra, state: &mut LoopState, i: u64, log_path: &Path) {
    let (perturbed, mut result, kept) = hill_climb_step(
        infra,
        &state.params,
        state.best_fitness,
        state.best_stability,
        state.scale,
    );
    result.iteration = i;

    let action = state.reviewer.assess(result.fitness);
    let action = if matches!(&action, Action::Restart) {
        maybe_consult_advisor(infra, log_path, action)
    } else {
        action
    };
    let (new_scale, new_ff) = apply_action(
        action,
        &mut state.params,
        state.scale,
        &state.reviewer,
        &mut result,
    );
    state.scale = new_scale;
    if let Some(ff) = new_ff {
        state.fitness_fn = ff;
    }

    if kept {
        state.params = perturbed;
        state.best_fitness = result.fitness;
        state.best_stability = result.stability;
        result.description = format!("KEEP: {}", format_params(&state.params));
    } else if result.description.is_empty() {
        result.description = "discard".into();
    }

    log_iteration(i, &result.description, &result);
    if let Ok(h) = log_result(log_path, &result, &state.chain_hash) {
        state.chain_hash = h;
    }
}

fn save_checkpoint(state: &LoopState, i: u64, log_path: &Path) {
    let cp = Checkpoint {
        iteration: i,
        params: state.params.values.clone(),
        fitness_fn: format!("{:?}", state.fitness_fn),
        best_fitness: state.best_fitness,
        best_stability: state.best_stability,
        scale: state.scale,
        plateau_count: state.reviewer.plateau_count(),
        widen_count: state.reviewer.widen_count(),
        last_chain_hash: state.chain_hash.clone(),
    };
    let cp_path = checkpoint::checkpoint_path(log_path);
    let _ = checkpoint::save(&cp_path, &cp);
}
