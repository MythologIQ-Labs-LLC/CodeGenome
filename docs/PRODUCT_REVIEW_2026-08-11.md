# CodeGenome Product Review — 2026-08-11

Comprehensive repository review and technology-landscape research, covering: code health, project hygiene, dependency freshness, GitHub state (issues/PRs/CI), and where the surrounding technology has advanced past the repo. Concludes with a prioritized development plan for maturing CodeGenome from research prototype to product.

---

## 1. Executive Summary

CodeGenome's core is in better shape than most research prototypes: the workspace compiles clean on Rust 1.94, clippy is near-silent, 262 tests exist, error handling is disciplined, and the multi-language architecture (trait-based backends over a shared IR) is genuinely well designed. The thesis — a multi-layer, confidence-fused code graph as agent memory — has been independently validated by 2025–26 research (LocAgent, CodeCompass, Codebase-Memory).

But the project has three classes of problems standing between it and "complete product":

1. **Correctness defects in the core thesis.** Symbol identity is name-addressed, not content-addressed (all `run()` functions collapse to one node). The Rust language backend never recurses below top-level items, so `impl` methods produce zero call edges — the call graph is largely empty for idiomatic Rust. The MCP server panics on malformed tool arguments. The experiment engine's headline `SWITCH_FITNESS` action is a no-op.
2. **Absent product infrastructure.** No CI whatsoever, no release tags, no publishable crate metadata, ~96 MB of experiment logs committed to git (44 MB of it live scratch state at the repo root), and documentation whose headline numbers contradict each other (199 vs 224 vs 254 vs 262 tests).
3. **Technology drift.** MCP went stateless on 2026-07-28 and the repo is four major SDK versions behind (rmcp 0.16 vs 3.x). tree-sitter is two ABI-relevant minors behind. stack-graphs died; SCIP went open-governance; agent-memory interop standardization (W3C CG, MIF, AMP) started without CodeGenome at the table.

The governance system itself needs attention: the Session 5 seal attests a directory rename that never completed (`codegenome-core/` remains as a 56-file byte-identical orphan), all 9 SHADOW_GENOME failure entries have remediation "Pending," and the ledger's genesis hash references two documents (`CONCEPT.md`, `ARCHITECTURE_PLAN.md`) that are absent from the repo.

Note: no artifact named "EvolvAI" exists in the repo; "self-evolving" refers to the experiment engine (`codegenome-substrate/src/experiments/`). This review treats the ask as: evolve CodeGenome into a standalone product.

---

## 2. What Is Solid

- **Build health:** `cargo check`, `cargo test --no-run`, and `cargo clippy` all pass (4 minor warnings + 4 unused test imports total).
- **Architecture:** clean crate layering (identity → substrate → cli/mcp); trait-based `LanguageSupport` with a language-neutral IR and shared graph builder; 9 real overlays; noisy-OR fusion; well-built tamper-evident TSV hash chain with atomic checkpointing in the experiment engine.
- **MCP integration is real:** official `rmcp` SDK (not hand-rolled), 11 tools, a genuine write gate (actor + toolchain + index-freshness policy), Claude Code init support.
- **The LLM advisor is real, not stubbed:** local Phi-3 via mistralrs with graceful degradation. No cloud calls, no API-key handling — a privacy-friendly default.
- **Research honesty:** RUN-001 claims are backed by 50 MB of committed raw data, and the README openly reports that 224K iterations of hill-climbing gained 0.006 fitness while one architectural change gained 0.35.
- **Test quality where it exists** (identity crate: 171 tests, property tests, self-indexing tests, per-language backend coverage).

---

## 3. Functional Defects (highest priority)

| # | Defect | Location | Impact |
|---|--------|----------|--------|
| D1 | MCP server **panics on malformed tool args** (`unwrap_or_else(... panic!)` on deserialize) | `codegenome-mcp/src/server.rs:186-190` | Any bad tool call from an LLM client kills the server. Should return `McpError`. |
| D2 | **Symbol addresses omit file/module path and content** — `blake3("{kind}:{name}")` only | `codegenome-identity/src/lang/graph_builder.rs:126-128` (same bug in legacy `index/parser.rs:133-135`) | Distinct symbols with the same name merge into one node. The "content-addressed identity" thesis is not actually implemented at symbol level. |
| D3 | **Rust backend is non-recursive** — only walks top-level items; `impl` methods yield no call edges; inline `mod` contents invisible | `codegenome-identity/src/lang/rust.rs:59-128` | Call graph largely empty for idiomatic Rust; undercuts `impact`/`trace`/`context` MCP tools. TS and Python backends recurse correctly. |
| D4 | Experiment engine: **`SWITCH_FITNESS` advisor action is inert** — hill-climb reads immutable `infra.fitness_fn`, advisor writes `state.fitness_fn` which is never read after iteration 0 | `codegenome-substrate/src/experiments/runner.rs:37-46,159-162` | The LLM advisor's main lever does nothing; only WIDEN/RESTART take effect. |
| D5 | Experiment engine: **status hardcoded `Pass`**; `Fail`/`Inconclusive` unreachable | `runner.rs:26` | TSV status column is a constant. |
| D6 | Experiment engine **only sees Rust** (`collect_rs_files` + legacy `parse_rust_files`), bypassing the multi-language orchestrator | `experiments/fitness.rs:125-134,218-235` | Fitness signal never observes TS/Python. |
| D7 | **No directory exclusions** during source collection — recurses into `target/`, `node_modules/`, `.git/` | `index/orchestrator.rs:198-217`; `experiments/fitness.rs:218-235` | Indexing this repo with `--source-dir .` ingests vendored dependency sources. |
| D8 | LSP overlay is a **stub** — shells out to `rust-analyzer --version`; the protocol implementation exists only as a comment | `overlay/lsp.rs:32-59` | README claims 9 overlays "Implemented"; backlog is honest, README is not. |
| D9 | Provenance metadata **hardcodes** `"toolchain": "tree-sitter-rust + heuristic-resolver"` in every MCP response regardless of actual backend | `codegenome-mcp/src/tools/mod.rs:132` | Every TS/Python result is mislabelled — in a system whose thesis is rigorous provenance. |
| D10 | `codegenome_status` tool advertises an **empty input schema** but reads `source_dir` from arguments | `server.rs:126-129` | Schema-following clients can never populate it. |
| D11 | Experiment state via MCP is **in-process only** — after restart, `experiment_status`/`experiment_results` return empty despite the TSV on disk; `experiment_start` hardcodes `model_id: None` so the **LLM advisor is unreachable via MCP** | `tools/experiment_tool.rs:38`, `tools/mod.rs:41-52` | MCP surface is a degraded subset of the CLI. |

---

## 4. Repository Hygiene & Infrastructure Gaps

### 4.1 The `codegenome-core/` orphan

`codegenome-core/` (56 files, 4,316 LOC) is byte-identical to `codegenome-substrate/` — residue of the Session 5 rename that copied but never deleted. Its manifest even declares `name = "codegenome-substrate"`. Not a workspace member; never compiled; inflates all counts; will silently rot. **META_LEDGER Entry #118 seals the rename as "SUBSTANTIATED. Reality = Promise" — a falsified seal** the governance protocol failed to catch. Fix: `git rm -r codegenome-core` plus a ledger remediation entry.

### 4.2 Committed experiment data (~96 MB)

- Repo root: `experiment_log.txt` (22 MB, captured stderr — written by no code), `experiments.tsv` (22 MB, the CLI's default `--log-file`), `experiments.checkpoint.json` — **live mutable run state, tracked in git**. Any user following Quick Start immediately dirties 44 MB of tracked files.
- `data/runs/` (50 MB): defensible as a labelled research archive, but belongs in LFS or release assets.
- **None are gitignored.** Meanwhile `.gitignore` lists `docs/`, `plan-*.md`, `.failsafe/` — which are all already tracked and therefore public despite the apparent intent to keep governance artifacts private.

### 4.3 No CI, no releases, no publishing path

- **No `.github/workflows/`** — the only checks on PRs are GitHub's injected CodeQL/Dependabot scans. Every quality claim (test counts, "Section 4 Violations: 0", the plan docs' own "CI Validation" blocks) rests on humans running commands locally. BACKLOG S4 claims CI secret scanning is complete; it was never built.
- **Zero git tags**; backlog cites versions to v0.21.0 while every manifest says 0.1.0.
- **`cargo publish` would fail on all four crates**: no `license`, no `repository`, no `rust-version` in any manifest; no `[workspace.package]`; no `publish = false` either, so intent is ambiguous.
- Absent: CONTRIBUTING.md, CHANGELOG.md, SECURITY.md, CODE_OF_CONDUCT.md, issue/PR templates, rustfmt/clippy config, deny.toml, release automation.
- `.githooks/pre-commit` (gitleaks) is sound but requires manual `git config core.hooksPath .githooks` documented only inside the hook itself — a fresh clone has no secret scanning. `.gitleaks.toml` allowlists the four largest machine-generated surfaces but not `experiment_log.txt` — inconsistent.

### 4.4 Documentation truth

- README states **three different test counts** (badge 199; metrics 224; actual 262), **6 CLI commands** (actual 11), **4 MCP tools** in two places and 11 in two others (actual 11), **15 edge types** (actual 16), a **phantom `codegenome-governance` crate** (three mentions), omits `codegenome-identity` from the workspace table entirely, and the **Quick Start clone URL is wrong** (`MythologIQ/CodeGenome` vs actual `MythologIQ-Labs-LLC/CodeGenome`).
- `SYSTEM_STATE.md` is two sessions stale and contradicts BACKLOG on which security blockers remain (only S2 is actually open).
- All three `plan-*.md` files are the **VETOed** blueprint versions (per SHADOW_GENOME), never revised, still referencing dead `codegenome-core/` paths — yet the work shipped. A reader cannot tell what was actually built.
- The ledger's genesis hash is computed over `CONCEPT.md` + `ARCHITECTURE_PLAN.md`, **neither of which exists in the repo** — the chain is unverifiable from a fresh clone, and it is prose, not machine-parseable.
- SHADOW_GENOME: 9 entries, duplicate numbering, **every remediation "Pending"** — a write-only failure log, which its own entries #5/#6 identify as the problem.

### 4.5 Test-coverage gaps

- `codegenome-cli`: **0 tests** across 11 subcommands.
- `codegenome-mcp`: 7 tests, none covering `dispatch_tool`/`call_tool` — the request path containing the D1 panic is fully unexercised. No `#[tokio::test]` anywhere.
- No `tests/` integration directories in any crate; nothing exercises public APIs externally or the built binary.

---

## 5. GitHub State (as of 2026-08-11)

- **Open issues: 0.** The real backlog exists only in `docs/BACKLOG.md` — invisible to GitHub planning.
- **Open PRs: 4.**
  - **#4** "Add scoped Agent Memory doctrine backlink" — clean, mergeable, CodeQL passed. Ready to merge.
  - **#3** quinn-proto 0.11.14→0.11.16 (security-relevant), **#2** serde_with 3.18→3.21, **#1** git2 0.19→0.20.4 — Dependabot PRs aged 2–7 weeks with **no meaningful checks** (CodeQL neutral or absent). #1 is a semver-major bump. None should merge until CI exists to validate them; with CI in place all three validate automatically.
- **CI gates: none exist.** Only dynamic Dependabot/CodeQL workflows. This is the root cause of the stale PR queue.

---

## 6. Dependency Freshness

Toolchain: rustc 1.94.1, edition 2021 (2024 available), no `rust-toolchain.toml`, no MSRV declared.

| Dependency | Locked | Latest (Aug 2026) | Assessment |
|---|---|---|---|
| **rmcp** | 0.16.0 | **3.1.2** | Four majors behind; 0.16 predates SDK 1.0. Largest gap in the tree. Forces schemars 1.x migration (lock currently carries schemars 0.8 + 0.9 + 1.2 simultaneously). |
| **tree-sitter** | 0.24.7 | **0.26.12** | Two ABI-relevant minors behind; blocks grammar upgrades. Pin carefully (0.26.0 shipped with CLI/codegen churn). |
| tree-sitter-rust / -python | 0.23.3 / 0.23.6 | 0.24.2 / 0.25.0 | Grammar drift: newer language syntax may not parse. |
| tree-sitter-typescript | 0.23.2 | 0.23.2 | Current. |
| **git2** | 0.19.0 | **0.21.0** | Two minors behind; tracks libgit2 security fixes. Low migration cost. |
| **bincode** | 1.3.3 | **3.0.0** | 1.x frozen/legacy. Wire-format change — plan deliberately if serialized output feeds identity hashes. |
| toml | 0.8.23 | 1.1.4 | Superseded major; duplicate 0.9 in lock. |
| rand | 0.9.2 | 0.10.2 | One major behind, small churn. |
| blake3 / serde / serde_json / clap / tokio / rayon | — | — | Current or one `cargo update` away. |
| mistralrs | 0.8.1 | 0.8.1 | Current, but dominates build time — consider feature-gating. |
| lsp-types | 0.97.0 | 0.97.0 | Current but low-activity upstream. |

---

## 7. Technology Landscape — Where the World Moved

### 7.1 MCP went stateless (biggest single item)

Three spec revisions since the repo's integration was built:

- **2025-06-18:** structured tool output (`structuredContent` + `outputSchema`), elicitation, OAuth resource-server model.
- **2025-11-25:** CIMD client registration, icons, URL-mode elicitation, experimental durable **tasks**.
- **2026-07-28:** MCP becomes a **stateless request/response protocol** — no initialize handshake, no sessions; full JSON Schema 2020-12 tool schemas; **cacheable results (`ttlMs`/`cacheScope`)**; tasks graduate to an extension; **Roots, Sampling, Logging, and HTTP+SSE deprecated with a 12-month offramp**.

Implications: upgrade rmcp to 3.x; adopt `outputSchema` for graph results; content-addressed results are the *ideal* cacheable payload (immutable ID ⇒ long TTL); move indexing and experiment runs onto the tasks extension instead of the custom in-process RunManager (which also fixes D11); stdio-only transport is fine for local, but statelessness should shape any remote ambition.

### 7.2 Code-intelligence ecosystem

- **SCIP moved to open governance** (March 2026; steering committee incl. Uber, Meta, Sourcegraph; "SEP" RFC process). Safest interchange bet — and an opening: CodeGenome could **propose confidence/provenance extensions to SCIP** rather than only consuming it.
- **stack-graphs archived** (Sept 2025); **Kythe** in maintenance; **Glean** (Meta) is the healthiest design reference for multi-layer fact storage.
- "Code knowledge graph for agents" became a crowded category (Serena, GitHub MCP server, Sourcegraph MCP, Codebase-Memory, CodeGraphContext, Potpie, Greptile). **Plain "find references" is commodity.** CodeGenome's defensible differentiators are exactly what competitors lack: cross-layer confidence fusion, runtime-trace corroboration, and provenance.
- The 2026 literature (CodeCompass, LocAgent) sets an evidentiary bar: **a graph must demonstrably beat agentic grep** on cross-file tasks with token-cost accounting. CodeGenome has no benchmark story yet.

### 7.3 Agent-memory standardization started

- **W3C AI Agent Memory Interoperability Community Group** (proposed May 2026).
- **MIF** (Memory Interchange Format): JSON-LD + Markdown with **W3C PROV provenance and bi-temporal tracking** — exactly CodeGenome's provenance story, already specified.
- **AMP** (Agent Memory Protocol): markdown-first, git-friendly memory files.
- Framework layer matured: Mem0, Zep/Graphiti (temporal KG with fact-validity windows), Letta, LangMem. Anthropic shipped the API memory tool (file-style `/memories`).

Implications: position CodeGenome as **domain-specific code memory** that generic memory layers point into, not a Mem0 competitor. Build a MIF/AMP export adapter; track the W3C CG. Adopt **bi-temporal validity on edges** (valid-from/valid-until bounded by commits) alongside confidence — this is where the memory field converged, and it formalizes the existing staleness model. Offer a **file-style projection** of graph neighborhoods for memory-tool-native agents.

### 7.4 Provenance/governance standards

- **W3C PROV** is the de facto model for agent/RAG provenance; **PROV-AGENT** (Oak Ridge) extends it with MCP concepts. Mapping CodeGenome's evidence bundles to PROV-DM (edge = Entity, wasGeneratedBy parse/index/trace Activity, wasAssociatedWith Agent) turns the noisy-OR source list into standards-conformant exportable provenance.
- **Sigstore/in-toto/SLSA** are being extended to AI artifacts. Backlog items W4/W5 (SLSA attestation, Sigstore signing of graph artifacts) are no longer wishlist-grade — they're the expected answer to the now-named **memory-poisoning** threat model, and BLAKE3 digests make them cheap to add.
- **Label LLM-derived facts** (advisor conclusions, belief assertions) as AI-generated provenance, distinguishable from compiler-grade facts — matches C2PA/EU-AI-Act disclosure direction.

---

## 8. Prioritized Development Plan

### Phase 0 — Truth and safety (days)
1. Delete `codegenome-core/`; add a META_LEDGER remediation entry correcting seal #118.
2. Fix D1 (MCP panic → `McpError`); add a dispatch-path regression test.
3. Untrack root run state (`experiment_log.txt`, `experiments.tsv`, `experiments.checkpoint.json`); add proper `.gitignore` entries; default the log path out of the repo root. Decide `data/runs/` → LFS or release asset.
4. Fix README: clone URL, crate table (`codegenome-identity` in, `codegenome-governance` out), reconcile all counts; regenerate or delete `SYSTEM_STATE.md`; mark `plan-*.md` superseded; resolve the `.gitignore`-vs-tracked-docs contradiction deliberately.

### Phase 1 — CI and release infrastructure (days)
5. `.github/workflows/ci.yml`: fmt + clippy `-D warnings` + `test --workspace` + build, plus gitleaks and cargo-deny/audit; make it a required check.
6. `[workspace.package]` with `license`, `repository`, `rust-version`; hoist duplicated deps to `[workspace.dependencies]`; add `rust-toolchain.toml`.
7. Merge PR #4; then validate and merge Dependabot PRs #3 (security-relevant — first), #2, #1 (semver-major — review API changes) once CI is green on them.
8. CONTRIBUTING.md (incl. `git config core.hooksPath .githooks`), SECURITY.md, CHANGELOG.md; start tagging releases (release-plz or cargo-release); mirror BACKLOG into GitHub issues so work is schedulable.

### Phase 2 — Core-thesis correctness (weeks)
9. Fix D2: symbol addresses must include file path + module path (and ideally content hash) — this is the product's foundational claim.
10. Fix D3: make the Rust backend recursive (impl methods, nested mods) to parity with TS/Python; add call-graph completeness tests using this repo as fixture.
11. Fix D4–D6: experiment engine (fitness switching, real status, multi-language fitness via the orchestrator); retire or quarantine the legacy Rust-only parse path.
12. Fix D7 (directory exclusion list), D9 (real toolchain provenance per backend), D10 (status schema).
13. Test debt: CLI command tests, MCP integration tests through `dispatch_tool`, at least one end-to-end binary test.

### Phase 3 — Technology catch-up (weeks)
14. rmcp 0.16 → 3.x (+ schemars 1.x): stateless model, `outputSchema` structured results, cache hints on content-addressed responses, tasks extension for indexing/experiments (fixes D11), elicitation for setup.
15. tree-sitter 0.24 → 0.26 + grammar bumps (rust 0.24, python 0.25); pin carefully.
16. git2 → 0.21; plan bincode 1 → 3 deliberately (wire-format audit first); edition 2024.
17. LSP overlay: implement the protocol or de-scope it honestly (README currently overclaims).

### Phase 4 — Product differentiation (months)
18. **Provenance as a standard**: PROV-DM export of evidence bundles; label LLM-derived facts; claim-level auditability in every MCP response (node IDs + source spans + per-source confidence).
19. **Attestation** (backlog W4/W5): Sigstore-signed in-toto attestations over index snapshots — the memory-poisoning answer.
20. **Interop**: MIF/AMP export adapter; engage the W3C Agent Memory CG; propose confidence/provenance SEPs to SCIP.
21. **Temporal model**: bi-temporal edge validity bounded by commits, complementing confidence.
22. **Benchmarks**: LocAgent/CodeCompass-style evaluation vs agentic-grep baselines with token-cost accounting — the adoption argument the field now requires.
23. **File-style memory projection** of graph neighborhoods for memory-tool-native agents.

### Governance process (parallel, ongoing)
- Close or explicitly triage the 9 SHADOW_GENOME "Pending" remediations; fix the duplicate numbering.
- Restore/reconstruct `CONCEPT.md` and `ARCHITECTURE_PLAN.md` (or re-anchor the genesis hash) and make the ledger machine-verifiable — ideally by CodeGenome's own tooling (`codegenome verify` extended to the ledger). The strongest possible demo of the product is the product verifying its own governance chain.
- Preserve audit history instead of overwriting `AUDIT_REPORT.md`.

---

## 9. Verification Notes

Findings in this review were produced by parallel code inspection, `cargo check/test/clippy` runs on the current toolchain, git history analysis, GitHub API queries, and web research with sources (MCP spec/blog, crates.io, SCIP/tree-sitter/stack-graphs repos, W3C, arXiv). File/line references are as of commit `02565cc` on `main`.
