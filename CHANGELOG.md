# Changelog

All notable changes to this project are documented here.
Format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/); the project does not yet cut tagged releases (all crates are 0.1.0 — versions cited in `docs/BACKLOG.md` up to v0.21.0 predate this file and were documentation-level only).

## [Unreleased] — 2026-08-11 repository review remediation

### Fixed
- **MCP server no longer panics on malformed tool arguments.** `dispatch_tool` returns a proper MCP `invalid_params` error for bad arguments and unknown tool names; previously any malformed call from a client crashed the stdio service. Regression tests added.
- Deleted `codegenome-core/` — a 56-file byte-identical orphan of `codegenome-substrate/` left behind by the Session 5 rename. The governance seal claiming that rename was complete is corrected by Shadow Genome Failure #6.
- README corrections: clone URL, actual crate names (the documented `codegenome-governance` crate never existed), test count (264), CLI command count (11), MCP tool count (11), edge types (16), ledger entries (118); LSP overlay is now honestly labelled a stub; RUN-002 marked paused.
- `SYSTEM_STATE.md` regenerated from verified reality (previous seal was two sessions stale and contradicted BACKLOG).

### Added
- **CI** (`.github/workflows/ci.yml`): rustfmt check, clippy `-D warnings`, full test suite, gitleaks secret scan, and advisory `cargo audit` on every PR.
- README *Intent & Value Provenance* section: the repository's permanent dual mandate (experimental instrument + product substrate) and the rule that product claims trace to the experimental record.
- Workspace package metadata (`license`, `repository`, `rust-version` inherited via `[workspace.package]`) — crates are now publishable in principle. `rust-version = 1.88` is the documented floor, not yet CI-verified.
- `rust-toolchain.toml` (stable + clippy + rustfmt), `CONTRIBUTING.md`, `SECURITY.md`, this changelog.
- Shadow Genome Failure #6 — first entry with a completed remediation.

### Changed
- One-time `cargo fmt` sweep across the workspace (518 pre-existing diffs) so the CI format gate starts from a clean baseline.
- Dependencies refreshed: git2 0.19 → 0.20.4 (validated, semver-major), quinn-proto 0.11.16 and serde_with 3.22 (Dependabot PRs #1–#3 merged after local validation), plus compatible `cargo update` sweep. All 264 tests pass.
- Live experiment run state (`experiments.tsv`, `experiment_log.txt`, `experiments.checkpoint.json`, ~44 MB) untracked and gitignored; deliberate research archives remain in `data/runs/`. Historical blobs remain in git history (an LFS/history rewrite was deliberately not performed).
- The three VETOed `docs/plan-*.md` blueprints carry superseded banners and are retained as historical record.

## [Unreleased] — Phase 2: core-thesis correctness

### Fixed
- **Symbol identity is now file-scoped** (`sym:{path}:{kind}:{name}`). Previously `blake3(kind:name)` merged every same-named symbol across the codebase into one node. The formula existed as five divergent inline copies (graph builder, both resolvers, process overlay, runtime-trace overlay, federation export table); all now call the single shared `symbol_address`. Same-file same-kind same-name collisions remain a known limitation.
- **Rust extraction is recursive**: `impl` methods, items in inline `mod` blocks, and nested functions are extracted as symbols; calls inside methods produce call edges (the call graph was previously empty for idiomatic Rust); call attribution picks the innermost enclosing function.
- **All languages contribute syntax symbols**: `parse_one` delegated to the language backends instead of hardcoded Rust node kinds, so TypeScript/Python files now produce symbol nodes (previously zero) and `ParsedFile` provenance names the real backend.
- **Experiment engine**: `SWITCH_FITNESS` now takes effect (the hill-climb evaluates the loop's current fitness function, with best-score re-baselining on switch); result status is computed (Fail on numeric divergence, Inconclusive when nothing was measurable) instead of hardcoded `Pass`; fitness builds overlays through the multi-language pipeline instead of the Rust-only legacy path.
- **Source collection excludes** hidden dirs, `target/`, `node_modules/`, `__pycache__/`, `venv/`, `vendor/` (both the indexer and the experiment engine's file walk).
- MCP: response provenance derives the toolchain label from registered backends (TS/Python results were labelled "tree-sitter-rust"); `codegenome_status` schema now declares the `source_dir` field it reads; `codegenome_experiment_status` has its own input type.

### Added
- Regression tests: recursive extraction (impl methods, inline modules, nested-fn attribution, in-function imports), cross-file address distinctness, CLI arg-tree validation + parse tests for all 11 subcommands + index→status round-trip (the CLI crate previously had zero tests).
- `.cargo/audit.toml`: the CI audit job is now blocking, with RUSTSEC-2026-0189 (rmcp HTTP transport, not compiled into this stdio-only server) as the single documented, time-boxed exception until the Phase 3 rmcp upgrade. A red audit job now always means a new, untriaged advisory.

### Known issues (tracked in docs/PRODUCT_REVIEW_2026-08-11.md)
- rmcp 0.16 predates the stateless MCP 2026-07-28 spec (current SDK: 3.x) and carries RUSTSEC-2026-0189 (DNS rebinding in the HTTP transport — not compiled into this stdio-only server); tree-sitter 0.24 vs current 0.26; bincode 1.x unmaintained. All Phase 3.
- LSP overlay remains a stub; experiment state over MCP is in-process only and the LLM advisor is CLI-only.
