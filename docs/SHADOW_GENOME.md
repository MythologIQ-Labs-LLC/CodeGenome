# Shadow Genome

## Failure Entry #1

**Date**: 2026-04-01T23:45:00Z
**Verdict ID**: AUDIT_REPORT.md @ 2026-04-01T23:45:00Z
**Failure Mode**: ORPHAN

### What Failed
`codegenome-mcp/src/server.rs` — MCP server lifecycle module

### Why It Failed
The MCP server module exists in the blueprint but has no reachable entry point. The CLI crate defines `{index, query, status, verify}` commands but no `serve` command. The `codegenome-mcp` crate has no binary target (`[[bin]]`). The server cannot be started.

### Pattern to Avoid
When designing a server/daemon module, always define the startup path explicitly — either as a binary target in the crate or as a CLI subcommand that imports and runs it. Modules without entry points are dead code.

### Remediation Attempted
Pending — requires Governor to add `serve` command to CLI or binary target to MCP crate.

---

## Failure Entry #2

**Date**: 2026-04-01T23:45:00Z
**Verdict ID**: AUDIT_REPORT.md @ 2026-04-01T23:45:00Z
**Failure Mode**: HALLUCINATION

### What Failed
ARCHITECTURE_PLAN.md dependency table — listed 5 dependencies, omitted 3 required by the blueprint's own interface contracts.

### Why It Failed
The diff::mapper contract specifies "Git diff → ranges → symbols" requiring `git2`. The MCP server contract specifies "JSON-RPC" requiring an MCP/RPC library. The on-disk graph store requires binary serialization beyond `serde_json`. These were not listed, creating a false picture of the project's dependency surface for an L3 security assessment.

### Pattern to Avoid
For L3 projects, the dependency table must account for every external library implied by interface contracts. Incomplete dependency documentation masks supply-chain risk.

### Remediation Attempted
Pending — requires Governor to update dependency table with git2, MCP SDK, and binary serialization crate.

---

## Failure Entry #3

**Date**: 2026-04-03T08:00:00Z
**Verdict ID**: AUDIT_REPORT.md @ 2026-04-03T08:00:00Z
**Failure Mode**: GHOST_PATH

### What Failed
`codegenome-cli/src/main.rs` lines 41-45 — 5 CLI commands (index, query, status, verify, serve) exposed to users but resolving to `commands::stub()`.

### Why It Failed
The Phase 4 implementation added the CLI binary with all 7 commands from ARCHITECTURE_PLAN.md, but only implemented `experiment`. The remaining 5 were wired to a `stub()` function that prints "not yet implemented." A user running `codegenome index` receives a useless response. Ghost commands in an L3 binary with governance and security claims are unacceptable — they create a false surface area.

### Pattern to Avoid
Never expose user-facing commands that are not implemented. If functionality is planned for future phases, do not add it to the command enum until it is ready. The CLI should only advertise what it can deliver.

### Remediation Attempted
Pending — requires Governor to either remove stub commands or implement them.

---

## Failure Entry #4

**Date**: 2026-04-03T08:00:00Z
**Verdict ID**: AUDIT_REPORT.md @ 2026-04-03T08:00:00Z
**Failure Mode**: HALLUCINATION

### What Failed
ARCHITECTURE_PLAN.md dependency table — lists 8 dependencies, omits 2 that are actively used in the codebase (`rand`, `tree-sitter-rust`).

### Why It Failed
`rand` was approved in the Phase 4 Gate Tribunal (Entry #17) but the blueprint's dependency table was never updated. `tree-sitter-rust` is the Rust grammar companion required by the syntax overlay but was never listed. This repeats the exact failure mode from Shadow Genome Entry #2: "incomplete dependency documentation masks supply-chain risk." The pattern was documented but not prevented.

### Pattern to Avoid
When a Gate Tribunal approves new dependencies, the ARCHITECTURE_PLAN.md dependency table MUST be updated in the same phase. Dependencies implied by companion libraries (e.g., tree-sitter grammars) must be explicitly listed for L3 projects. Shadow Genome lessons must be actively enforced, not merely recorded.

### Remediation Attempted
Pending — requires Governor to update dependency table with `rand` and `tree-sitter-rust` entries.

---

## Failure Entry #5

**Date**: 2026-04-04T03:15:00Z
**Verdict ID**: AUDIT_REPORT.md @ 2026-04-04T03:15:00Z
**Failure Mode**: COMPLEXITY_VIOLATION

### What Failed
`plan-phase7-adaptive-engine.md` Phase 2 — proposed changes to `experiments/runner.rs::run_continuous` would push function from 35 to ~45 lines.

### Why It Failed
The plan added review logic (initialization + assess call + 3-variant match + restart/widen) to an already 35-line function without accounting for the 40-line limit. Adding ~10 lines of control flow inside a loop body crosses the threshold.

### Pattern to Avoid
When a plan proposes adding logic to an existing function, perform line-level accounting against Section 4 limits. If the function is already above 30 lines, any non-trivial addition likely requires a split. Plans must specify the split strategy, not leave it to the Specialist.

### Remediation Attempted
Pending — requires Governor to revise plan with explicit split strategy for `run_continuous`.

---

## Failure Entry #6

**Date**: 2026-04-04T05:15:00Z
**Verdict ID**: AUDIT_REPORT.md @ 2026-04-04T05:15:00Z
**Failure Mode**: HALLUCINATION

### What Failed
`plan-phase8-tier2-llm.md` — adds `mistralrs` dependency to Cargo.toml without updating ARCHITECTURE_PLAN.md dependency table.

### Why It Failed
Third occurrence of the same pattern (Shadow Genome #2, #4). Plans add dependencies to Cargo.toml but do not include updating the blueprint's dependency table. For L3 projects, every external library must be documented.

### Pattern to Avoid
When a plan introduces ANY new dependency, it MUST include a step to update ARCHITECTURE_PLAN.md dependency table with the package name, justification, and vanilla alternative. This check should be automatic in plan authoring — not caught only by audit.

### Remediation Attempted
Pending — requires Governor to add blueprint table update step to plan.

---
## Failure Entry #7

**Date**: 2026-04-06T17:28:57.3545359Z
**Verdict ID**: AUDIT_REPORT.md @ 2026-04-06T17:28:57.3545359Z
**Failure Mode**: HALLUCINATION

### What Failed
`plan-experiment-analytics-and-federation.md` - repo-local analytics and workspace federation blueprint

### Why It Failed
The plan requires parameter correlations from TSV data via `log::read_log()` even though the current experiment log format and parser do not persist parameter values and reconstruct `params` as empty for every row. It also adds a new `codegenome-core/src/federation/*` module tree and a new MCP tool without carrying the crate-root export path and server registration path needed to make them reachable.

### Pattern to Avoid
Do not plan derived analytics on fields that the current data surface does not actually encode. Do not propose new module trees or MCP tools without tracing their full build path through crate roots and server registration points.

### Remediation Attempted
Pending - requires Governor to revise the blueprint so data provenance and build connectivity are explicit in the plan.

---

## Failure #3: File Size Violations + Missing Migration Path

**Date**: 2026-04-06
**Ledger Entry**: #89
**Blueprint**: plan-index-pipeline-refactor.md (Index Pipeline Refactor + Graph Traversal Engine)
**Verdict**: VETO (3 violations)

### What Failed

V1/V2: Plan proposed merging multiple extraction files into single `index/` modules without counting lines. `index/flow.rs` would hit ~335 lines (flow_extract 186 + flow_dfg 99 + helpers 50). `index/resolver.rs` would hit ~344 lines (semantic_extract 204 + resolution helpers 140). Both exceed the 250-line Section 4 Razor limit.

V3: Plan stripped constructors from overlay types (`parse_rust_files`, `from_syntax`, `from_source`, `fuse`) but failed to account for 30+ existing callers in test and production code. No migration path specified — Phase 1 as written would break compilation.

### Why It Failed

The Governor counted modules but did not count lines when planning file merges. The refactor plan focused on the new module structure without auditing the existing call graph for functions being moved.

### Pattern to Avoid

When planning code extraction/migration: (1) always sum source line counts before proposing merges, (2) always grep for all callers of functions being moved and specify their migration path in the plan.

### Remediation Attempted

Pending — requires Governor to split oversized modules and add caller migration section.

---

## Failure #4: Duplicate Domain Type + Missing Test Infrastructure

**Date**: 2026-04-07
**Ledger Entry**: #93
**Blueprint**: plan-mcp-claude-code-integration.md (MCP Claude Code Integration)
**Verdict**: VETO (2 violations)

### What Failed

V1: Plan proposed `WriteGateDecision { Allow, Deny }` in `governance/write_gate.rs` while `governance/policy.rs` already defines `Decision { Allow, Deny, RequireApproval }`. Two decision enums for the same concept in the same governance module.

V2: Plan proposed test files in `codegenome-mcp/src/tests/` but the MCP crate has no test module infrastructure — no `#[cfg(test)] mod tests;` in `lib.rs`, no `tests/mod.rs`. Test files would be orphans.

### Why It Failed

The Governor designed the write gate as a standalone subsystem without checking what types already existed in the same module namespace. Similarly, proposed tests in a crate without verifying the crate had test infrastructure.

### Pattern to Avoid

When adding new types to an existing module: (1) check for existing types with overlapping semantics — reuse before creating. When proposing tests in a crate: (2) verify the crate has `#[cfg(test)]` module registration and include it in the plan if missing.

### Remediation Attempted

Pending — requires Governor to reuse `governance::policy::Decision` and add MCP test module infrastructure to the plan.

---

## Failure #5: File Size + Missing Migration (REPEAT of #3)

**Date**: 2026-04-07
**Ledger Entry**: #99
**Blueprint**: plan-multi-language-support.md (Multi-Language Support)
**Verdict**: VETO (2 violations)
**Repeat**: Shadow Genome #3 pattern (file size violation + missing caller migration)

### What Failed

V1: `lang/rust.rs` claims ~200 lines but absorbs extract.rs (204L) + flow_cfg.rs (188L) + flow_dfg.rs (100L) = 492L source. Even factoring out IR types, function bodies total ~345L.

V2: Plan says logic "moves from" index/extract.rs, index/flow_cfg.rs, index/flow_dfg.rs but doesn't specify what happens to these modules or their 9+ callers.

### Why It Failed

Same root cause as Failure #3: the Governor did not sum source lines before proposing merges, and did not grep callers before proposing moves. This is a **repeat failure** despite the pattern being documented.

### Pattern to Avoid

MANDATORY before any plan that moves code between modules:
1. `wc -l` every source file involved
2. Sum and verify against 250-line limit
3. `grep` every moved function/module for callers
4. List every caller and its migration path in the plan

### Remediation Attempted

Pending — requires Governor to split rust.rs and add migration section.

---
## Failure #6: Falsified Session Seal — Incomplete Directory Rename

**Date**: 2026-08-11
**Ledger Entry**: #118 (the seal being corrected)
**Blueprint**: session-5 (crate split / rename)
**Verdict**: Seal claimed "SUBSTANTIATED. Reality = Promise." Reality was not Promise.

### What Failed

META_LEDGER Entry #118 seals "Directory rename: `codegenome-core/` → `codegenome-substrate/`" as substantiated. The copy happened; the delete did not. `codegenome-core/` remained tracked in git as a 56-file, 4,316-LOC byte-identical duplicate of `codegenome-substrate/` — outside the workspace, never compiled, silently rotting, and inflating every file/test/LOC metric derived from the tree. Its manifest even declared `name = "codegenome-substrate"`, a package-name collision.

### Why It Failed

Substantiation verified the presence of the new tree but never verified the absence of the old one. `git status` was clean because the old files were still tracked, unchanged. No automated check compared the workspace member list against directories on disk.

### Pattern to Avoid

A rename is two operations: create AND delete. Substantiation of any move/rename must verify both sides — assert the destination exists and the source is gone (`git ls-files <old-path>` returns empty). Metrics quoted in seals must be computed from the workspace member list, not the raw tree.

### Remediation Attempted

**COMPLETED** (2026-08-11): `codegenome-core/` deleted in full (commit `5914380`, branch `claude/agent-memory-repo-review-2tay4p`). This entry serves as the ledger correction for seal #118. First Shadow Genome entry with a closed remediation; the remaining "Pending" entries above still require triage.

---
_Shadow Genome tracks failure patterns to prevent repetition._
