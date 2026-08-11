use std::path::Path;

use crate::graph::edge::{Edge, Relation};
use crate::graph::node::{Node, NodeKind, Provenance, Source, Timestamp};
use crate::identity::{address_of, UorAddress};
use crate::lang::ir::*;

/// Build graph nodes and edges from language-neutral IR.
/// Shared across all language backends.
pub fn build_file_graph(
    file_path: &Path,
    source: &[u8],
    lang_name: &str,
    symbols: &[SymbolDef],
    imports: &[ImportRef],
    calls: &[CallRef],
    impls: &[ImplRef],
) -> (Vec<Node>, Vec<Edge>) {
    let prov = Provenance {
        source: Source::ToolOutput,
        actor: format!("tree-sitter-{lang_name}"),
        timestamp: Timestamp(0),
        justification: None,
    };
    let file_addr = file_address(file_path);
    let content_hash = address_of(source);

    let mut nodes = vec![Node {
        address: file_addr,
        kind: NodeKind::File,
        provenance: prov.clone(),
        confidence: 1.0,
        created_at: Timestamp(0),
        content_hash,
        span: None,
    }];
    let mut edges = Vec::new();

    // Symbols → Nodes + Contains edges
    for sym in symbols {
        let addr = symbol_address(file_path, &sym.source_kind, &sym.name);
        nodes.push(Node {
            address: addr,
            kind: NodeKind::Symbol,
            provenance: prov.clone(),
            confidence: 1.0,
            created_at: Timestamp(0),
            content_hash: address_of(
                &source[sym.span.start_byte as usize..sym.span.end_byte as usize],
            ),
            span: Some(sym.span),
        });
        edges.push(Edge {
            source: file_addr,
            target: addr,
            relation: Relation::Contains,
            confidence: 1.0,
            provenance: prov.clone(),
            evidence: vec![],
        });
    }

    // Imports → edges
    let symbol_table = build_local_table(file_path, symbols);
    for imp in imports {
        if let Some(&target) = symbol_table.get(&imp.imported_name) {
            edges.push(Edge {
                source: file_addr,
                target,
                relation: Relation::Imports,
                confidence: 0.8,
                provenance: prov.clone(),
                evidence: vec![],
            });
        }
    }

    // Calls → edges
    for call in calls {
        let Some(&callee) = symbol_table.get(&call.callee_name) else {
            continue;
        };
        let caller = find_enclosing(file_path, &call.caller_span, symbols);
        let Some(caller_addr) = caller else { continue };
        edges.push(Edge {
            source: caller_addr,
            target: callee,
            relation: Relation::Calls,
            confidence: 0.7,
            provenance: prov.clone(),
            evidence: vec![],
        });
    }

    // Impls → edges
    for imp in impls {
        let Some(trait_name) = &imp.trait_name else {
            continue;
        };
        let Some(&type_addr) = symbol_table.get(&imp.type_name) else {
            continue;
        };
        let Some(&trait_addr) = symbol_table.get(trait_name) else {
            continue;
        };
        edges.push(Edge {
            source: type_addr,
            target: trait_addr,
            relation: Relation::Implements,
            confidence: 0.8,
            provenance: prov.clone(),
            evidence: vec![],
        });
    }

    (nodes, edges)
}

fn file_address(path: &Path) -> UorAddress {
    address_of(format!("file:{}", path.display()).as_bytes())
}

/// Symbol identity: file path + syntax kind + name.
/// File scoping eliminates cross-file name collisions (every `run()` in
/// the codebase used to hash to the same node). Same-file symbols with
/// identical kind and name still collide; qualifying by container path
/// is future work.
pub fn symbol_address(file_path: &Path, kind: &str, name: &str) -> UorAddress {
    address_of(format!("sym:{}:{kind}:{name}", file_path.display()).as_bytes())
}

fn build_local_table(
    file_path: &Path,
    symbols: &[SymbolDef],
) -> std::collections::HashMap<String, UorAddress> {
    symbols
        .iter()
        .map(|s| {
            (
                s.name.clone(),
                symbol_address(file_path, &s.source_kind, &s.name),
            )
        })
        .collect()
}

/// Innermost symbol whose span contains `span` — with methods and nested
/// items now extracted, the narrowest containing span is the true caller
/// (the first match would be the outer impl/mod block).
fn find_enclosing(
    file_path: &Path,
    span: &crate::graph::node::Span,
    symbols: &[SymbolDef],
) -> Option<UorAddress> {
    symbols
        .iter()
        .filter(|s| s.span.start_line <= span.start_line && s.span.end_line >= span.end_line)
        .min_by_key(|s| s.span.end_line - s.span.start_line)
        .map(|s| symbol_address(file_path, &s.source_kind, &s.name))
}
