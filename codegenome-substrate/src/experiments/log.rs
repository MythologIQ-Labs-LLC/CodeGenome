use std::collections::{BTreeMap, HashMap};
use std::fs::{File, OpenOptions};
use std::io::{BufRead, BufReader, Write};
use std::path::Path;

use crate::experiments::config::ExperimentStatus;

/// Result of a single experiment run.
#[derive(Clone, Debug)]
pub struct ExperimentResult {
    pub iteration: u64,
    pub params: HashMap<String, f64>,
    pub fitness: f64,
    pub stability: f64,
    pub status: ExperimentStatus,
    pub cycle_time_ms: u64,
    pub description: String,
    pub chain_hash: String,
}

/// Compute BLAKE3 hash of row content + previous hash.
pub fn row_hash(row_tsv: &str, prev_hash: &str) -> String {
    let input = format!("{row_tsv}{prev_hash}");
    blake3::hash(input.as_bytes()).to_hex()[..16].to_string()
}

/// Append one experiment result to TSV log with chain hash.
pub fn log_result(
    path: &Path,
    result: &ExperimentResult,
    prev_hash: &str,
) -> Result<String, String> {
    let needs_header = !path.exists() || file_is_empty(path);
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)
        .map_err(|e| e.to_string())?;

    if needs_header {
        writeln!(
            file,
            "iteration\tfitness\tstability\tcycle_ms\tstatus\tparams_json\tdescription\tchain_hash"
        )
        .map_err(|e| e.to_string())?;
    }

    let row_content = format_row(result);
    let hash = row_hash(&row_content, prev_hash);
    writeln!(file, "{row_content}\t{hash}").map_err(|e| e.to_string())?;
    file.flush().map_err(|e| e.to_string())?;
    Ok(hash)
}

/// Read all results from a TSV log with chain verification.
pub fn read_log(path: &Path) -> Result<Vec<ExperimentResult>, String> {
    let file = File::open(path).map_err(|e| e.to_string())?;
    let reader = BufReader::new(file);
    let mut results = Vec::new();
    let mut rows_for_verify = Vec::new();

    for (i, line) in reader.lines().enumerate() {
        let line = line.map_err(|e| e.to_string())?;
        if i == 0 || line.trim().is_empty() {
            continue;
        }
        let (result, row_content, hash) = parse_tsv_line(&line)?;
        rows_for_verify.push((row_content, hash));
        results.push(result);
    }

    verify_chain(&rows_for_verify)?;
    Ok(results)
}

fn format_row(result: &ExperimentResult) -> String {
    let status_str = match result.status {
        ExperimentStatus::Pass => "pass",
        ExperimentStatus::Fail => "fail",
        ExperimentStatus::Inconclusive => "inconclusive",
    };
    let params_json = params_json(&result.params).map_err_to_string();
    format!(
        "{}\t{:.6}\t{:.6}\t{}\t{}\t{}\t{}",
        result.iteration,
        result.fitness,
        result.stability,
        result.cycle_time_ms,
        status_str,
        params_json,
        result.description,
    )
}

fn parse_tsv_line(line: &str) -> Result<(ExperimentResult, String, String), String> {
    let parts: Vec<&str> = line.split('\t').collect();
    if parts.len() < 8 {
        return Err(format!("TSV row has {} columns, expected 8", parts.len()));
    }
    let hash = parts[7].to_string();
    let row_content = parts[..7].join("\t");
    let result = ExperimentResult {
        iteration: parts[0]
            .parse()
            .map_err(|e: std::num::ParseIntError| e.to_string())?,
        fitness: parts[1]
            .parse()
            .map_err(|e: std::num::ParseFloatError| e.to_string())?,
        stability: parts[2]
            .parse()
            .map_err(|e: std::num::ParseFloatError| e.to_string())?,
        cycle_time_ms: parts[3]
            .parse()
            .map_err(|e: std::num::ParseIntError| e.to_string())?,
        status: match parts[4] {
            "pass" => ExperimentStatus::Pass,
            "fail" => ExperimentStatus::Fail,
            _ => ExperimentStatus::Inconclusive,
        },
        params: parse_params(parts[5])?,
        description: parts[6].to_string(),
        chain_hash: hash.clone(),
    };
    Ok((result, row_content, hash))
}

fn verify_chain(rows: &[(String, String)]) -> Result<(), String> {
    let mut prev = genesis_hash();
    for (i, (content, hash)) in rows.iter().enumerate() {
        let expected = row_hash(content, &prev);
        if expected != *hash {
            return Err(format!(
                "Chain integrity failure at row {i}: expected {expected}, found {hash}"
            ));
        }
        prev = hash.clone();
    }
    Ok(())
}

pub fn genesis_hash() -> String {
    row_hash("CODEGENOME_EXPERIMENT_GENESIS", "")
}

pub fn last_hash_from_log(path: &Path) -> Result<String, String> {
    let results = read_log(path)?;
    Ok(results
        .last()
        .map_or_else(genesis_hash, |r| r.chain_hash.clone()))
}

fn file_is_empty(path: &Path) -> bool {
    std::fs::metadata(path)
        .map(|m| m.len() == 0)
        .unwrap_or(true)
}

fn parse_params(value: &str) -> Result<HashMap<String, f64>, String> {
    let parsed: BTreeMap<String, f64> =
        serde_json::from_str(value).map_err(|e| format!("Invalid params_json: {e}"))?;
    Ok(parsed.into_iter().collect())
}

fn params_json(params: &HashMap<String, f64>) -> Result<String, serde_json::Error> {
    let ordered: BTreeMap<String, f64> = params.iter().map(|(k, v)| (k.clone(), *v)).collect();
    serde_json::to_string(&ordered)
}

trait MapErrToString<T> {
    fn map_err_to_string(self) -> T;
}

impl MapErrToString<String> for Result<String, serde_json::Error> {
    fn map_err_to_string(self) -> String {
        self.unwrap_or_else(|_| "{}".into())
    }
}
