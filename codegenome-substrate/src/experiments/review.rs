use rand::Rng;

use crate::experiments::config::ExperimentParams;

/// Adaptive action emitted by the reviewer.
#[derive(Clone, Debug)]
pub enum Action {
    Continue,
    WidenSearch(f64),
    Restart,
    SwitchFitness(String),
}

/// Tracks experiment history and detects plateaus.
/// Pure state machine: accumulates fitness, emits actions.
pub struct ReviewState {
    plateau_count: u32,
    plateau_threshold: u32,
    restart_threshold: u32,
    widen_count: u32,
    base_scale: f64,
    best_fitness: f64,
}

impl ReviewState {
    pub fn new(plateau_threshold: u32, restart_threshold: u32, base_scale: f64) -> Self {
        Self {
            plateau_count: 0,
            plateau_threshold,
            restart_threshold,
            widen_count: 0,
            base_scale,
            best_fitness: f64::NEG_INFINITY,
        }
    }

    /// Assess a fitness value. Returns the next action.
    pub fn assess(&mut self, fitness: f64) -> Action {
        if fitness > self.best_fitness {
            self.best_fitness = fitness;
            self.plateau_count = 0;
            self.widen_count = 0;
            return Action::Continue;
        }
        self.plateau_count += 1;
        if self.plateau_count < self.plateau_threshold {
            return Action::Continue;
        }
        self.plateau_count = 0;
        self.widen_count += 1;
        if self.widen_count >= self.restart_threshold {
            self.widen_count = 0;
            return Action::Restart;
        }
        let scale = self.base_scale * 2.0_f64.powi(self.widen_count as i32);
        Action::WidenSearch(scale)
    }

    pub fn base_scale(&self) -> f64 {
        self.base_scale
    }

    pub fn plateau_count(&self) -> u32 {
        self.plateau_count
    }

    pub fn widen_count(&self) -> u32 {
        self.widen_count
    }

    pub fn best_fitness(&self) -> f64 {
        self.best_fitness
    }

    pub fn resume(
        plateau_threshold: u32,
        restart_threshold: u32,
        base_scale: f64,
        plateau_count: u32,
        widen_count: u32,
        best_fitness: f64,
    ) -> Self {
        Self {
            plateau_count,
            plateau_threshold,
            restart_threshold,
            widen_count,
            base_scale,
            best_fitness,
        }
    }
}

/// Generate random params within bounded ranges.
pub fn random_params() -> ExperimentParams {
    let mut rng = rand::rng();
    ExperimentParams::new()
        .with("confidence_threshold", rng.random_range(0.01..=0.99))
        .with("attenuation_factor", rng.random_range(0.1..=2.0))
        .with("max_depth", rng.random_range(1.0..=20.0))
}
