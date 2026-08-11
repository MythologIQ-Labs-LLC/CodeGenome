pub mod entry;

use std::fs::{File, OpenOptions};
use std::io::{BufRead, BufReader, Write};
use std::path::Path;

/// A graph operation that produces evidence.
#[derive(Clone, Debug)]
pub enum Operation {
    Index,
    Query,
    DetectChanges,
    Fuse,
}

/// One evidence entry in the tamper-evident log.
#[derive(Clone, Debug)]
pub struct EvidenceEntry {
    pub timestamp: u64,
    pub operation: Operation,
    pub input_hash: String,
    pub output_hash: String,
    pub chain_hash: String,
}

pub fn genesis_hash() -> String {
    row_hash("CODEGENOME_EVIDENCE_GENESIS", "")
}

pub fn log_evidence(path: &Path, entry: &EvidenceEntry, prev_hash: &str) -> Result<String, String> {
    let needs_header = !path.exists() || file_is_empty(path);
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)
        .map_err(|e| e.to_string())?;

    if needs_header {
        writeln!(
            file,
            "timestamp\toperation\tinput_hash\toutput_hash\tchain_hash"
        )
        .map_err(|e| e.to_string())?;
    }

    let op_str = op_to_str(&entry.operation);
    let row = format!(
        "{}\t{}\t{}\t{}",
        entry.timestamp, op_str, entry.input_hash, entry.output_hash,
    );
    let hash = row_hash(&row, prev_hash);
    writeln!(file, "{row}\t{hash}").map_err(|e| e.to_string())?;
    file.flush().map_err(|e| e.to_string())?;
    Ok(hash)
}

pub fn read_evidence(path: &Path) -> Result<Vec<EvidenceEntry>, String> {
    let file = File::open(path).map_err(|e| e.to_string())?;
    let reader = BufReader::new(file);
    let mut entries = Vec::new();
    let mut rows = Vec::new();

    for (i, line) in reader.lines().enumerate() {
        let line = line.map_err(|e| e.to_string())?;
        if i == 0 || line.trim().is_empty() {
            continue;
        }
        let (entry, content, hash) = parse_line(&line)?;
        rows.push((content, hash));
        entries.push(entry);
    }
    verify_chain(&rows)?;
    Ok(entries)
}

fn parse_line(line: &str) -> Result<(EvidenceEntry, String, String), String> {
    let parts: Vec<&str> = line.split('\t').collect();
    if parts.len() < 5 {
        return Err(format!(
            "Evidence row has {} columns, expected 5",
            parts.len()
        ));
    }
    let content = parts[..4].join("\t");
    let entry = EvidenceEntry {
        timestamp: parts[0]
            .parse()
            .map_err(|e: std::num::ParseIntError| e.to_string())?,
        operation: str_to_op(parts[1]),
        input_hash: parts[2].to_string(),
        output_hash: parts[3].to_string(),
        chain_hash: parts[4].to_string(),
    };
    Ok((entry, content, parts[4].to_string()))
}

fn verify_chain(rows: &[(String, String)]) -> Result<(), String> {
    let mut prev = genesis_hash();
    for (i, (content, hash)) in rows.iter().enumerate() {
        let expected = row_hash(content, &prev);
        if expected != *hash {
            return Err(format!(
                "Evidence chain failure at row {i}: expected {expected}, found {hash}"
            ));
        }
        prev = hash.clone();
    }
    Ok(())
}

fn row_hash(content: &str, prev: &str) -> String {
    let input = format!("{content}{prev}");
    blake3::hash(input.as_bytes()).to_hex()[..16].to_string()
}

fn op_to_str(op: &Operation) -> &'static str {
    match op {
        Operation::Index => "index",
        Operation::Query => "query",
        Operation::DetectChanges => "detect_changes",
        Operation::Fuse => "fuse",
    }
}

fn str_to_op(s: &str) -> Operation {
    match s {
        "index" => Operation::Index,
        "query" => Operation::Query,
        "detect_changes" => Operation::DetectChanges,
        _ => Operation::Fuse,
    }
}

fn file_is_empty(path: &Path) -> bool {
    std::fs::metadata(path)
        .map(|m| m.len() == 0)
        .unwrap_or(true)
}
