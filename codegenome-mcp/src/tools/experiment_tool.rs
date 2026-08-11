use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use codegenome_identity::graph::overlay::OverlayKind;
use codegenome_substrate::experiments::config::*;
use codegenome_substrate::experiments::runner;

use crate::tools::{CodegenomeTools, RunHandle};

impl CodegenomeTools {
    /// Start an experiment loop on a background thread.
    pub fn experiment_start(&self, source_dir: &str, max_iterations: u64) -> String {
        let mut mgr = self.run_manager.lock().unwrap();
        if let Some(ref run) = mgr.active_run {
            if !run.completed.load(Ordering::Relaxed) {
                return serde_json::json!({
                    "error": "experiment already running",
                    "run_id": run.run_id,
                })
                .to_string();
            }
        }

        let run_id = format!(
            "{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs()
        );
        let log_path = self.store_dir.join(format!("experiment_{run_id}.tsv"));
        let completed = Arc::new(AtomicBool::new(false));
        let completed_clone = Arc::clone(&completed);
        let src = source_dir.to_string();
        let lp = log_path.clone();

        std::thread::spawn(move || {
            let infra = ExperimentInfra {
                source_dir: PathBuf::from(&src),
                overlays: vec![
                    OverlayKind::Syntax,
                    OverlayKind::Semantic,
                    OverlayKind::Flow,
                ],
                fitness_fn: FitnessFunction::ImpactAccuracy,
                model_id: None,
            };
            let params = ExperimentParams::default();
            runner::run_continuous(&infra, params, &lp, Some(max_iterations));
            completed_clone.store(true, Ordering::Relaxed);
        });

        mgr.active_run = Some(RunHandle {
            run_id: run_id.clone(),
            max_iterations,
            log_path,
            completed,
        });

        serde_json::json!({
            "run_id": run_id,
            "status": "started",
            "max_iterations": max_iterations,
        })
        .to_string()
    }

    /// Poll experiment status.
    pub fn experiment_status(&self) -> String {
        let mgr = self.run_manager.lock().unwrap();
        let Some(ref run) = mgr.active_run else {
            return serde_json::json!({"status": "idle"}).to_string();
        };

        let done = run.completed.load(Ordering::Relaxed);
        let iterations = count_tsv_rows(&run.log_path);
        let best = best_fitness(&run.log_path);

        serde_json::json!({
            "run_id": run.run_id,
            "status": if done { "completed" } else { "running" },
            "iterations": iterations,
            "best_fitness": best,
        })
        .to_string()
    }

    /// Read last N experiment results.
    pub fn experiment_results(&self, last_n: usize) -> String {
        let mgr = self.run_manager.lock().unwrap();
        let Some(ref run) = mgr.active_run else {
            return "[]".to_string();
        };
        let rows = tail_tsv(&run.log_path, last_n);
        serde_json::to_string(&rows).unwrap_or_else(|_| "[]".into())
    }
}

fn count_tsv_rows(path: &std::path::Path) -> usize {
    std::fs::read_to_string(path)
        .map(|s| s.lines().count().saturating_sub(1))
        .unwrap_or(0)
}

fn best_fitness(path: &std::path::Path) -> f64 {
    std::fs::read_to_string(path)
        .map(|s| {
            s.lines()
                .skip(1)
                .filter_map(|l| l.split('\t').nth(1)?.parse::<f64>().ok())
                .fold(0.0_f64, f64::max)
        })
        .unwrap_or(0.0)
}

fn tail_tsv(path: &std::path::Path, n: usize) -> Vec<serde_json::Value> {
    let Ok(content) = std::fs::read_to_string(path) else {
        return vec![];
    };
    let lines: Vec<&str> = content.lines().collect();
    lines
        .iter()
        .rev()
        .take(n)
        .rev()
        .filter_map(|line| {
            let p: Vec<&str> = line.split('\t').collect();
            if p.len() < 6 {
                return None;
            }
            Some(serde_json::json!({
                "iteration": p[0],
                "fitness": p[1],
                "stability": p[2],
                "cycle_ms": p[3],
                "description": p[5],
            }))
        })
        .collect()
}
