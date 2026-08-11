<p align="center">
  <strong>CODEGENOME</strong><br>
  <em>Unified Code Reality Graph</em>
</p>

<p align="center">
  <a href="https://www.rust-lang.org/"><img src="https://img.shields.io/badge/rust-1.88%2B-orange?logo=rust&logoColor=white" alt="Rust 1.88+"></a>
  <a href="#project-metrics"><img src="https://img.shields.io/badge/tests-264%20passing-brightgreen" alt="Tests: 264 passing"></a>
  <a href="#graph-composition"><img src="https://img.shields.io/badge/overlays-9%20(8%20active%20%2B%20LSP%20stub)-blueviolet" alt="Overlays: 9 (8 active + LSP stub)"></a>
  <a href="#multi-language-support"><img src="https://img.shields.io/badge/languages-3%20(Rust%20%7C%20TS%20%7C%20Python)-blue" alt="Languages: 3"></a>
  <a href="#governance"><img src="https://img.shields.io/badge/governance-active-critical" alt="Governance: active"></a>
  <a href="#research-tracker"><img src="https://img.shields.io/badge/status-research%20prototype-yellow" alt="Status: research prototype"></a>
  <a href="LICENSE"><img src="https://img.shields.io/badge/license-MIT-blue" alt="License: MIT"></a>
</p>

---

## Table of Contents

- [Overview](#overview)
- [Intent & Value Provenance](#intent--value-provenance)
- [Problem Statement](#problem-statement)
- [Architecture](#architecture)
  - [Workspace](#workspace)
  - [Layer Model](#layer-model)
  - [Core Concepts](#core-concepts)
- [Multi-Language Support](#multi-language-support)
- [Quick Start](#quick-start)
- [Graph Composition](#graph-composition)
- [Confidence Fusion](#confidence-fusion)
- [Experiment Engine](#experiment-engine)
  - [Tier Model](#tier-model)
  - [Fitness Functions](#fitness-functions)
  - [Parameters](#parameters)
- [Research Tracker](#research-tracker)
  - [Completed Experiments](#completed-experiments)
  - [Key Findings](#key-findings)
  - [Open Research Questions](#open-research-questions)
- [Governance](#governance)
- [Agent Memory Alignment](#agent-memory-alignment)
- [MCP Integration](#mcp-integration)
- [Project Metrics](#project-metrics)
- [Dependencies](#dependencies)
- [Roadmap](#roadmap)
- [Contributing](#contributing)
- [License](#license)

---

## Overview

CODEGENOME is a content-addressed, multi-layer program analysis graph that merges syntax, semantics, control/data flow, process traces, and runtime observations into a single queryable substrate. Every node has a cryptographic identity (BLAKE3). Every edge carries a confidence score. Independent evidence sources fuse via noisy-OR. The system hill-climbs on its own accuracy using an adaptive, self-evolving experiment engine.

The intended outcome is not "better code search" or "a smarter assistant." The intended outcome is a **universal internal representation** from which those functions can be derived as query, traversal, policy, and mutation operations over the same canonical reality model.

> **Research prototype.** Not production-ready. APIs will change.

## Intent & Value Provenance

This repository carries a **dual mandate**, and both halves are permanent:

1. **Experimental instrument.** CODEGENOME was designed from the start to *measure itself* — recall/impact accuracy, propagation behavior, graph density, cycle time, and governance integrity are first-class, instrumented metrics. The experiment engine, the fitness functions, the tamper-evident run logs, and the archived raw data in `data/runs/` are not scaffolding to be removed on the way to a product; they are the evidence-generating apparatus the product's claims rest on.
2. **Product substrate.** CODEGENOME is developed toward a usable product: a governed code-reality graph that AI agents consume as domain-specific memory (via MCP and the [Agent Memory](https://github.com/MythologIQ-Labs-LLC/agent-memory) doctrine).

The connection between the two is the point: **every capability the product claims must trace to the experimental record that established it.** Fusion is in the product because RUN-001/RUN-002 showed a structural fitness jump (0.29 → 0.64 baseline) that 224K iterations of parameter tuning could not produce. Freshness gating is in the product because staleness is a measured, computable relation. Future capabilities enter the same way — through experiments whose raw data is retained and whose conclusions are recorded in the governance ledger. Changes that would degrade the repository's capacity to run, log, verify, and archive experiments are treated as regressions, even if they simplify the product.

## Problem Statement

The current tooling landscape fragments code understanding across incompatible models — syntax-aware parsers, semantic indexers, dependency analyzers, runtime tracing systems, and assistant retrieval pipelines. Each builds a partial internal truth optimized for one surface area. The ecosystem compensates by repeatedly translating, summarizing, or approximating across boundaries.

This produces four persistent failures:

| Failure | Description |
|---------|-------------|
| **Identity drift** | The same artifact is represented differently across tools and sessions |
| **Semantic drift** | A relationship recovered in one subsystem cannot be expressed in another without lossy conversion |
| **Provenance loss** | High-level conclusions outlive the evidence and method that produced them |
| **Governance weakness** | Read/write operations lack a substrate-level model for confidence, policy, and auditability |

CODEGENOME addresses these failures by replacing tool-local truth with canonical graph reality.

## Architecture

```
Source Files (.rs, .ts, .tsx, .py)
    │
    ├── [Language Detection]     extension-based routing
    │
    ▼
[Syntax Overlay]      tree-sitter AST → Contains edges (confidence: 1.0)
[Semantic Overlay]    Multi-lang symbol resolution → Calls, Imports, Implements (0.7–0.8)
[Flow Overlay]        Intraprocedural CFG/DFG → ControlFlow, DataFlow (1.0)
[Process Overlay]     Entrypoint detection → PartOfProcess edges (0.9^depth)
[PDG Overlay]         CFG+DFG composition → ControlDependence (1.0)
[Runtime Overlay]     TSV trace ingestion → weighted Calls (count/10)
[SCIP Overlay]        Compiler index → References, Implements (1.0)
[LSP Overlay]         rust-analyzer → References (1.0)
    │
    ▼
[Confidence Fusion]   Noisy-OR merge: two 0.7 edges → one 0.94 edge
    │
    ▼
[Signal Propagation]  Impact forward, staleness backward
    │
    ▼
[Experiment Engine]   Adaptive hill-climbing with LLM advisor
```

### Workspace

| Crate | Purpose |
|-------|---------|
| `codegenome-identity` | Graph types, UOR identity, 9 overlays, multi-lang extraction (`lang/`), indexing pipeline, fusion, signal propagation, evidence, diff mapping |
| `codegenome-substrate` | Experiments engine, beliefs, embeddings, federation, governance (policy, Merkle ledger, write gate) |
| `codegenome-cli` | 14 CLI commands: `index`, `query`, `status`, `serve`, `init`, `verify`, `analyze`, `experiment`, `federate`, `visualize`, `workspace-report`, `export-prov`, `attest`, `map` |
| `codegenome-mcp` | MCP stdio tool server with 11 governed tools (7 read + 4 write-gated) |

### Layer Model

| Layer | Purpose | Modules |
|-------|---------|---------|
| **L1: Canonical Reality** | Store what exists | `graph/`, `identity/`, `store/` |
| **L2: Extraction** | Convert source evidence into graph artifacts | `lang/`, `index/`, `overlay/` |
| **L3: Composition** | Fuse observations into coherent state | `index/merger`, `confidence/fusion` |
| **L4: Query** | Make graph reality operationally usable | `graph/query`, `graph/traversal` |
| **L5: Governance** | Constrain and audit actions | `codegenome-substrate/src/governance/` |
| **L6: Evaluation** | Measure and improve substrate quality | `codegenome-substrate/src/experiments/` |

### Core Concepts

**UOR Address.** Every graph node is identified by `BLAKE3(content)`. The same code always produces the same address, regardless of toolchain version or language.

**Overlay.** A composable layer that contributes nodes and edges without knowledge of other overlays. Nine overlays exist, each implementing the `Overlay` trait.

**Observer Separation.** External systems (tree-sitter, LSP, SCIP, runtime traces) contribute observations about reality. No observer is canonical. The graph is canonical.

**Provenance.** Every non-trivial graph artifact carries: what created it, when, whether it was inferred or observed, and what evidence supports it.

## Multi-Language Support

CODEGENOME extracts code intelligence from three language families through a shared `LanguageSupport` trait:

| Language | Grammar | Extensions | Extraction |
|----------|---------|------------|------------|
| **Rust** | `tree-sitter-rust` | `.rs` | Symbols, imports, calls, impls, CFG, DFG |
| **TypeScript** | `tree-sitter-typescript` | `.ts`, `.tsx` | Functions, classes, interfaces, enums, imports, calls, class heritage, CFG, DFG |
| **Python** | `tree-sitter-python` | `.py` | Functions, classes (decorator unwrap), imports, calls, base classes, CFG, DFG |

All backends produce the same language-neutral intermediate representation (`lang/ir.rs`), which feeds into a shared graph builder. The pipeline automatically detects file languages, groups by backend, and dispatches extraction in parallel.

## Quick Start

```bash
# Clone and build
git clone https://github.com/MythologIQ-Labs-LLC/CodeGenome.git
cd CodeGenome
cargo build --workspace

# Index a mixed-language codebase
cargo run -p codegenome-cli -- index --source-dir ./src

# Query impact from a location
cargo run -p codegenome-cli -- query --file src/lib.rs --line 44

# Check index freshness
cargo run -p codegenome-cli -- status

# Start MCP server for AI assistant integration
cargo run -p codegenome-cli -- serve

# Run the self-evolving experiment engine
cargo run -p codegenome-cli -- experiment --source-dir ./src --max-iterations 100

# Verify experiment chain integrity
cargo run -p codegenome-cli -- verify
```

## Graph Composition

Nine overlay layers compose through a shared `Overlay` trait:

| Layer | Edge Types | Source | Confidence |
|-------|-----------|--------|-----------|
| Syntax | `Contains` | tree-sitter AST | 1.0 |
| Semantic | `Calls`, `Imports`, `Implements` | Multi-lang symbol resolution | 0.7–0.8 |
| Flow | `ControlFlow`, `DataFlow` | Intraprocedural AST walk | 1.0 |
| Process | `PartOfProcess` | Entrypoint BFS tracing | 0.9^depth |
| PDG | `ControlDependence`, `DataFlow` | CFG+DFG composition | 1.0 |
| Runtime | `Calls` (weighted) | TSV trace file | count/10 |
| SCIP | `References`, `Implements` | Compiler index (protobuf) | 1.0 |
| LSP | `References` | rust-analyzer subprocess (**stub** — detects the binary, contributes no edges yet) | 1.0 |
| **Fused** | **All types merged** | **Noisy-OR fusion** | **max ≤ fused ≤ 1.0** |

## Confidence Fusion

When multiple overlays produce edges between the same nodes with the same relation, their confidences fuse via noisy-OR:

```
P(fused) = 1 - ∏(1 - cᵢ)
```

Two independent 0.7 estimates become 0.91. Three become 0.973. This is the mechanism by which structural improvements to the graph produce larger accuracy gains than parameter tuning.

## Experiment Engine

### Tier Model

The engine implements a four-tier autoresearch pattern:

| Tier | Behavior | Trigger |
|------|---------|---------|
| **1 — Mechanical** | Hill-climb 3 parameters | Every iteration |
| **1.5 — Adaptive** | Widen search exponentially, restart with random params | Plateau (10 no-improvement) |
| **2 — Agent-Guided** | Local LLM (4-bit quantized) reads TSV, recommends strategy | After mechanical restart |
| **3 — Autopoietic** | Switch fitness function mid-run | LLM recommends `SWITCH_FITNESS` |

### Fitness Functions

Four fitness functions (switchable at runtime):

| Function | Measures | Range |
|----------|---------|-------|
| `ImpactAccuracy` | Sibling reachability via propagation | [0, 1] |
| `PropagationDepth` | Average max hops from sampled symbols | [0, 1] |
| `CycleTime` | Parse-propagate speed (faster = higher) | [0, 1] |
| `GraphDensity` | Edge-to-node ratio in fused overlay | [0, 1] |

### Parameters

| Parameter | Default | Range | Effect |
|-----------|---------|-------|--------|
| `confidence_threshold` | 0.75 | 0.01–0.99 | Minimum confidence to include in propagation |
| `attenuation_factor` | 0.85 | 0.1–2.0 | Signal decay per hop |
| `max_depth` | 5.0 | 1.0–20.0 | Maximum propagation depth |

## Research Tracker

### Completed Experiments

| ID | Name | Iterations | Duration | Peak Fitness | Status |
|----|------|-----------|----------|-------------|--------|
| **RUN-001** | Pre-Fusion Exhaustive Search | 224,703 | ~62 hrs | 0.896 | Complete — raw data archived in `data/runs/` |
| **RUN-002** | Post-Fusion Baseline | 20+ (early) | — | 0.825 (early) | Paused — baseline recorded, full run not yet resumed |

### Key Findings

<details>
<summary><strong>RUN-001: Pre-Fusion Engine (224,703 iterations)</strong></summary>

The first autonomous run used the three-layer engine (syntax + semantic + flow) without confidence fusion.

```
Duration:     ~62 hours (~100 iterations/minute)
Iterations:   224,703
Fitness peak: 0.896
Avg fitness:  0.59
Total KEEPs:  8 (0.004%)
Restarts:     ~7,500
```

**Trend analysis (sampled every 25K iterations):**

| Iteration | Avg Fitness | Max Fitness | KEEPs | Restarts |
|-----------|------------|------------|-------|----------|
| 0–25K | 0.591 | 0.894 | 6 | 331 |
| 25K–50K | 0.588 | 0.894 | 0 | 334 |
| 50K–75K | 0.620 | 0.894 | 0 | 333 |
| 100K–125K | 0.612 | 0.894 | 0 | 333 |
| 125K–150K | 0.565 | 0.895 | 1 | 333 |
| 175K–200K | 0.596 | 0.896 | 1 | 332 |
| 200K–225K | 0.620 | 0.896 | 0 | 334 |

*The 75K–100K and 150K–175K buckets were omitted from the original analysis; the full raw record (all 224,703 iterations) is committed at `data/runs/run-001-prefusion-224k.tsv` and can be re-derived from it.*

**Findings:**
- The fitness function has a structural ceiling determined by the graph's edge confidence distribution
- The ceiling at 0.896 was confirmed stable across 224K iterations
- The engine found it within the first 1,000 iterations and spent the remaining 223K cycling plateau→widen→restart
- **Discovery:** The engine converged on `attenuation_factor ≈ 1.3–1.5` — signal *amplification* rather than decay. This compensates for low-confidence heuristic edges. Above ~1.7, BFS scores diverge (NaN), revealing a stability boundary

</details>

<details>
<summary><strong>RUN-002: Post-Fusion Engine (in progress)</strong></summary>

After adding confidence fusion (noisy-OR), the engine was restarted with the same parameters.

```
Baseline:     0.644 (vs 0.292 pre-fusion — 2.2x higher starting point)
Early peak:   0.825 at iteration 20
Architecture: Fused overlay merges duplicate edges
```

**Findings:**
- Fusion raises the ceiling by strengthening weak edges — two 0.7 estimates fuse to 0.94
- The baseline jumped from 0.29 to 0.64 without parameter changes — purely from structural improvement
- The engine reached 0.825 in 20 iterations (vs 224K iterations at 0.896 pre-fusion)

</details>

### Phase-by-Phase Fitness Evolution

```
Phase  4 (baseline):     fitness = 0.29   ← syntax only, 1 overlay
Phase  6 (three-layer):  fitness = 0.29   ← 3 overlays, unfused
Phase  7 (adaptive):     fitness = 0.35   ← plateau detection + widen
Phase 11 (fusion):       fitness = 0.87   ← noisy-OR edge merge (breakthrough)
Run 001 (224K iters):    ceiling = 0.896  ← pre-fusion exhaustive search
Run 002 (post-fusion):   baseline = 0.64  ← fusion raises the floor
```

> **Key insight:** Structural changes to the graph (adding overlays, fusing edges) produce larger fitness improvements than parameter optimization. The engine spent 224K iterations improving parameters by 0.006. One architectural change (fusion) improved the baseline by 0.35.

### Open Research Questions

| # | Question | Hypothesis | Status |
|---|----------|-----------|--------|
| RQ-1 | What is the post-fusion fitness ceiling? | Higher than 0.896 due to stronger edge confidences | Open (RUN-002 paused) |
| RQ-2 | Does multi-language indexing affect graph density? | Cross-language call edges increase density but may reduce precision | Planned |
| RQ-3 | Can federated cross-repo edges improve impact analysis? | Cross-repo dependency edges should reduce false negatives in blast radius | Planned |
| RQ-4 | Does the autopoietic tier (fitness switching) find better optima? | Dynamic fitness landscapes prevent premature convergence | Planned |
| RQ-5 | What is the optimal confidence floor for write gating? | Too low permits unreliable mutations; too high blocks legitimate operations | Planned |

## Governance

Every graph operation is governed:

- **Merkle Ledger**: BLAKE3-chained record of all operations with cryptographic integrity
- **Evidence Log**: Tamper-evident TSV with chain verification
- **Policy Engine**: TOML rules for allow/deny/require-approval decisions
- **Write Gating**: Confidence floors and provenance requirements before mutation
- **Index Freshness**: Source hash comparison detects stale indexes
- **Snapshot Attestation**: `codegenome attest` emits an in-toto Statement with BLAKE3 digests over every store artifact; `--verify` detects tampering (the substrate-level answer to agent-memory poisoning). Sigstore signing of the statement is a release-pipeline step.
- **Standards Provenance Export**: `codegenome export-prov` maps the graph's intrinsic provenance to W3C PROV-JSON (entities/activities/agents, `wasGeneratedBy`/`wasAttributedTo`), with LLM-derived and belief-based claims explicitly labelled `cg:aiGenerated` — kept distinguishable from compiler-grade observations

```toml
# governance.toml
[[rules]]
operation = "query"
condition = "impact_nodes > 10"
action = "require-approval"
```

## Agent Memory Alignment

This repository implements the **code reality graph** — content-addressed code artifact identity, evidence and provenance bundles, confidence fusion, and impact traversal — within the [Agent Memory](https://github.com/MythologIQ-Labs-LLC/agent-memory) reference architecture. CODEGENOME supplies domain evidence and graph structure; it does not own general memory lifecycle or authority doctrine.

Canonical doctrine this work maps to:

- [ADR-005 — CodeGenome is the code reality substrate](https://github.com/MythologIQ-Labs-LLC/agent-memory/blob/main/docs/adr/ADR-005-codegenome-is-code-reality-substrate.md)
- [Source trust is a first-class signal (ADR-009)](https://github.com/MythologIQ-Labs-LLC/agent-memory/blob/main/docs/adr/ADR-009-source-trust-is-a-first-class-signal.md)
- [Repo implementation map](https://github.com/MythologIQ-Labs-LLC/agent-memory/blob/main/docs/05-repo-implementation-map.md)
- [Canonical and derived state](https://github.com/MythologIQ-Labs-LLC/agent-memory/blob/main/docs/programs/runtime-evidence/canonical-and-derived-state.md) — this repository's index-freshness check (source-hash comparison detecting stale indexes) is a working instance of the staleness-as-computable-relation design that document proposes

The governing boundary: confidence is evidence support, not lifecycle permanence. Inferred edges carry estimator provenance and preserve disagreement rather than averaging it away; graph nodes become memory units only when consumed under Agent Memory's lifecycle and authority contracts.

```text
Implementation status: declared
Conformance evidence:  none yet
```

## MCP Integration

CODEGENOME exposes 11 governed tools via the Model Context Protocol (stdio):

| Tool | Input | Output | Gate |
|------|-------|--------|------|
| `codegenome_context` | file, line | Node + neighbors | Read |
| `codegenome_impact` | file, line | Blast radius with confidence scores | Read |
| `codegenome_detect_changes` | git ref range | Affected symbols + blast radius + process impacts | Read |
| `codegenome_trace` | entrypoint name | Process call chain | Read |
| `codegenome_assert` | claim, file, line, confidence | Belief node creation with provenance | Write |
| `codegenome_reindex` | actor | Re-index source files | Write |
| `codegenome_status` | — | Index freshness report | Read |
| `codegenome_experiment_start` | source dir, iterations | Start async experiment loop | Write |
| `codegenome_experiment_status` | — | Poll experiment progress | Read |
| `codegenome_experiment_results` | last N | Recent experiment results | Read |
| `codegenome_workspace_trace` | workspace, from, to | Cross-repo workspace path trace | Read |

All responses are governed — evidence is automatically compiled and policy is evaluated before returning results. Write-gated tools pass through `WriteGatePolicy` before mutation.

```bash
# Start the MCP server
codegenome index --source-dir .
codegenome serve
```

## Project Metrics

| Metric | Value |
|--------|-------|
| Source files | 198 (Rust, ~15.3K LOC) |
| Languages supported | 3 (Rust, TypeScript, Python) |
| Tests | 264 (all passing) |
| Crates | 4 (identity + substrate + cli + mcp) |
| Overlays | 9 (8 active + LSP stub) + beliefs module |
| Edge types | 16 |
| CLI commands | 14 |
| MCP tools | 11 (7 read + 4 write-gated) |
| Fitness functions | 4 |
| Governance entries | 118 |
| Experiment data | 224K+ iterations archived in `data/runs/` |

## Dependencies

| Package | Purpose |
|---------|---------|
| `blake3` | Content-addressed hashing for UOR identity and chain integrity |
| `serde` + `postcard` + `serde_json` | Serialization for graph store, checkpoints, JSON output |
| `tree-sitter` | Incremental multi-language parsing engine |
| `tree-sitter-rust` | Rust grammar for syntax overlay |
| `tree-sitter-typescript` | TypeScript/TSX grammar for syntax overlay |
| `tree-sitter-python` | Python grammar for syntax overlay |
| `clap` | CLI argument parsing |
| `rand` | Randomized parameter perturbation in experiment loop |
| `rayon` | Parallel pipeline fan-out (per-language group) |
| `mistralrs` | Embedded local LLM for Tier 2 experiment advisor (4-bit quantized) |
| `tokio` | Async runtime for LLM inference and MCP server |
| `rmcp` | MCP protocol SDK for tool server |
| `toml` | Governance policy config parsing |
| `prost` | Protobuf decoding for SCIP index files |
| `lsp-types` | LSP protocol types for rust-analyzer integration |
| `git2` | Git repository operations for diff extraction |
| `proptest` | Property-based testing (dev only) |

## Roadmap

| Capability | Status | Location |
|------------|--------|----------|
| UOR content-addressed identity | Implemented | `identity/` |
| 9 overlay types | 8 implemented, LSP overlay stub | `overlay/` |
| Multi-language extraction (Rust + TS + Python) | Implemented | `lang/` |
| Language-neutral IR + shared graph builder | Implemented | `lang/ir.rs`, `lang/graph_builder.rs` |
| Multi-lang pipeline wiring | Implemented | `index/parser`, `index/flow`, `index/resolver_multi` |
| Confidence fusion (noisy-OR) | Implemented | `confidence/fusion.rs` |
| Signal propagation | Implemented | `signal/` |
| Karpathy Loop (Tier 1–3) | Implemented | `experiments/` |
| Secured experiment chain | Implemented | BLAKE3 chain + checkpoint |
| CLI (11 commands) | Implemented | `codegenome-cli/` |
| MCP server (11 tools) | Implemented | `codegenome-mcp/` |
| Governance (ledger + policy + write gating) | Implemented | `codegenome-substrate/src/governance/` |
| Evidence log | Implemented | `evidence/` |
| Index freshness | Implemented | `store/meta.rs` |
| Federation (cross-repo identity + symbol resolution) | Implemented | `federation/`, `graph/query_context` |
| Federated query (QueryContext polymorphism) | Implemented | `federation/query_context.rs` |
| Reasoning artifacts (beliefs as graph nodes) | Implemented | `belief/` |
| MCP assertion tool (write-gated) | Implemented | `codegenome-mcp/tools/assert_belief` |
| Impact propagation in detect_changes | Implemented | `diff/propagator`, `tools/detect` |
| W3C PROV-JSON provenance export | Implemented | `graph/prov.rs`, `export-prov` |
| In-toto index-snapshot attestation (backlog W4/W5 step 1) | Implemented (unsigned statement) | `store/attest.rs`, `attest` |
| Markdown repo-map projection | Implemented | `graph/repo_map.rs`, `map` |
| Capability broker | Planned | Awaiting FailSafe-Pro |
| LSP overlay (rust-analyzer protocol) | Planned | `overlay/lsp.rs` is a stub |
| Sigstore signing of attestations (backlog W4/W5 step 2) | Planned | release-pipeline `cosign attest` |
| IDE panels + code lenses (backlog W1) | Planned | — |

## Contributing

This is an active research project. The codebase evolves through a governance protocol (plan → audit → implement → substantiate) with cryptographic session seals. See [CONTRIBUTING.md](CONTRIBUTING.md) for build/test instructions, the pre-commit hook setup, and the PR process. Contact the maintainers before submitting significant changes.

## License

[MIT](LICENSE)

---

<p align="center">
  <em>198 files. 264 tests. 118 governance entries. 3 languages. 16 edge types. 11 MCP tools.<br>
  The graph fuses independent evidence into stronger signals.<br>
  The engine evolves. Beliefs become graph reality. Federation crosses boundaries.</em>
</p>
