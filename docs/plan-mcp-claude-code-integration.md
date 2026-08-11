> **SUPERSEDED (2026-08-11).** This blueprint was VETOed by audit (see docs/SHADOW_GENOME.md) and never revised; the work later shipped in modified form. Retained as historical record. File paths reference the pre-rename `codegenome-core/` layout, which no longer exists.

# Plan: MCP Claude Code Integration — Read-Open, Write-Gated

## Open Questions

- Should the confidence floor for write gating be a compile-time constant or a runtime config value? Leaning runtime (CLI flag or config file) so teams can tune without rebuilding.
- Should `resolve_address` match file paths by suffix (e.g. `src/lib.rs` matches `/full/path/to/src/lib.rs`) or require exact path? Leaning suffix match for ergonomics.

## CI Validation

```bash
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
cargo build --release
```

## Phase 1: Core Foundations — Resolution, Git Bridge, Write Gate Policy

Wire the traversal engine into a usable query path and add the domain-level governance policy for write operations.

### Affected Files

**Tests:**
- `codegenome-core/src/tests/resolve_tests.rs` — file+line → UorAddress resolution against fused overlay nodes
- `codegenome-core/src/tests/git_bridge_tests.rs` — git2 diff → OwnedDiff conversion with real and synthetic diffs
- `codegenome-core/src/tests/write_gate_tests.rs` — WriteGatePolicy evaluation: allow, deny, below-confidence rejection

**Implementation:**
- `codegenome-core/src/graph/resolve.rs` — **NEW** (~60 lines): `resolve_address(nodes, file, line) -> Option<UorAddress>`
- `codegenome-core/src/graph/mod.rs` — add `pub mod resolve;` and re-export
- `codegenome-core/src/diff/git_bridge.rs` — **NEW** (~120 lines): git2 diff extraction → `OwnedDiff`
- `codegenome-core/src/diff/mod.rs` — add `pub mod git_bridge;`
- `codegenome-core/src/governance/write_gate.rs` — **NEW** (~100 lines): `WriteGatePolicy` + `WriteGateDecision`
- `codegenome-core/src/governance/mod.rs` — add `pub mod write_gate;`
- `codegenome-core/Cargo.toml` — add `git2 = "0.19"` dependency

### Changes

**`graph/resolve.rs`** (~60 lines): Address resolution from human-readable inputs.

```rust
/// Resolve a file path + line number to a UorAddress.
/// Matches by path suffix (e.g. "src/lib.rs" matches stored full paths).
pub fn resolve_address(
    nodes: &[Node],
    file: &str,
    line: u32,
) -> Option<UorAddress>
```

Logic:
1. Filter nodes that have a span
2. Match nodes whose originating file path ends with `file`
3. Among matches, find the node whose span contains `line`
4. Return its `UorAddress`

Requires file path to be recoverable from nodes. Currently nodes don't store their file path directly — the file path is encoded in the `file:{}` content-addressed `UorAddress`. Resolution approach: accept a `file_index: &HashMap<UorAddress, PathBuf>` built from `ParsedFile`s, or scan for File nodes whose address matches `address_of(format!("file:{}", path))`.

Simpler v1: the MCP tool builds a `(PathBuf, Vec<Node>)` index at load time from the stored overlay + the source directory. `resolve_address` takes this index as input.

```rust
pub struct FileIndex {
    entries: Vec<(PathBuf, Vec<NodeRef>)>,
}

impl FileIndex {
    pub fn build(source_dir: &Path, nodes: &[Node]) -> Self
    pub fn resolve(&self, file: &str, line: u32) -> Option<UorAddress>
}
```

`NodeRef` is `(UorAddress, Span)` — lightweight, no cloning full nodes.

**`diff/git_bridge.rs`** (~120 lines): Git diff to `OwnedDiff` conversion.

```rust
/// Extract OwnedDiff from a git repository between two refs.
/// If `to` is None, diffs against the working tree.
pub fn git_diff(
    repo_path: &Path,
    from: Option<&str>,
    to: Option<&str>,
) -> Result<OwnedDiff, String>
```

Logic:
1. Open repo via `git2::Repository::open(repo_path)`
2. Resolve `from` ref (default: HEAD) to a tree
3. If `to` is None: diff tree against workdir. If `to` is Some: diff tree against tree.
4. Walk `git2::Diff` deltas and hunks, convert to `OwnedDiffFile` + `OwnedHunk`
5. Map `git2::Delta` status to `DiffStatus`

**`governance/write_gate.rs`** (~80 lines): Write privilege evaluation. Reuses existing `governance::policy::Decision` — no new decision enum.

```rust
use crate::governance::policy::Decision;

pub struct WriteGatePolicy {
    pub confidence_floor: f64,
    pub require_provenance: bool,
    pub require_freshness: bool,
}

pub struct WriteRequest {
    pub actor: String,
    pub toolchain_version: String,
    pub source_freshness: FreshnessReport,
    pub min_edge_confidence: f64,
}

impl WriteGatePolicy {
    pub fn default_policy() -> Self
    pub fn evaluate(&self, request: &WriteRequest) -> Decision
}
```

Returns `Decision::Allow`, `Decision::Deny(reason)`, or `Decision::RequireApproval(reason)`. Evaluation rules:
1. If `require_provenance` and actor is empty → `Deny("missing provenance")`
2. If `require_freshness` and source is stale → `Deny("index stale, reindex first")`
3. If `min_edge_confidence < confidence_floor` → `Deny("confidence {x} below floor {y}")`
4. Otherwise → `Allow`

### Unit Tests

- `resolve_tests.rs`:
  - Build FileIndex from 3-file fixture, resolve known file+line → correct UorAddress
  - Resolve with suffix match ("lib.rs" matches "src/lib.rs")
  - Resolve unknown file → None
  - Resolve line outside any span → None
- `git_bridge_tests.rs`:
  - Create temp git repo, add file, commit, modify, call `git_diff` → OwnedDiff with 1 Modified file
  - Diff with no changes → empty OwnedDiff
  - Hunk line numbers match the actual modification
- `write_gate_tests.rs`:
  - Default policy + valid request → `Decision::Allow`
  - Empty actor with require_provenance → `Decision::Deny` with reason containing "provenance"
  - Stale freshness with require_freshness → `Decision::Deny` with reason containing "stale"
  - Confidence 0.3 with floor 0.5 → `Decision::Deny` with reason containing both values
  - Confidence 0.8 with floor 0.5 → `Decision::Allow`


## Phase 2: MCP Tool Upgrade — Typed Inputs, Traversal, Write Enforcement

Rewrite MCP tools to use typed input schemas, wire traversal engine for reads, and enforce write gate on mutations.

### Affected Files

**Tests:**
- `codegenome-mcp/src/tests/mod.rs` — **NEW**: test module registration (required — MCP crate has no test infrastructure)
- `codegenome-mcp/src/tests/tool_schema_tests.rs` — verify all tool schemas are valid JSON Schema
- `codegenome-mcp/src/tests/write_enforcement_tests.rs` — reindex with stale source is rejected, reindex with valid state succeeds

**Implementation:**
- `codegenome-mcp/src/lib.rs` — add `#[cfg(test)] mod tests;` (MCP crate currently has no test module)
- `codegenome-mcp/src/tools/inputs.rs` — **NEW** (~120 lines): typed input structs with `#[derive(Deserialize, JsonSchema)]`
- `codegenome-mcp/src/tools/gate.rs` — **NEW** (~70 lines): MCP-layer write gate enforcement
- `codegenome-mcp/src/tools/mod.rs` — add `source_dir` to `CodegenomeTools`, update `load_overlay` to build `FileIndex`
- `codegenome-mcp/src/tools/context.rs` — rewrite: resolve → traverse → format with provenance
- `codegenome-mcp/src/tools/impact.rs` — rewrite: resolve → propagate → format with confidence
- `codegenome-mcp/src/tools/detect.rs` — rewrite: use `git_bridge::git_diff` instead of raw diff text
- `codegenome-mcp/src/tools/reindex.rs` — add write gate check before pipeline
- `codegenome-mcp/src/tools/trace.rs` — wire through traversal with relation filter
- `codegenome-mcp/src/server.rs` — register typed schemas, update dispatch to deserialize input structs
- `codegenome-mcp/Cargo.toml` — add `git2 = "0.19"` (needed for detect tool's git bridge)

### Changes

**`tools/inputs.rs`** (~120 lines): One struct per tool.

```rust
#[derive(Deserialize, JsonSchema)]
pub struct ContextInput {
    /// File path (relative to source root)
    pub file: String,
    /// Line number
    pub line: u32,
    /// Traversal depth (default: 1)
    #[serde(default = "default_depth")]
    pub depth: u32,
    /// Traversal direction: "downstream", "upstream", or "both"
    #[serde(default = "default_direction")]
    pub direction: String,
}

#[derive(Deserialize, JsonSchema)]
pub struct ImpactInput {
    pub file: String,
    pub line: u32,
}

#[derive(Deserialize, JsonSchema)]
pub struct DetectInput {
    /// Git ref to diff from (default: HEAD)
    #[serde(default = "default_head")]
    pub from_ref: String,
    /// Git ref to diff to (default: working tree)
    pub to_ref: Option<String>,
}

#[derive(Deserialize, JsonSchema)]
pub struct ReindexInput {
    /// Actor name for provenance (default: "claude-code")
    #[serde(default = "default_actor")]
    pub actor: String,
}

#[derive(Deserialize, JsonSchema)]
pub struct TraceInput {
    pub entrypoint: String,
    #[serde(default = "default_depth")]
    pub max_depth: u32,
}

#[derive(Deserialize, JsonSchema)]
pub struct StatusInput {}
```

**`tools/gate.rs`** (~70 lines): Write enforcement adapter.

```rust
use codegenome_core::governance::policy::Decision;
use codegenome_core::governance::write_gate::*;

pub fn check_write_privilege(
    source_dir: &Path,
    store_dir: &Path,
    actor: &str,
) -> Result<(), String>
```

Logic:
1. Build `WriteRequest` from actor, toolchain version (`env!("CARGO_PKG_VERSION")`), freshness check
2. Call `WriteGatePolicy::default_policy().evaluate(&request)`
3. If `Decision::Deny(reason)` → return `Err(reason)`
4. If `Decision::RequireApproval(reason)` → return `Err(format!("approval required: {reason}"))`
5. If `Decision::Allow` → return `Ok(())`

**`tools/mod.rs`**: Update `CodegenomeTools`:

```rust
pub struct CodegenomeTools {
    pub source_dir: PathBuf,
    pub store_dir: PathBuf,
    pub run_manager: Arc<Mutex<RunManager>>,
}
```

Add `source_dir` field. Update constructor: `fn new(source_dir, store_dir)`. Add helper:

```rust
pub fn load_with_index(&self) -> Option<(StoredOverlay, FileIndex)>
```

Loads the fused overlay and builds a `FileIndex` from source_dir + nodes.

**`tools/context.rs`** rewrite: resolve → traverse → format.

```rust
pub fn context(&self, input: &ContextInput) -> String {
    let (overlay, index) = self.load_with_index()?;
    let addr = index.resolve(&input.file, input.line)?;
    let direction = parse_direction(&input.direction);
    let query = Query { target: addr, direction, max_depth: input.depth, .. };
    let result = traversal::execute(&query, overlay.nodes(), overlay.edges());
    format_with_provenance(&result, &overlay)
}
```

Response includes provenance and confidence per node.

**`tools/impact.rs`** rewrite: same resolve → propagate pattern, but calls `propagate_impact` and formats with confidence scores and provenance.

**`tools/detect.rs`** rewrite: uses `git_bridge::git_diff` with `source_dir` as repo path.

```rust
pub fn detect(&self, input: &DetectInput) -> String {
    let diff = git_bridge::git_diff(
        &self.source_dir,
        Some(&input.from_ref),
        input.to_ref.as_deref(),
    )?;
    // ... existing detect_changes logic
}
```

**`tools/reindex.rs`** rewrite: write gate check before pipeline.

```rust
pub fn reindex(&self, input: &ReindexInput) -> String {
    if let Err(reason) = gate::check_write_privilege(
        &self.source_dir, &self.store_dir, &input.actor,
    ) {
        return json!({"error": "write denied", "reason": reason});
    }
    // ... existing pipeline call
}
```

**`server.rs`**: Update `make_tool` to use `schemars::schema_for!` for each input struct. Update `dispatch_tool` to deserialize args into typed structs.

```rust
fn make_typed_tool<T: JsonSchema>(name: &str, desc: &str) -> Tool {
    let schema = schemars::schema_for!(T);
    // Convert to serde_json::Map for rmcp
}

fn dispatch_tool(tools: &CodegenomeTools, req: &CallToolRequestParams) -> CallToolResult {
    match req.name.as_ref() {
        "codegenome_context" => {
            let input: ContextInput = deserialize_args(req)?;
            tools.context(&input)
        }
        // ...
    }
}
```

### Unit Tests

- `tool_schema_tests.rs`:
  - Each input struct produces valid JSON Schema (no panic on `schema_for!`)
  - ContextInput has required `file` and `line` fields
  - DetectInput `from_ref` defaults to "HEAD"
  - ReindexInput `actor` defaults to "claude-code"
- `write_enforcement_tests.rs`:
  - `check_write_privilege` with valid actor + fresh source → Ok
  - `check_write_privilege` with empty actor → Err containing "provenance"
  - `check_write_privilege` with stale source → Err containing "stale"


## Phase 3: Server Config + Claude Code Integration

Wire the server startup to accept source_dir, produce `.mcp.json` for zero-config Claude Code launch, and surface provenance/confidence in all read responses.

### Affected Files

**Tests:**
- `codegenome-core/src/tests/resolve_integration_tests.rs` — end-to-end: index source dir → resolve → traverse → verify results contain provenance

**Implementation:**
- `codegenome-cli/src/commands/serve.rs` — update to accept `--source` and `--store` flags
- `codegenome-cli/src/commands/init.rs` — **NEW** (~50 lines): `codegenome init` creates `.mcp.json` in a repo
- `codegenome-cli/src/commands/mod.rs` — register `init` subcommand
- `codegenome-cli/src/main.rs` — wire `init` command

### Changes

**`cli/commands/serve.rs`**: Update to pass both `source_dir` and `store_dir`.

```rust
pub fn run(source_dir: &str, store_dir: &str) {
    let rt = tokio::runtime::Runtime::new()
        .expect("Failed to create tokio runtime");
    if let Err(e) = rt.block_on(
        codegenome_mcp::server::run_stdio(
            source_dir.to_string(),
            store_dir.to_string(),
        )
    ) {
        eprintln!("MCP server error: {e}");
        std::process::exit(1);
    }
}
```

**`codegenome_mcp::server::run_stdio`**: Update signature to accept both dirs.

**`cli/commands/init.rs`** (~50 lines): Generate `.mcp.json` for Claude Code.

```rust
pub fn run(source_dir: &str, store_dir: &str) -> Result<(), String>
```

Writes `.mcp.json` to the current directory:

```json
{
  "mcpServers": {
    "codegenome": {
      "command": "codegenome",
      "args": ["serve", "--source", ".", "--store", ".codegenome"]
    }
  }
}
```

Also runs initial index if `.codegenome` doesn't exist.

**Response format upgrade**: All read tool responses include provenance metadata:

```json
{
  "nodes": [...],
  "meta": {
    "index_timestamp": 1712444400,
    "source_fresh": true,
    "toolchain": "tree-sitter-rust 0.23 + heuristic-resolver",
    "confidence_range": [0.5, 1.0]
  }
}
```

This surfaces data trustworthiness without requiring the caller to understand governance internals.

### Unit Tests

- `resolve_integration_tests.rs`:
  - Create a temp dir with 2 Rust files, run `run_pipeline`, load overlay, build FileIndex, resolve → traverse → verify result nodes have non-empty provenance and confidence > 0
