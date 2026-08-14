use std::collections::HashMap;
use std::path::{Path, PathBuf};

use crate::graph::edge::{Edge, Relation};
use crate::graph::node::{Provenance, Source, Timestamp};
use crate::identity::{address_of, UorAddress};
use crate::index::parser::ParsedFile;
use crate::index::resolver::ResolvedEdges;
use crate::lang::LanguageSupport;

type ScopedSymbolTable = HashMap<(PathBuf, String), UorAddress>;
type GlobalSymbolIndex = HashMap<String, Vec<UorAddress>>;
type SpanIndex = HashMap<(PathBuf, u32, u32), UorAddress>;

struct SymbolTable {
    scoped: ScopedSymbolTable,
    by_name: GlobalSymbolIndex,
}

impl SymbolTable {
    fn resolve(&self, path: &Path, name: &str) -> Option<UorAddress> {
        if let Some(address) = self.scoped.get(&(path.to_path_buf(), name.to_string())) {
            return Some(*address);
        }

        let candidates = self.by_name.get(name)?;
        match candidates.as_slice() {
            [only] => Some(*only),
            _ => None,
        }
    }
}

struct ResolveCtx<'a> {
    symbols: &'a SymbolTable,
    spans: &'a SpanIndex,
    prov: &'a Provenance,
}

/// Multi-language resolve: uses `LanguageSupport` per file group.
pub fn resolve_multi(
    parsed: &[ParsedFile],
    file_groups: &HashMap<&str, Vec<(PathBuf, Vec<u8>)>>,
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
    let spans = build_span_index(parsed);
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
            resolve_file(
                backend,
                path,
                source,
                &tree,
                file_address(path),
                &ctx,
                &mut edges,
            );
        }
    }

    ResolvedEdges { edges }
}

fn resolve_file(
    backend: &dyn LanguageSupport,
    path: &Path,
    source: &[u8],
    tree: &tree_sitter::Tree,
    file_addr: UorAddress,
    ctx: &ResolveCtx<'_>,
    edges: &mut Vec<Edge>,
) {
    for imp in backend.extract_imports(source, tree) {
        if let Some(addr) = ctx.symbols.resolve(path, &imp.imported_name) {
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
        let Some(callee) = ctx.symbols.resolve(path, &call.callee_name) else {
            continue;
        };
        let Some(caller_addr) = find_enclosing(path, &call.caller_span, ctx.spans) else {
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
        let Some(type_addr) = ctx.symbols.resolve(path, &imp.type_name) else {
            continue;
        };
        let Some(trait_addr) = ctx.symbols.resolve(path, trait_name) else {
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
    file_groups: &HashMap<&str, Vec<(PathBuf, Vec<u8>)>>,
    lang_map: &HashMap<&str, &dyn LanguageSupport>,
) -> SymbolTable {
    let mut scoped = HashMap::new();
    let mut by_name: GlobalSymbolIndex = HashMap::new();

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
                scoped.insert((path.clone(), sym.name.clone()), addr);
                let candidates = by_name.entry(sym.name).or_default();
                if !candidates.contains(&addr) {
                    candidates.push(addr);
                }
            }
        }
    }

    SymbolTable { scoped, by_name }
}

fn build_span_index(parsed: &[ParsedFile]) -> SpanIndex {
    let mut index = HashMap::new();
    for file in parsed {
        for node in &file.nodes {
            if let Some(span) = &node.span {
                index.insert(
                    (file.path.clone(), span.start_line, span.end_line),
                    node.address,
                );
            }
        }
    }
    index
}

fn find_enclosing(
    path: &Path,
    fn_span: &crate::graph::node::Span,
    spans: &SpanIndex,
) -> Option<UorAddress> {
    spans
        .get(&(
            path.to_path_buf(),
            fn_span.start_line,
            fn_span.end_line,
        ))
        .copied()
}

fn file_address(path: &Path) -> UorAddress {
    address_of(format!("file:{}", path.display()).as_bytes())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn edge_exists(edges: &[Edge], source: UorAddress, target: UorAddress) -> bool {
        edges.iter().any(|edge| {
            edge.source == source && edge.target == target && edge.relation == Relation::Calls
        })
    }

    #[test]
    fn duplicate_names_and_overlapping_spans_remain_file_scoped() {
        let main_path = PathBuf::from("fixture/main.rs");
        let decoy_path = PathBuf::from("fixture/decoy.rs");
        let files = vec![
            (
                main_path.clone(),
                b"fn leaf() {}\nfn middle() {\n    leaf();\n}\nfn top() {\n    middle();\n}\n"
                    .to_vec(),
            ),
            (
                decoy_path.clone(),
                b"fn decoy_leaf() {}\nfn middle() {\n    decoy_leaf();\n}\n".to_vec(),
            ),
        ];
        let groups = crate::lang::detect::group_by_language(&files);
        let languages = crate::lang::all_languages();
        let parsed = crate::index::parser::parse_files_multi(&groups, &languages);
        let lang_map: HashMap<&str, &dyn LanguageSupport> =
            languages.iter().map(|l| (l.name(), l.as_ref())).collect();
        let symbols = build_symbol_table(&groups, &lang_map);

        let main_middle = symbols.resolve(&main_path, "middle").unwrap();
        let main_leaf = symbols.resolve(&main_path, "leaf").unwrap();
        let main_top = symbols.resolve(&main_path, "top").unwrap();
        let decoy_middle = symbols.resolve(&decoy_path, "middle").unwrap();
        let decoy_leaf = symbols.resolve(&decoy_path, "decoy_leaf").unwrap();

        assert_ne!(main_middle, decoy_middle);
        assert!(symbols.resolve(Path::new("fixture/missing.rs"), "middle").is_none());

        let resolved = resolve_multi(&parsed, &groups, &languages);

        assert!(edge_exists(&resolved.edges, main_middle, main_leaf));
        assert!(edge_exists(&resolved.edges, main_top, main_middle));
        assert!(edge_exists(&resolved.edges, decoy_middle, decoy_leaf));
        assert!(!edge_exists(&resolved.edges, main_middle, decoy_leaf));
        assert!(!edge_exists(&resolved.edges, decoy_middle, main_leaf));
    }
}
