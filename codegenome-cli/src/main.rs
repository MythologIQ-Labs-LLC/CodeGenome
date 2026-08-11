mod commands;

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "codegenome", about = "Unified Code Reality Graph")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Build the code graph from source files
    Index {
        #[arg(long, default_value = ".")]
        source_dir: String,
        #[arg(long, default_value = ".codegenome")]
        store_dir: String,
    },
    /// Query impact from a file:line location
    Query {
        #[arg(long, default_value = ".codegenome")]
        store_dir: String,
        #[arg(long)]
        file: String,
        #[arg(long)]
        line: u32,
        #[arg(long, default_value = "downstream")]
        direction: String,
        #[arg(long)]
        json: bool,
    },
    /// Show index status and overlay counts
    Status {
        #[arg(long, default_value = ".codegenome")]
        store_dir: String,
        #[arg(long)]
        json: bool,
    },
    /// Start MCP tool server (stdio)
    Serve {
        #[arg(long, default_value = ".")]
        source_dir: String,
        #[arg(long, default_value = ".codegenome")]
        store_dir: String,
    },
    /// Initialize .mcp.json for Claude Code integration
    Init {
        #[arg(long, default_value = ".")]
        source_dir: String,
        #[arg(long, default_value = ".codegenome")]
        store_dir: String,
    },
    /// Verify experiment TSV chain integrity
    Verify {
        #[arg(long, default_value = "experiments.tsv")]
        log_file: String,
    },
    /// Analyze repo-local experiment results
    Analyze {
        #[arg(long, default_value = "experiments.tsv")]
        log_file: String,
        #[arg(long)]
        json: bool,
    },
    /// Run autonomous experiment loop
    Experiment {
        #[arg(long, default_value = ".")]
        source_dir: String,
        #[arg(long, default_value = "experiments.tsv")]
        log_file: String,
        #[arg(long)]
        max_iterations: Option<u64>,
        #[arg(long, default_value = "microsoft/Phi-3-mini-4k-instruct")]
        model: String,
        #[arg(long)]
        no_model: bool,
    },
    /// Build explicit workspace federation overlay
    Federate {
        #[arg(long)]
        workspace_config: String,
        #[arg(long, default_value = ".codegenome-workspace")]
        store_dir: String,
    },
    /// Export graph as interactive HTML visualization
    Visualize {
        #[arg(long, default_value = ".codegenome")]
        store_dir: String,
        #[arg(long, default_value = "graph.html")]
        output: String,
        #[arg(long, default_value_t = 0.0)]
        min_confidence: f64,
    },
    /// Export graph provenance as W3C PROV-JSON
    ExportProv {
        #[arg(long, default_value = ".codegenome")]
        store_dir: String,
        #[arg(long, default_value = "provenance.prov.json")]
        output: String,
    },
    /// Emit or verify an in-toto attestation of the index snapshot
    Attest {
        #[arg(long, default_value = ".codegenome")]
        store_dir: String,
        #[arg(long, default_value = "index-attestation.json")]
        output: String,
        /// Verify an existing attestation file instead of emitting
        #[arg(long)]
        verify: Option<String>,
    },
    /// Render a markdown repo map projection of the graph
    Map {
        #[arg(long, default_value = ".")]
        source_dir: String,
        #[arg(long, default_value = ".codegenome")]
        store_dir: String,
        #[arg(long, default_value = "-")]
        output: String,
    },
    /// Report workspace federation metrics
    WorkspaceReport {
        #[arg(long, default_value = ".codegenome-workspace")]
        store_dir: String,
        #[arg(long)]
        json: bool,
    },
}

fn main() {
    let cli = Cli::parse();
    match cli.command {
        Commands::Index {
            source_dir,
            store_dir,
        } => {
            commands::index::run(&source_dir, &store_dir);
        }
        Commands::Query {
            store_dir,
            file,
            line,
            direction,
            json,
        } => {
            commands::query::run(&store_dir, &file, line, &direction, json);
        }
        Commands::Status { store_dir, json } => {
            commands::status::run(&store_dir, json);
        }
        Commands::Serve {
            source_dir,
            store_dir,
        } => {
            commands::serve::run(&source_dir, &store_dir);
        }
        Commands::Init {
            source_dir,
            store_dir,
        } => {
            commands::init::run(&source_dir, &store_dir);
        }
        Commands::Verify { log_file } => {
            commands::verify::run(&log_file);
        }
        Commands::Analyze { log_file, json } => {
            commands::analyze::run(&log_file, json);
        }
        Commands::Experiment {
            source_dir,
            log_file,
            max_iterations,
            model,
            no_model,
        } => commands::experiment::run(
            &source_dir,
            &log_file,
            max_iterations,
            if no_model { None } else { Some(model) },
        ),
        Commands::Federate {
            workspace_config,
            store_dir,
        } => {
            commands::federate::run(&workspace_config, &store_dir);
        }
        Commands::Visualize {
            store_dir,
            output,
            min_confidence,
        } => {
            commands::visualize::run(&store_dir, &output, min_confidence);
        }
        Commands::ExportProv { store_dir, output } => {
            commands::export_prov::run(&store_dir, &output);
        }
        Commands::Attest {
            store_dir,
            output,
            verify,
        } => {
            commands::attest::run(&store_dir, &output, verify.as_deref());
        }
        Commands::Map {
            source_dir,
            store_dir,
            output,
        } => {
            commands::map::run(&source_dir, &store_dir, &output);
        }
        Commands::WorkspaceReport { store_dir, json } => {
            commands::workspace_report::run(&store_dir, json);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::CommandFactory;

    #[test]
    fn cli_arg_tree_is_valid() {
        // Catches conflicting/duplicate/malformed arg definitions for
        // every subcommand at test time instead of at first invocation.
        Cli::command().debug_assert();
    }

    #[test]
    fn all_subcommands_parse() {
        let cases: &[&[&str]] = &[
            &["codegenome", "index", "--source-dir", "src"],
            &["codegenome", "query", "--file", "a.rs", "--line", "4"],
            &["codegenome", "status", "--json"],
            &["codegenome", "serve"],
            &["codegenome", "init"],
            &["codegenome", "verify"],
            &["codegenome", "analyze", "--json"],
            &[
                "codegenome",
                "experiment",
                "--no-model",
                "--max-iterations",
                "5",
            ],
            &["codegenome", "federate", "--workspace-config", "ws.toml"],
            &["codegenome", "visualize", "--min-confidence", "0.5"],
            &["codegenome", "workspace-report"],
            &["codegenome", "export-prov"],
            &["codegenome", "attest"],
            &["codegenome", "attest", "--verify", "a.json"],
            &["codegenome", "map"],
        ];
        for args in cases {
            assert!(
                Cli::try_parse_from(*args).is_ok(),
                "failed to parse: {args:?}"
            );
        }
    }

    #[test]
    fn query_requires_file_and_line() {
        assert!(Cli::try_parse_from(["codegenome", "query"]).is_err());
        assert!(Cli::try_parse_from(["codegenome", "query", "--file", "a.rs"]).is_err());
    }

    #[test]
    fn experiment_defaults_are_stable() {
        let cli = Cli::try_parse_from(["codegenome", "experiment"]).unwrap();
        let Commands::Experiment {
            source_dir,
            log_file,
            max_iterations,
            no_model,
            ..
        } = cli.command
        else {
            panic!("wrong variant");
        };
        assert_eq!(source_dir, ".");
        assert_eq!(log_file, "experiments.tsv");
        assert_eq!(max_iterations, None);
        assert!(!no_model);
    }

    #[test]
    fn index_then_status_round_trip() {
        let base = std::env::temp_dir().join("codegenome-cli-roundtrip");
        let src = base.join("src");
        let store = base.join("store");
        let _ = std::fs::remove_dir_all(&base);
        std::fs::create_dir_all(&src).unwrap();
        std::fs::write(
            src.join("lib.rs"),
            "pub fn entry() { helper(); }\nfn helper() {}\n",
        )
        .unwrap();

        commands::index::run(src.to_str().unwrap(), store.to_str().unwrap());
        assert!(store.exists(), "index must create the store directory");
        let entries = std::fs::read_dir(&store).unwrap().count();
        assert!(entries > 0, "store directory must not be empty");

        // Must not panic reading the store it just wrote.
        commands::status::run(store.to_str().unwrap(), true);
        let _ = std::fs::remove_dir_all(&base);
    }
}
