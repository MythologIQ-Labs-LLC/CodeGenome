use std::fs::{File, OpenOptions};
use std::io::{BufRead, BufReader, Write};
use std::path::Path;

#[derive(Clone, Debug)]
pub struct LedgerEntry {
    pub sequence: u64,
    pub timestamp: u64,
    pub operation: String,
    pub actor: String,
    pub input_hash: String,
    pub output_hash: String,
    pub chain_hash: String,
}

pub fn genesis_hash() -> String {
    row_hash("CODEGENOME_GOVERNANCE_GENESIS", "")
}

pub fn append(path: &Path, entry: &LedgerEntry, prev_hash: &str) -> Result<String, String> {
    let needs_header = !path.exists() || file_is_empty(path);
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)
        .map_err(|e| e.to_string())?;

    if needs_header {
        writeln!(
            file,
            "seq\ttimestamp\toperation\tactor\tinput\toutput\tchain_hash"
        )
        .map_err(|e| e.to_string())?;
    }

    let row = format!(
        "{}\t{}\t{}\t{}\t{}\t{}",
        entry.sequence,
        entry.timestamp,
        entry.operation,
        entry.actor,
        entry.input_hash,
        entry.output_hash,
    );
    let hash = row_hash(&row, prev_hash);
    writeln!(file, "{row}\t{hash}").map_err(|e| e.to_string())?;
    file.flush().map_err(|e| e.to_string())?;
    Ok(hash)
}

pub fn read_ledger(path: &Path) -> Result<Vec<LedgerEntry>, String> {
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

fn parse_line(line: &str) -> Result<(LedgerEntry, String, String), String> {
    let parts: Vec<&str> = line.split('\t').collect();
    if parts.len() < 7 {
        return Err(format!("Ledger row has {} cols, expected 7", parts.len()));
    }
    let content = parts[..6].join("\t");
    let entry = LedgerEntry {
        sequence: parts[0]
            .parse()
            .map_err(|e: std::num::ParseIntError| e.to_string())?,
        timestamp: parts[1]
            .parse()
            .map_err(|e: std::num::ParseIntError| e.to_string())?,
        operation: parts[2].to_string(),
        actor: parts[3].to_string(),
        input_hash: parts[4].to_string(),
        output_hash: parts[5].to_string(),
        chain_hash: parts[6].to_string(),
    };
    Ok((entry, content, parts[6].to_string()))
}

fn verify_chain(rows: &[(String, String)]) -> Result<(), String> {
    let mut prev = genesis_hash();
    for (i, (content, hash)) in rows.iter().enumerate() {
        let expected = row_hash(content, &prev);
        if expected != *hash {
            return Err(format!("Governance chain failure at row {i}"));
        }
        prev = hash.clone();
    }
    Ok(())
}

fn row_hash(content: &str, prev: &str) -> String {
    let input = format!("{content}{prev}");
    blake3::hash(input.as_bytes()).to_hex()[..16].to_string()
}

fn file_is_empty(path: &Path) -> bool {
    std::fs::metadata(path)
        .map(|m| m.len() == 0)
        .unwrap_or(true)
}
