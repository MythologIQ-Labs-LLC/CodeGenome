use std::path::Path;

use codegenome_substrate::experiments::log;

pub fn run(log_file: &str) {
    let path = Path::new(log_file);
    if !path.exists() {
        eprintln!("Log file not found: {log_file}");
        std::process::exit(1);
    }

    match log::read_log(path) {
        Ok(results) => {
            let last_hash = results
                .last()
                .map(|r| r.chain_hash.as_str())
                .unwrap_or("(empty)");
            println!(
                "Chain verified. {} entries. Last hash: {last_hash}",
                results.len()
            );
        }
        Err(e) => {
            eprintln!("Verification FAILED: {e}");
            std::process::exit(1);
        }
    }
}
