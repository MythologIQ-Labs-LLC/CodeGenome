# Contributing to CODEGENOME

CODEGENOME is an active research project with a dual mandate — experimental instrument and product substrate (see the README's *Intent & Value Provenance* section). Contributions are welcome, but the governance protocol below is not optional.

## Getting set up

```bash
git clone https://github.com/MythologIQ-Labs-LLC/CodeGenome.git
cd CodeGenome

# Enable the repo's pre-commit secret scanning hook (required)
git config core.hooksPath .githooks

cargo build --workspace
```

The toolchain is pinned by `rust-toolchain.toml` (stable, with clippy and rustfmt). The declared floor is Rust 1.88.

Note: `codegenome-substrate` depends on `mistralrs` (embedded local LLM); the first build is heavy. Everything else builds quickly.

## Before you push

CI enforces all of the following on every PR — run them locally first:

```bash
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

## Ground rules

- **Don't degrade the experimental apparatus.** The experiment engine, fitness functions, tamper-evident logs, and `data/runs/` archives are first-class features, not scaffolding. Changes that would reduce the repo's capacity to run, log, verify, or archive experiments are regressions.
- **Don't commit run state.** `experiments.tsv`, `experiment_log.txt`, and `*.checkpoint.json` are gitignored live artifacts. Deliberate research archives go under `data/runs/` with a labelled commit.
- **Numbers in docs must be real.** Test counts, tool counts, and metrics in README/docs are verified claims. If your change moves a number, update it — or better, don't hand-maintain what CI can verify.
- **Governance record.** Significant work follows the plan → audit → implement → substantiate cycle recorded in `docs/META_LEDGER.md`; failures and their remediations go in `docs/SHADOW_GENOME.md`. A rename or move must be substantiated on both sides (destination exists **and** source is gone) — see Shadow Genome Failure #6.

## Pull requests

- Branch from `main`; keep PRs focused.
- Describe what changed and why; link the backlog item (`docs/BACKLOG.md`) if one applies.
- CI (format, clippy, tests, secret scan) must pass before review.

For significant or architectural changes, open an issue to discuss with the maintainers first.
