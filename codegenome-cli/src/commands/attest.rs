use std::path::Path;

use codegenome_identity::store::attest::{attest_store, verify_statement};

pub fn run(store_dir: &str, output: &str, verify: Option<&str>) {
    let dir = Path::new(store_dir);

    if let Some(statement_path) = verify {
        let raw = match std::fs::read_to_string(statement_path) {
            Ok(s) => s,
            Err(e) => {
                eprintln!("Failed to read {statement_path}: {e}");
                return;
            }
        };
        let statement: serde_json::Value = match serde_json::from_str(&raw) {
            Ok(v) => v,
            Err(e) => {
                eprintln!("Invalid statement JSON: {e}");
                return;
            }
        };
        match verify_statement(&statement, dir) {
            Ok(failures) if failures.is_empty() => {
                println!("VERIFIED: store matches attestation");
            }
            Ok(failures) => {
                println!("TAMPERED: {} subject(s) failed", failures.len());
                for f in failures {
                    println!("  - {f}");
                }
                std::process::exit(1);
            }
            Err(e) => eprintln!("Verification error: {e}"),
        }
        return;
    }

    match attest_store(dir) {
        Ok(statement) => {
            let json = serde_json::to_string_pretty(&statement).unwrap_or_else(|_| "{}".into());
            if output == "-" {
                println!("{json}");
            } else if let Err(e) = std::fs::write(output, json) {
                eprintln!("Failed to write {output}: {e}");
            } else {
                eprintln!(
                    "Attestation written to {output} ({} subjects). Sign in CI with: cosign attest --predicate {output}",
                    statement["subject"].as_array().map(|a| a.len()).unwrap_or(0)
                );
            }
        }
        Err(e) => eprintln!("Attestation failed: {e}"),
    }
}
