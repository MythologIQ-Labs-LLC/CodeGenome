use std::collections::HashMap;

use crate::graph::edge::{Edge, Relation};
use crate::graph::node::{Provenance, Source, Timestamp};
use crate::graph::overlay::Overlay;
use crate::identity::{address_of, UorAddress};
use crate::index::extract::*;
use crate::index::parser::ParsedFile;
use crate::overlay::syntax::SyntaxOverlay;

/// Resolved semantic edges from symbol resolution.
pub struct ResolvedEdges {
    pub edges: Vec<Edge>,
}

/// Resolve semantic edges from parsed files and source bytes.
pub fn resolve(parsed: &[ParsedFile], files: &[(std::path::PathBuf, Vec<u8>)]) -> ResolvedEdges {
    let provenance = Provenance {
        source: Source::Inferred,
        actor: "heuristic-resolver".into(),
        timestamp: Timestamp(0),
        justification: None,
    };

    let symbol_table = build_symbol_table(files);
    let syntax = SyntaxOverlay::from_parsed(parsed);
    let span_index = build_span_index(&syntax);
    let mut edges = Vec::new();

    for (path, source) in files {
        let Some(tree) = parse_file(source) else {
            continue;
        };
        let file_addr = file_address(path);
        resolve_uses(
            &tree,
            source,
            file_addr,
            &symbol_table,
            &provenance,
            &mut edges,
        );
        resolve_calls(
            &tree,
            source,
            &symbol_table,
            &span_index,
            &provenance,
            &mut edges,
        );
        resolve_impls(&tree, source, &symbol_table, &provenance, &mut edges);
    }

    ResolvedEdges { edges }
}

type SymbolTable = HashMap<String, UorAddress>;
type SpanIndex = HashMap<(u32, u32), UorAddress>;

fn build_symbol_table(files: &[(std::path::PathBuf, Vec<u8>)]) -> SymbolTable {
    use crate::lang::LanguageSupport;
    let backend = crate::lang::rust::RustLanguage;
    let mut table = HashMap::new();
    for (path, source) in files {
        let Some(tree) = parse_file(source) else {
            continue;
        };
        for sym in backend.extract_symbols(source, &tree) {
            let addr =
                crate::lang::graph_builder::symbol_address(path, &sym.source_kind, &sym.name);
            table.insert(sym.name, addr);
        }
    }
    table
}

fn build_span_index(syntax: &SyntaxOverlay) -> SpanIndex {
    let mut index = HashMap::new();
    for node in syntax.nodes() {
        if let Some(span) = &node.span {
            index.insert((span.start_line, span.end_line), node.address);
        }
    }
    index
}

fn resolve_uses(
    tree: &tree_sitter::Tree,
    source: &[u8],
    file_addr: UorAddress,
    symbols: &SymbolTable,
    prov: &Provenance,
    edges: &mut Vec<Edge>,
) {
    for target in extract_use_targets(source, tree) {
        if let Some(&addr) = symbols.get(&target.imported_name) {
            edges.push(Edge {
                source: file_addr,
                target: addr,
                relation: Relation::Imports,
                confidence: 0.8,
                provenance: prov.clone(),
                evidence: vec![],
            });
        }
    }
}

fn resolve_calls(
    tree: &tree_sitter::Tree,
    source: &[u8],
    symbols: &SymbolTable,
    spans: &SpanIndex,
    prov: &Provenance,
    edges: &mut Vec<Edge>,
) {
    for site in extract_call_sites(source, tree) {
        let Some(&callee) = symbols.get(&site.callee_name) else {
            continue;
        };
        let Some(caller_addr) = find_enclosing_symbol(&site.caller_span, spans) else {
            continue;
        };
        edges.push(Edge {
            source: caller_addr,
            target: callee,
            relation: Relation::Calls,
            confidence: 0.7,
            provenance: prov.clone(),
            evidence: vec![],
        });
    }
}

fn resolve_impls(
    tree: &tree_sitter::Tree,
    source: &[u8],
    symbols: &SymbolTable,
    prov: &Provenance,
    edges: &mut Vec<Edge>,
) {
    for target in extract_impl_targets(source, tree) {
        let trait_name = match &target.trait_name {
            Some(n) => n,
            None => continue,
        };
        let Some(&type_addr) = symbols.get(&target.type_name) else {
            continue;
        };
        let Some(&trait_addr) = symbols.get(trait_name) else {
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
}

fn find_enclosing_symbol(
    fn_span: &crate::graph::node::Span,
    spans: &SpanIndex,
) -> Option<UorAddress> {
    spans.get(&(fn_span.start_line, fn_span.end_line)).copied()
}

const SYMBOL_KINDS: &[&str] = &[
    "function_item",
    "struct_item",
    "enum_item",
    "impl_item",
    "mod_item",
    "trait_item",
];

fn symbol_node_name(node: &tree_sitter::Node, source: &[u8]) -> Option<String> {
    if !SYMBOL_KINDS.contains(&node.kind()) {
        return None;
    }
    node.child_by_field_name("name")
        .and_then(|n| n.utf8_text(source).ok())
        .map(String::from)
}

fn parse_file(source: &[u8]) -> Option<tree_sitter::Tree> {
    let mut parser = tree_sitter::Parser::new();
    parser
        .set_language(&tree_sitter_rust::LANGUAGE.into())
        .ok()?;
    parser.parse(source, None)
}

fn file_address(path: &std::path::Path) -> UorAddress {
    let content = format!("file:{}", path.display());
    address_of(content.as_bytes())
}

// Multi-language resolve is in resolver_multi.rs
pub use super::resolver_multi::resolve_multi;
