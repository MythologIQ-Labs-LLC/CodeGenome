use std::collections::HashMap;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct Checkpoint {
    pub iteration: u64,
    pub params: HashMap<String, f64>,
    pub fitness_fn: String,
    pub best_fitness: f64,
    pub best_stability: f64,
    pub scale: f64,
    pub plateau_count: u32,
    pub widen_count: u32,
    pub last_chain_hash: String,
}

pub fn save(path: &Path, cp: &Checkpoint) -> Result<(), String> {
    let tmp = path.with_extension("tmp");
    let json = serde_json::to_string_pretty(cp).map_err(|e| e.to_string())?;
    std::fs::write(&tmp, json).map_err(|e| e.to_string())?;
    std::fs::rename(&tmp, path).map_err(|e| e.to_string())
}

pub fn load(path: &Path) -> Result<Checkpoint, String> {
    let data = std::fs::read_to_string(path).map_err(|e| e.to_string())?;
    serde_json::from_str(&data).map_err(|e| e.to_string())
}

pub fn checkpoint_path(log_path: &Path) -> PathBuf {
    log_path.with_extension("checkpoint.json")
}
