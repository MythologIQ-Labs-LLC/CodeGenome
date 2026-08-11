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
        let addr = symbol_address(&sym.source_kind, &sym.name);
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
    let symbol_table = build_local_table(symbols);
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
        let caller = find_enclosing(&call.caller_span, symbols);
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

fn symbol_address(kind: &str, name: &str) -> UorAddress {
    address_of(format!("{kind}:{name}").as_bytes())
}

fn build_local_table(symbols: &[SymbolDef]) -> std::collections::HashMap<String, UorAddress> {
    symbols
        .iter()
        .map(|s| (s.name.clone(), symbol_address(&s.source_kind, &s.name)))
        .collect()
}

fn find_enclosing(span: &crate::graph::node::Span, symbols: &[SymbolDef]) -> Option<UorAddress> {
    symbols
        .iter()
        .find(|s| s.span.start_line <= span.start_line && s.span.end_line >= span.end_line)
        .map(|s| symbol_address(&s.source_kind, &s.name))
}
