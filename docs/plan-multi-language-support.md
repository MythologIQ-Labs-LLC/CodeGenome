> **SUPERSEDED (2026-08-11).** This blueprint was VETOed by audit (see docs/SHADOW_GENOME.md) and never revised; the work later shipped in modified form. Retained as historical record. File paths reference the pre-rename `codegenome-core/` layout, which no longer exists.

# Plan: Multi-Language Support — Rust + TypeScript + Python

## Open Questions

- Should `arrow_function` in TypeScript be treated as a symbol (named via variable assignment) or ignored? Leaning toward extracting when assigned to a named variable.
- Should Python's `decorated_definition` unwrap to the inner `function_definition`/`class_definition` or be treated as its own symbol kind? Leaning toward unwrap.

## CI Validation

```bash
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
cargo build --release
```

## Phase 1: Language Abstraction + Shared IR Types

Extract the language boundary: shared intermediate types, `LanguageSupport` trait, and Rust backend as first implementor. Split Rust backend into two files to stay under 250 lines.

### File Size Audit

| Proposed File | Content | Estimate | Under 250? |
|---|---|---|---|
| `lang/ir.rs` | IR types (SymbolDef, ImportRef, CallRef, ImplRef, CfEdge, CfKind, DfEdge) | ~80 | Yes |
| `lang/mod.rs` | LanguageSupport trait + module declarations + all_languages() | ~40 | Yes |
| `lang/graph_builder.rs` | IR → Node + Edge shared builder | ~100 | Yes |
| `lang/rust.rs` | Rust symbol/import/call/impl extraction + trait impl | ~185 | Yes |
| `lang/rust_flow.rs` | Rust CFG + DFG extraction (delegated from rust.rs) | ~180 | Yes |
| `lang/detect.rs` | Extension-based language detection + file grouping | ~40 | Yes |

### Affected Files

**Tests:**
- `codegenome-core/src/tests/lang_rust_tests.rs` — Rust backend produces correct SymbolDefs, ImportRefs, CallRefs from known source
- `codegenome-core/src/tests/lang_ir_tests.rs` — shared IR types, graph builder produces correct nodes/edges from IR

**Implementation:**
- `codegenome-core/src/lang/mod.rs` — **NEW** (~40 lines): LanguageSupport trait + module registration
- `codegenome-core/src/lang/ir.rs` — **NEW** (~80 lines): shared IR types
- `codegenome-core/src/lang/graph_builder.rs` — **NEW** (~100 lines): IR → Node + Edge conversion
- `codegenome-core/src/lang/rust.rs` — **NEW** (~185 lines): Rust symbols/imports/calls/impls extraction
- `codegenome-core/src/lang/rust_flow.rs` — **NEW** (~180 lines): Rust CFG + DFG extraction
- `codegenome-core/src/lang/detect.rs` — **NEW** (~40 lines): extension-based language detection
- `codegenome-core/src/lib.rs` — add `pub mod lang;`
- `codegenome-core/src/index/parser.rs` — update to use `LanguageSupport` trait
- `codegenome-core/src/index/mod.rs` — `collect_rs_files` → `collect_source_files`

**Migration — existing `index/` extraction modules become re-exports:**
- `codegenome-core/src/index/extract.rs` — becomes re-export of `lang::rust` extraction types
- `codegenome-core/src/index/flow_cfg.rs` — becomes re-export of `lang::ir::CfEdge` + `lang::rust_flow` functions
- `codegenome-core/src/index/flow_dfg.rs` — becomes re-export of `lang::rust_flow::extract_data_flow`

### Migration: Existing Callers

All 9 callers of `index/extract`, `index/flow_cfg`, `index/flow_dfg` are preserved via re-export wrappers. The `index/` modules delegate to `lang/` internally.

**`index/extract.rs`** — re-export wrapper:
```rust
// Re-exports for backward compatibility.
// Extraction logic now lives in crate::lang::rust.
pub use crate::lang::ir::{CallRef as CallSite, ImportRef as UseTarget, ImplRef as ImplTarget};
pub use crate::lang::ir::node_span;
// extract_use_targets, extract_call_sites, extract_impl_targets
// re-exported via lang::rust public functions
```

2 callers preserved:
- `index/resolver.rs` — `use crate::index::extract::*`
- `overlay/semantic_extract.rs` — re-exports from `crate::index::extract`

**`index/flow_cfg.rs`** — re-export wrapper:
```rust
pub use crate::lang::ir::{CfEdge as CfgEdge, CfKind as CfgKind, DfEdge as DfgEdge};
pub use crate::lang::rust_flow::extract_control_flow;
pub use crate::lang::ir::node_span;
```

3 callers preserved:
- `index/flow.rs` — `use crate::index::flow_cfg`
- `overlay/flow_extract.rs` — re-exports from `crate::index::flow_cfg`
- `tests/index_flow_tests.rs` — `use crate::index::flow_cfg::*`

**`index/flow_dfg.rs`** — re-export wrapper:
```rust
pub use crate::lang::rust_flow::extract_data_flow;
```

3 callers preserved:
- `index/flow.rs` — `use crate::index::flow_dfg`
- `index/flow_dfg.rs` internal — `use crate::index::flow_cfg::DfgEdge` (now from ir.rs)
- `overlay/flow_dfg.rs` — re-exports from `crate::index::flow_dfg`
- `tests/index_flow_tests.rs` — `use crate::index::flow_dfg::*`

### Changes

**`lang/ir.rs`** (~80 lines): Language-neutral intermediate representation.

```rust
pub struct SymbolDef { pub name: String, pub kind: SymbolKind, pub span: Span, pub source_kind: String }
pub enum SymbolKind { Function, Class, Trait, Enum, Module, Other(String) }
pub struct ImportRef { pub imported_name: String, pub span: Span }
pub struct CallRef { pub caller_span: Span, pub callee_name: String, pub span: Span }
pub struct ImplRef { pub type_name: String, pub trait_name: Option<String>, pub span: Span }
pub struct CfEdge { pub source_span: Span, pub target_span: Span, pub kind: CfKind }
pub enum CfKind { Sequential, Branch, BackEdge, Return }
pub struct DfEdge { pub def_span: Span, pub use_span: Span, pub var_name: String }
```

**`lang/mod.rs`** (~40 lines): Trait + registry.

```rust
pub trait LanguageSupport: Send + Sync {
    fn name(&self) -> &str;
    fn extensions(&self) -> &[&str];
    fn language(&self) -> tree_sitter::Language;
    fn extract_symbols(&self, source: &[u8], tree: &tree_sitter::Tree) -> Vec<SymbolDef>;
    fn extract_imports(&self, source: &[u8], tree: &tree_sitter::Tree) -> Vec<ImportRef>;
    fn extract_calls(&self, source: &[u8], tree: &tree_sitter::Tree) -> Vec<CallRef>;
    fn extract_impls(&self, source: &[u8], tree: &tree_sitter::Tree) -> Vec<ImplRef>;
    fn extract_control_flow(&self, source: &[u8], tree: &tree_sitter::Tree) -> Vec<CfEdge>;
    fn extract_data_flow(&self, source: &[u8], tree: &tree_sitter::Tree) -> Vec<DfEdge>;
}

pub fn all_languages() -> Vec<Box<dyn LanguageSupport>> { /* rust only for Phase 1 */ }
```

**`lang/graph_builder.rs`** (~100 lines): Shared IR → graph conversion. Centralizes `address_of` patterns, Contains/Imports/Calls/Implements edge creation.

**`lang/rust.rs`** (~185 lines): Rust backend — symbol/import/call/impl extraction. Moves from `index/extract.rs`. Delegates flow to `rust_flow.rs`.

```rust
impl LanguageSupport for RustLanguage {
    fn extract_control_flow(&self, source, tree) -> Vec<CfEdge> {
        rust_flow::extract_control_flow(source, tree)
    }
    fn extract_data_flow(&self, source, tree) -> Vec<DfEdge> {
        rust_flow::extract_data_flow(source, tree)
    }
}
```

**`lang/rust_flow.rs`** (~180 lines): Rust CFG + DFG extraction. Moves from `index/flow_cfg.rs` + `index/flow_dfg.rs`. Returns `lang::ir::CfEdge` and `lang::ir::DfEdge` instead of the old types.

**`lang/detect.rs`** (~40 lines): Extension routing.

```rust
pub fn detect_language(path: &Path) -> Option<&'static str>
pub fn group_by_language(files: &[(PathBuf, Vec<u8>)]) -> HashMap<&'static str, Vec<(PathBuf, Vec<u8>)>>
pub fn supported_extensions() -> &'static [&'static str]  // ["rs", "ts", "tsx", "py"]
```

### Unit Tests

- `lang_rust_tests.rs`:
  - Parse 3-function Rust file → 3 SymbolDefs with correct names and kinds
  - Parse file with `use` → ImportRef
  - Parse file with function call → CallRef with caller span
  - Parse `impl Trait for Type` → ImplRef
- `lang_ir_tests.rs`:
  - Build graph from synthetic IR (2 SymbolDefs + 1 CallRef) → verify Node count and Calls edge


## Phase 2: TypeScript + Python Backends

### Affected Files

**Tests:**
- `codegenome-core/src/tests/lang_typescript_tests.rs` — TS backend: functions, classes, imports, calls
- `codegenome-core/src/tests/lang_python_tests.rs` — Python backend: defs, classes, imports, calls

**Implementation:**
- `codegenome-core/src/lang/typescript.rs` — **NEW** (~200 lines): TypeScript/TSX backend
- `codegenome-core/src/lang/python.rs` — **NEW** (~180 lines): Python backend
- `codegenome-core/src/lang/mod.rs` — register TS + Python, update `all_languages()`
- `codegenome-core/Cargo.toml` — add `tree-sitter-typescript = "0.23"`, `tree-sitter-python = "0.23"`

### Changes

**`lang/typescript.rs`** (~200 lines): TypeScript/TSX backend.

```rust
pub struct TypeScriptLanguage { tsx: bool }
impl TypeScriptLanguage {
    pub fn ts() -> Self { Self { tsx: false } }
    pub fn tsx() -> Self { Self { tsx: true } }
}
```

Node kind mappings:
- Symbols: `function_declaration`, `class_declaration`, `interface_declaration`, `enum_declaration`, `type_alias_declaration`
- Imports: `import_statement`
- Calls: `call_expression`
- Impls: `class_declaration` with `extends`/`implements`
- Flow: `if_statement`, `switch_statement`, `for_statement`, `for_in_statement`, `while_statement`, `try_statement`
- Data: `variable_declaration`, `lexical_declaration`

Grammar: `LANGUAGE_TYPESCRIPT` for `.ts`, `LANGUAGE_TSX` for `.tsx`.

**`lang/python.rs`** (~180 lines): Python backend.

Node kind mappings:
- Symbols: `function_definition`, `class_definition`, `decorated_definition` (unwrap)
- Imports: `import_statement`, `import_from_statement`
- Calls: `call`
- Impls: `class_definition` with `argument_list` (base classes)
- Flow: `if_statement`, `for_statement`, `while_statement`, `match_statement`, `try_statement`, `with_statement`
- Data: `assignment`, `augmented_assignment`

Grammar: `tree_sitter_python::LANGUAGE`.

### Unit Tests

- `lang_typescript_tests.rs`:
  - Parse TS with `function greet()`, `class Foo`, `import { x } from 'y'` → SymbolDefs, ImportRef
  - Parse TSX component → SymbolDef
  - Parse `interface Bar` → SymbolDef with kind Trait
- `lang_python_tests.rs`:
  - Parse `def greet():`, `class Foo:`, `from os import path` → SymbolDefs, ImportRef
  - Parse `foo()` call → CallRef
  - Parse `@decorator\ndef bar():` → SymbolDef unwraps to function


## Phase 3: Wire Pipeline + Update Orchestrator

### Affected Files

**Tests:**
- `codegenome-core/src/tests/multi_lang_pipeline_tests.rs` — index mixed-language dir, verify all languages produce nodes/edges

**Implementation:**
- `codegenome-core/src/index/parser.rs` — add `parse_files_multi` dispatching per-language group
- `codegenome-core/src/index/resolver.rs` — update `build_symbol_table` to use `LanguageSupport` per file
- `codegenome-core/src/index/flow.rs` — update to delegate to `LanguageSupport::extract_control_flow/data_flow`
- `codegenome-core/src/index/mod.rs` — `collect_source_files` replaces `collect_rs_files`
- `codegenome-core/src/index/orchestrator.rs` — group files by language, fan-out per group

### Changes

**`index/parser.rs`**: New multi-language entry point:

```rust
pub fn parse_files_multi(
    file_groups: &HashMap<&str, Vec<(PathBuf, Vec<u8>)>>,
    languages: &[Box<dyn LanguageSupport>],
) -> Vec<ParsedFile>
```

Existing `parse_files` becomes a thin wrapper calling `parse_files_multi` with Rust-only group.

**`index/resolver.rs`**: `build_symbol_table` uses `LanguageSupport::extract_symbols` instead of hardcoded Rust grammar. Accepts `languages: &[Box<dyn LanguageSupport>]`.

**`index/flow.rs`**: `extract_flow` delegates per-file to the matching `LanguageSupport` backend's `extract_control_flow` + `extract_data_flow`.

**`index/mod.rs`**: `collect_source_files` uses `lang::detect::supported_extensions()` to include `.rs`, `.ts`, `.tsx`, `.py`.

**`index/orchestrator.rs`**: Groups files by language via `lang::detect::group_by_language`, then fans out per-group in rayon scope.

### Unit Tests

- `multi_lang_pipeline_tests.rs`:
  - Create temp dir with `lib.rs`, `app.ts`, `util.py`
  - Run `run_pipeline` → verify IndexResult has nodes from all 3 files
  - Verify fused overlay contains nodes for functions from each language
