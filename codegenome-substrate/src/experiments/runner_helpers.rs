use rand::Rng;

use crate::experiments::advisor;
use crate::experiments::config::*;
use crate::experiments::log::{self, ExperimentResult};
use crate::experiments::review::{self, Action, ReviewState};

pub fn apply_action(
    action: Action,
    params: &mut ExperimentParams,
    scale: f64,
    reviewer: &ReviewState,
    result: &mut ExperimentResult,
) -> (f64, Option<FitnessFunction>) {
    match action {
        Action::Continue => (scale, None),
        Action::WidenSearch(s) => {
            result.description = format!("widen: scale={s:.3}");
            (s, None)
        }
        Action::Restart => {
            *params = review::random_params();
            result.description = "RESTART: random params".into();
            (reviewer.base_scale(), None)
        }
        Action::SwitchFitness(ref name) => {
            let ff = parse_fitness_fn(name);
            result.description = format!("SWITCH_FITNESS: {name}");
            (reviewer.base_scale(), Some(ff))
        }
    }
}

pub fn maybe_consult_advisor(
    infra: &ExperimentInfra,
    log_path: &std::path::Path,
    fallback: Action,
) -> Action {
    let Some(model_id) = &infra.model_id else {
        return fallback;
    };
    let Ok(history) = log::read_log(log_path) else {
        return fallback;
    };
    let available = &[
        "ImpactAccuracy",
        "PropagationDepth",
        "CycleTime",
        "GraphDensity",
    ];
    advisor::advise(&history, model_id, available)
}

pub fn log_iteration(i: u64, label: &str, result: &ExperimentResult) {
    eprintln!(
        "[{i}] {label}: fitness={:.4} stability={:.4} ({} ms)",
        result.fitness, result.stability, result.cycle_time_ms,
    );
}

pub fn parse_fitness_fn(name: &str) -> FitnessFunction {
    match name {
        "ImpactAccuracy" => FitnessFunction::ImpactAccuracy,
        "PropagationDepth" => FitnessFunction::PropagationDepth,
        "CycleTime" => FitnessFunction::CycleTime,
        "GraphDensity" => FitnessFunction::GraphDensity,
        other => FitnessFunction::Custom(other.to_string()),
    }
}

pub fn perturb(params: &ExperimentParams, scale: f64) -> ExperimentParams {
    let mut rng = rand::rng();
    let mut new = params.clone();
    for (key, value) in new.values.iter_mut() {
        let delta: f64 = rng.random_range(-scale..=scale);
        *value += delta;
        match key.as_str() {
            "confidence_threshold" => *value = value.clamp(0.01, 0.99),
            "attenuation_factor" => *value = value.clamp(0.1, 2.0),
            "max_depth" => *value = value.clamp(1.0, 20.0),
            _ => {}
        }
    }
    new
}

pub fn format_params(params: &ExperimentParams) -> String {
    if params.values.is_empty() {
        return "default".into();
    }
    params
        .values
        .iter()
        .map(|(k, v)| format!("{k}={v:.3}"))
        .collect::<Vec<_>>()
        .join(", ")
}
