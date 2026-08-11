> **SUPERSEDED (2026-08-11).** This blueprint was VETOed by audit (see docs/SHADOW_GENOME.md) and never revised; the work later shipped in modified form. Retained as historical record. File paths reference the pre-rename `codegenome-core/` layout, which no longer exists.

# Plan: Index Pipeline Refactor + Graph Traversal Engine

## Open Questions

- Should `index/parser.rs` expose a `ParsedFile` struct (file address + nodes + edges) or return `(Vec<Node>, Vec<Edge>)` tuples? Leaning toward `ParsedFile` for cache key association.
- Should the per-file cache (`index/cache.rs`) serialize with bincode (fast, existing dep) or JSON (debuggable)? Leaning bincode for consistency with `store/ondisk.rs`.

## CI Validation

```bash
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
cargo build --release
```

## Phase 1: Extract Index Modules from Overlay

Separate "what an overlay is" (data + trait) from "how it gets built" (extraction logic). All new `index/` files stay under 250 lines via targeted splits.

### Affected Files

**Tests (written/updated first):**
- `codegenome-core/src/tests/index_parser_tests.rs` — parse single file, verify node/edge counts match current `syntax.rs` output
- `codegenome-core/src/tests/index_resolver_tests.rs` — resolve uses/calls/impls, verify edge relations and confidence values
- `codegenome-core/src/tests/index_flow_tests.rs` — CFG + DFG extraction, verify edge kinds match current `flow_extract.rs` output
- `codegenome-core/src/tests/index_dynamic_tests.rs` — runtime trace ingestion, verify edges from TSV input
- `codegenome-core/src/tests/index_merger_tests.rs` — fuse overlays, verify dedup + noisy-OR confidence

**Implementation (new index modules):**
- `codegenome-core/src/index/parser.rs` — **NEW** (~90 lines): tree-sitter parsing, extracted from `overlay/syntax.rs` + `overlay/syntax_extract.rs`
- `codegenome-core/src/index/extract.rs` — **NEW** (~205 lines): tree-sitter extraction helpers (use targets, call sites, impl targets), extracted from `overlay/semantic_extract.rs`
- `codegenome-core/src/index/resolver.rs` — **NEW** (~145 lines): symbol resolution (symbol table, resolve uses/calls/impls), extracted from `overlay/semantic.rs`
- `codegenome-core/src/index/flow_cfg.rs` — **NEW** (~190 lines): control flow extraction + shared helpers (`node_span`, `block_statements`), extracted from `overlay/flow_extract.rs`
- `codegenome-core/src/index/flow_dfg.rs` — **NEW** (~100 lines): data flow extraction, extracted from `overlay/flow_dfg.rs`
- `codegenome-core/src/index/flow.rs` — **NEW** (~60 lines): thin coordinator that calls `flow_cfg` + `flow_dfg` and builds `FlowResult`
- `codegenome-core/src/index/dynamic.rs` — **NEW** (~70 lines): runtime trace ingestion, extracted from `overlay/runtime.rs`
- `codegenome-core/src/index/merger.rs` — **NEW** (~90 lines): overlay fusion, extracted from `overlay/fused.rs`
- `codegenome-core/src/index/mod.rs` — update `run_pipeline()` to call new index modules

**Overlay files become thin types:**
- `overlay/syntax.rs` — remove `parse_rust_files()` body, keep `SyntaxOverlay` struct + `Overlay` impl + re-export wrapper
- `overlay/semantic.rs` — remove `SemanticOverlay::from_syntax()` body and helpers, keep struct + impl + re-export wrapper
- `overlay/flow.rs` — remove `FlowOverlay::from_source()` body and helpers, keep struct + impl + re-export wrapper
- `overlay/fused.rs` — remove `fuse()` body and helpers, keep `FusedOverlay` struct + impl + re-export wrapper
- `overlay/runtime.rs` — remove `RuntimeOverlay::from_trace_file()` body and helpers, keep struct + impl + re-export wrapper
- `overlay/syntax_extract.rs` — becomes empty re-export of `index::parser` types (deleted after migration)
- `overlay/semantic_extract.rs` — becomes empty re-export of `index::extract` types (deleted after migration)
- `overlay/flow_extract.rs` — becomes empty re-export of `index::flow_cfg` types (deleted after migration)
- `overlay/flow_dfg.rs` — becomes empty re-export of `index::flow_dfg` types (deleted after migration)

### File Size Audit

| Proposed File | Source Lines | Estimate | Under 250? |
|---|---|---|---|
| `index/parser.rs` | syntax.rs(45L build) + syntax_extract.rs(78L extract_symbols+helpers) - shared with overlay | ~90 | Yes |
| `index/extract.rs` | semantic_extract.rs(204L) - node_span duplicate(6L) | ~200 | Yes |
| `index/resolver.rs` | semantic.rs resolution fns(~140L) | ~145 | Yes |
| `index/flow_cfg.rs` | flow_extract.rs(186L) | ~190 | Yes |
| `index/flow_dfg.rs` | flow_dfg.rs(99L) | ~100 | Yes |
| `index/flow.rs` | flow.rs builder helpers(~50L) + coordinator | ~60 | Yes |
| `index/dynamic.rs` | runtime.rs(~65L logic) | ~70 | Yes |
| `index/merger.rs` | fused.rs(~85L logic) | ~90 | Yes |

### Migration: Existing Callers

All existing callers of moved functions are preserved via re-export wrappers in `overlay/*.rs`. The wrappers delegate to the new `index/` entry points, keeping all existing import paths valid.

**`overlay/syntax.rs` — re-export `parse_rust_files`:**

```rust
/// Backward-compatible wrapper. Delegates to index::parser.
pub fn parse_rust_files(files: &[(PathBuf, Vec<u8>)]) -> SyntaxOverlay {
    SyntaxOverlay::from_parsed(&crate::index::parser::parse_files(files))
}
```

14 callers preserved:
- `index/mod.rs`, `experiments/fitness.rs`
- `tests/`: `self_index.rs`, `detect_changes_signal.rs`, `flow_tests.rs`, `fusion_tests.rs`, `pdg_tests.rs`, `process_tests.rs`, `runtime_tests.rs`, `semantic_tests.rs`, `signal_tests.rs`
- `tests/experiments/`: `fitness_tests.rs`, `overlay_isolation.rs`, `self_index_metrics.rs`

**`overlay/semantic.rs` — re-export `from_syntax`:**

```rust
impl SemanticOverlay {
    /// Backward-compatible wrapper. Delegates to index::resolver.
    pub fn from_syntax(syntax: &SyntaxOverlay, files: &[(PathBuf, Vec<u8>)]) -> Self {
        let parsed = crate::index::parser::parse_files(files);
        Self::from_resolved(&crate::index::resolver::resolve(&parsed))
    }
}
```

8 callers preserved:
- `index/mod.rs`, `experiments/fitness.rs`
- `tests/`: `semantic_tests.rs`, `flow_tests.rs`, `fusion_tests.rs`, `process_tests.rs`
- `tests/experiments/`: `fitness_tests.rs`

**`overlay/flow.rs` — re-export `from_source`:**

```rust
impl FlowOverlay {
    /// Backward-compatible wrapper. Delegates to index::flow.
    pub fn from_source(files: &[(PathBuf, Vec<u8>)]) -> Self {
        Self::from_flow_result(&crate::index::flow::extract_flow(files))
    }
}
```

7 callers preserved:
- `index/mod.rs`, `experiments/fitness.rs`
- `tests/`: `flow_tests.rs`, `fusion_tests.rs`, `pdg_tests.rs`, `process_tests.rs`
- `tests/experiments/`: `fitness_tests.rs`

**`overlay/fused.rs` — re-export `fuse`:**

```rust
/// Backward-compatible wrapper. Delegates to index::merger.
pub fn fuse(overlays: &[&dyn Overlay]) -> FusedOverlay {
    crate::index::merger::fuse(overlays)
}
```

3 callers preserved:
- `index/mod.rs`, `experiments/fitness.rs`, `tests/fusion_tests.rs`

**`overlay/runtime.rs` — re-export `from_trace_file`:**

```rust
impl RuntimeOverlay {
    /// Backward-compatible wrapper. Delegates to index::dynamic.
    pub fn from_trace_file(trace_path: &Path, source_files: &[(PathBuf, Vec<u8>)]) -> Result<Self, String> {
        let parsed = crate::index::parser::parse_files(source_files);
        Ok(Self::from_trace(&crate::index::dynamic::ingest_trace(trace_path, &parsed)?))
    }
}
```

1 caller preserved:
- `tests/runtime_tests.rs`

### Changes

**`index/parser.rs`** (~90 lines): Contains `ParsedFile` struct and parsing functions.

```rust
pub struct ParsedFile {
    pub path: PathBuf,
    pub file_address: UorAddress,
    pub content_hash: UorAddress,
    pub nodes: Vec<Node>,
    pub edges: Vec<Edge>,
}

/// Parse a single file. Pure function.
pub fn parse_file(path: &Path, source: &[u8]) -> ParsedFile

/// Parse multiple files. Creates one Parser, reuses it across files.
pub fn parse_files(files: &[(PathBuf, Vec<u8>)]) -> Vec<ParsedFile>
```

Moves `extract_symbols` call and file-node construction from `overlay/syntax.rs`. Reuses `syntax_extract::extract_symbols` logic (moved to this module). The `SyntaxOverlay` constructor becomes:

```rust
// in overlay/syntax.rs
impl SyntaxOverlay {
    pub fn from_parsed(parsed: &[ParsedFile]) -> Self
}
```

**`index/extract.rs`** (~200 lines): Tree-sitter extraction helpers shared by resolver and parser.

Moves from `overlay/semantic_extract.rs`:
- `UseTarget`, `CallSite`, `ImplTarget` structs
- `extract_use_targets()`, `extract_call_sites()`, `extract_impl_targets()`
- All private helpers: `use_leaf_name`, `walk_for_ident`, `collect_calls`, `call_callee_name`, `last_identifier`, `parse_impl_item`, `type_text`, `node_span`

**`index/resolver.rs`** (~145 lines): Symbol resolution logic.

```rust
pub struct ResolvedEdges {
    pub edges: Vec<Edge>,
}

/// Resolve semantic edges from parsed files. Pure function.
pub fn resolve(parsed: &[ParsedFile]) -> ResolvedEdges
```

Moves from `overlay/semantic.rs`:
- `build_symbol_table`, `build_span_index`
- `resolve_uses`, `resolve_calls`, `resolve_impls`
- `find_enclosing_symbol`, `symbol_node_name`, `parse_file`, `file_address`

Uses `index::extract` for tree-sitter extraction helpers.

**`index/flow_cfg.rs`** (~190 lines): Control flow extraction.

Moves from `overlay/flow_extract.rs`:
- `CfgEdge`, `CfgKind`, `DfgEdge` structs (public, used by `flow_dfg.rs`)
- `extract_control_flow()`
- All private helpers: `extract_block_flow`, `extract_control_structure`, `extract_if_flow`, `extract_match_flow`, `extract_loop_flow`, `extract_return_flow`, `block_statements`, `node_span`

**`index/flow_dfg.rs`** (~100 lines): Data flow extraction.

Moves from `overlay/flow_dfg.rs`:
- `extract_data_flow()`
- All private helpers: `extract_fn_data_flow`, `collect_let_defs`, `collect_ident_uses`, `is_definition_site`, `node_span`

Uses `index::flow_cfg::DfgEdge` for the return type.

**`index/flow.rs`** (~60 lines): Thin coordinator.

```rust
pub struct FlowResult {
    pub nodes: Vec<Node>,
    pub edges: Vec<Edge>,
}

/// Extract control flow and data flow. Pure function.
pub fn extract_flow(files: &[(PathBuf, Vec<u8>)]) -> FlowResult
```

Moves from `overlay/flow.rs`:
- `stmt_node_pair`, `ensure_node`, `stmt_address`, `parse_file`
- Orchestration: for each file, call `flow_cfg::extract_control_flow` + `flow_dfg::extract_data_flow`, build nodes/edges

**`index/dynamic.rs`** (~70 lines): Runtime trace ingestion.

```rust
pub struct TraceResult {
    pub edges: Vec<Edge>,
}

/// Ingest a TSV trace file. Pure function.
pub fn ingest_trace(trace_path: &Path, parsed: &[ParsedFile]) -> Result<TraceResult, String>
```

Moves from `overlay/runtime.rs`:
- `parse_trace_line`, `build_name_index`, `parse_file`

**`index/merger.rs`** (~90 lines): Overlay fusion.

```rust
/// Fuse overlays. Dedup nodes by address, merge edges via noisy-OR.
pub fn fuse(overlays: &[&dyn Overlay]) -> FusedOverlay
```

Moves from `overlay/fused.rs`:
- `dedup_nodes`, `fuse_edges`, `merge_edge_group`

**`index/mod.rs`**: Update `run_pipeline()` to chain through new modules:

```rust
pub mod cache;
pub mod dynamic;
pub mod extract;
pub mod flow;
pub mod flow_cfg;
pub mod flow_dfg;
pub mod merger;
pub mod orchestrator;
pub mod parser;
pub mod resolver;
```

`run_pipeline()` becomes: `parser::parse_files` → `SyntaxOverlay::from_parsed` → `resolver::resolve` → `SemanticOverlay::from_resolved` → `flow::extract_flow` → `FlowOverlay::from_flow_result` → `merger::fuse` → store.

### Unit Tests

- `index_parser_tests.rs` — parse a 3-function Rust file, assert node count = 3 symbols + 1 file, edge count = 3 Contains edges. Verify `ParsedFile.content_hash` is deterministic.
- `index_resolver_tests.rs` — two files with cross-file call, assert Calls edge exists with confidence 0.7. File with `use` statement, assert Imports edge. Impl block, assert Implements edge.
- `index_flow_tests.rs` — function with if/else, assert Branch CFG edges. Function with let + use, assert DataFlow edge. Verify flow_cfg and flow_dfg produce consistent results when called through `flow::extract_flow`.
- `index_dynamic_tests.rs` — synthetic TSV input, assert Calls edges with confidence = min(count/10, 1.0).
- `index_merger_tests.rs` — two overlays with same edge at different confidences, assert fused confidence = noisy-OR result.


## Phase 2: Orchestrator with Rayon Fan-Out + Incremental Cache

### Affected Files

**Tests:**
- `codegenome-core/src/tests/index_orchestrator_tests.rs` — pipeline produces same fused output as Phase 1's sequential path; incremental re-index skips unchanged files
- `codegenome-core/src/tests/index_cache_tests.rs` — cache hit/miss behavior, dirty file detection

**Implementation:**
- `codegenome-core/src/index/orchestrator.rs` — **NEW**: pipeline orchestration with dependency DAG + rayon
- `codegenome-core/src/index/cache.rs` — **NEW**: per-file content hash cache for incremental indexing
- `codegenome-core/src/index/mod.rs` — `run_pipeline()` delegates to `orchestrator::run()`
- `codegenome-core/Cargo.toml` — add `rayon = "1"` dependency

### Changes

**`index/cache.rs`** (~80 lines): Per-file overlay cache.

```rust
pub struct FileCache {
    root: PathBuf,
}

pub struct CachedFile {
    pub content_hash: String,  // BLAKE3 truncated hex
    pub parsed: ParsedFile,
}

impl FileCache {
    pub fn new(store_dir: &Path) -> Self
    /// Load cached parse result. Returns None on miss or hash mismatch.
    pub fn get(&self, path: &Path, current_hash: &str) -> Option<CachedFile>
    /// Store parse result.
    pub fn put(&self, path: &Path, cached: &CachedFile) -> Result<(), String>
}
```

Serialization uses bincode. Cache directory: `{store_dir}/file_cache/`. One file per source file, keyed by path hash.

**`index/orchestrator.rs`** (~120 lines): Pipeline with explicit stages.

```rust
pub struct PipelineConfig {
    pub source_dir: PathBuf,
    pub store_dir: PathBuf,
    pub trace_path: Option<PathBuf>,
}

pub fn run(config: &PipelineConfig) -> Result<IndexResult, String>
```

Execution flow:
1. Collect source files, compute BLAKE3 hashes
2. Load `FileCache`, partition files into dirty/clean
3. Parse dirty files via `parser::parse_files`, load clean from cache
4. Store newly parsed files in cache
5. Build `SyntaxOverlay` from all `ParsedFile`s
6. **Rayon fan-out** (syntax complete, then parallel):
   - `resolver::resolve(parsed)` → `SemanticOverlay`
   - `flow::extract_flow(files)` → `FlowOverlay`
   - If `trace_path` provided: `dynamic::ingest_trace()` → `RuntimeOverlay`
7. **Barrier**: collect all overlays
8. `merger::fuse()` → `FusedOverlay`
9. Write to `OnDiskStore`, update `IndexMeta`

Rayon fan-out uses `rayon::scope`:

```rust
rayon::scope(|s| {
    s.spawn(|_| { /* resolver */ });
    s.spawn(|_| { /* flow */ });
    s.spawn(|_| { /* dynamic trace */ });
});
```

**`index/mod.rs`**: `run_pipeline()` becomes a thin wrapper:

```rust
pub fn run_pipeline(source_dir: &Path, store_dir: &Path) -> Result<IndexResult, String> {
    orchestrator::run(&PipelineConfig {
        source_dir: source_dir.to_path_buf(),
        store_dir: store_dir.to_path_buf(),
        trace_path: None,
    })
}
```

Preserves existing call sites (CLI, MCP tools).

### Unit Tests

- `index_orchestrator_tests.rs` — index a small fixture directory, verify `IndexResult` counts. Re-index without changes, verify `is_fresh == true`. Modify one file, re-index, verify only that file is re-parsed (assert via node count delta).
- `index_cache_tests.rs` — put a `CachedFile`, get it back, assert equality. Change content hash, assert cache miss. Verify cache survives across `FileCache` instances (persistence).


## Phase 3: Graph Traversal Engine + Extractions

### Affected Files

**Tests:**
- `codegenome-core/src/tests/traversal_tests.rs` — BFS traversal, direction filtering, confidence thresholds, path collection
- `codegenome-core/src/tests/confidence_fusion_tests.rs` — migrated from `confidence/mod.rs` inline tests + new edge cases
- `codegenome-core/src/tests/propagator_tests.rs` — diff → symbol mapping → impact/staleness propagation end-to-end

**Implementation:**
- `codegenome-core/src/graph/traversal.rs` — **NEW** (~120 lines): query execution on `&[Node]` + `&[Edge]`
- `codegenome-core/src/graph/mod.rs` — add `pub mod traversal;` and re-exports
- `codegenome-core/src/confidence/fusion.rs` — **NEW** (~40 lines): extracted from `confidence/mod.rs`
- `codegenome-core/src/confidence/mod.rs` — becomes re-export hub
- `codegenome-core/src/diff/propagator.rs` — **NEW** (~45 lines): adapter connecting diff → overlays → signals

### Changes

**`graph/traversal.rs`** (~120 lines): Query execution engine. Operates on slices.

```rust
/// Execute a query against graph data. Pure function.
pub fn execute(
    query: &Query,
    nodes: &[Node],
    edges: &[Edge],
) -> QueryResult
```

Implementation:
1. Build adjacency index from edges (respecting `query.direction`)
2. BFS from `query.target` up to `query.max_depth`
3. Filter edges by `query.relation_filter` if set
4. Prune paths below `query.min_confidence`
5. Collect reachable nodes, traversed edges, discovered paths
6. Compute overall confidence via `path_confidence`

Internal helpers:
- `build_adj_index(edges, direction) -> HashMap<UorAddress, Vec<(UorAddress, f64, Relation)>>`
- `bfs_paths(target, adj, max_depth, min_confidence) -> Vec<Vec<UorAddress>>`

Reuses existing `Direction` enum from `graph/query.rs`.

**`confidence/fusion.rs`** (~40 lines): Move `path_confidence`, `multi_path_confidence`, `impact_score` and their inline tests from `confidence/mod.rs`.

**`confidence/mod.rs`**: Becomes:

```rust
pub mod fusion;
pub use fusion::{path_confidence, multi_path_confidence, impact_score};
```

Existing call sites unchanged.

**`diff/propagator.rs`** (~45 lines): Bridges diff → signal modules.

```rust
/// Map a diff to affected symbols, propagate impact and staleness.
/// Thin adapter: delegates to signal::impact + signal::staleness.
pub fn propagate(
    diff: &OwnedDiff,
    overlays: &[&dyn Overlay],
) -> ChangeSet
```

Extracts orchestration logic from `diff/mapper.rs::detect_changes`. The `detect_changes` function in `mapper.rs` delegates to `propagator::propagate`. `find_changed_nodes` and `hunk_overlaps_span` stay in `mapper.rs`.

### Unit Tests

- `traversal_tests.rs`:
  - Linear chain A→B→C, downstream from A: returns all 3 nodes, 2 edges, 1 path
  - Same chain, upstream from C: returns all 3 nodes, path C←B←A
  - Diamond graph, verify both paths discovered
  - `max_depth=1` from A: returns only A,B
  - `min_confidence=0.5` with a 0.3 edge: path pruned
  - `relation_filter=Some(vec![Calls])`: non-Calls edges excluded
- `confidence_fusion_tests.rs`:
  - Migrate existing 2 tests from `confidence/mod.rs`
  - Add: single path confidence = product of edges
  - Add: multi-path with 3 independent paths, verify noisy-OR
  - Add: impact_score boundary values (0.0, 1.0, >1.0 clamping)
- `propagator_tests.rs`:
  - Synthetic diff touching one function, verify it appears in `changed_nodes`
  - Verify downstream impact scores attenuate by edge confidence
  - Verify upstream staleness propagates backward
