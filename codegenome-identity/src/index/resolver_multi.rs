use std::collections::HashMap;

use crate::graph::edge::{Edge, Relation};
use crate::graph::node::{Provenance, Source, Timestamp};
use crate::graph::overlay::Overlay;
use crate::identity::{address_of, UorAddress};
use crate::index::parser::ParsedFile;
use crate::index::resolver::ResolvedEdges;
use crate::lang::LanguageSupport;
use crate::overlay::syntax::SyntaxOverlay;

type SymbolTable = HashMap<String, UorAddress>;
type SpanIndex = HashMap<(u32, u32), UorAddress>;

struct ResolveCtx<'a> {
    symbols: &'a SymbolTable,
    spans: &'a SpanIndex,
    prov: &'a Provenance,
}

/// Multi-language resolve: uses `LanguageSupport` per file group.
pub fn resolve_multi(
    parsed: &[ParsedFile],
    file_groups: &HashMap<&str, Vec<(std::path::PathBuf, Vec<u8>)>>,
    languages: &[Box<dyn LanguageSupport>],
) -> ResolvedEdges {
    let lang_map: HashMap<&str, &dyn LanguageSupport> =
        languages.iter().map(|l| (l.name(), l.as_ref())).collect();

    let prov = Provenance {
        source: Source::Inferred,
        actor: "heuristic-resolver".into(),
        timestamp: Timestamp(0),
        justification: None,
    };

    let symbols = build_symbol_table(file_groups, &lang_map);
    let syntax = SyntaxOverlay::from_parsed(parsed);
    let spans = build_span_index(&syntax);
    let ctx = ResolveCtx {
        symbols: &symbols,
        spans: &spans,
        prov: &prov,
    };
    let mut edges = Vec::new();

    for (lang_name, files) in file_groups {
        let Some(&backend) = lang_map.get(lang_name) else {
            continue;
        };
        let mut parser = tree_sitter::Parser::new();
        if parser.set_language(&backend.language()).is_err() {
            continue;
        }
        for (path, source) in files {
            let Some(tree) = parser.parse(source.as_slice(), None) else {
                continue;
            };
            resolve_file(backend, source, &tree, file_address(path), &ctx, &mut edges);
        }
    }

    ResolvedEdges { edges }
}

fn resolve_file(
    backend: &dyn LanguageSupport,
    source: &[u8],
    tree: &tree_sitter::Tree,
    file_addr: UorAddress,
    ctx: &ResolveCtx<'_>,
    edges: &mut Vec<Edge>,
) {
    for imp in backend.extract_imports(source, tree) {
        if let Some(&addr) = ctx.symbols.get(&imp.imported_name) {
            edges.push(Edge {
                source: file_addr,
                target: addr,
                relation: Relation::Imports,
                confidence: 0.8,
                provenance: ctx.prov.clone(),
                evidence: vec![],
            });
        }
    }
    for call in backend.extract_calls(source, tree) {
        let Some(&callee) = ctx.symbols.get(&call.callee_name) else {
            continue;
        };
        let Some(caller_addr) = find_enclosing(&call.caller_span, ctx.spans) else {
            continue;
        };
        edges.push(Edge {
            source: caller_addr,
            target: callee,
            relation: Relation::Calls,
            confidence: 0.7,
            provenance: ctx.prov.clone(),
            evidence: vec![],
        });
    }
    for imp in backend.extract_impls(source, tree) {
        let Some(ref trait_name) = imp.trait_name else {
            continue;
        };
        let Some(&type_addr) = ctx.symbols.get(&imp.type_name) else {
            continue;
        };
        let Some(&trait_addr) = ctx.symbols.get(trait_name) else {
            continue;
        };
        edges.push(Edge {
            source: type_addr,
            target: trait_addr,
            relation: Relation::Implements,
            confidence: 0.8,
            provenance: ctx.prov.clone(),
            evidence: vec![],
        });
    }
}

fn build_symbol_table(
    file_groups: &HashMap<&str, Vec<(std::path::PathBuf, Vec<u8>)>>,
    lang_map: &HashMap<&str, &dyn LanguageSupport>,
) -> SymbolTable {
    let mut table = HashMap::new();
    for (lang_name, files) in file_groups {
        let Some(&backend) = lang_map.get(lang_name) else {
            continue;
        };
        let mut parser = tree_sitter::Parser::new();
        if parser.set_language(&backend.language()).is_err() {
            continue;
        }
        for (path, source) in files {
            let Some(tree) = parser.parse(source.as_slice(), None) else {
                continue;
            };
            for sym in backend.extract_symbols(source, &tree) {
                let addr =
                    crate::lang::graph_builder::symbol_address(path, &sym.source_kind, &sym.name);
                table.insert(sym.name, addr);
            }
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

fn find_enclosing(fn_span: &crate::graph::node::Span, spans: &SpanIndex) -> Option<UorAddress> {
    spans.get(&(fn_span.start_line, fn_span.end_line)).copied()
}

fn file_address(path: &std::path::Path) -> UorAddress {
    address_of(format!("file:{}", path.display()).as_bytes())
}
