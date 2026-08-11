use std::collections::HashMap;
use std::path::{Path, PathBuf};

use codegenome_identity::graph::edge::{Edge, Relation};
use codegenome_identity::graph::node::{Provenance, Timestamp};
use codegenome_identity::identity::{address_of, UorAddress};
use codegenome_identity::lang::LanguageSupport;

/// Build an export table: map symbol name → UorAddress.
/// Extracts top-level symbols from source files using the
/// language backend.
pub fn build_export_table(
    lang: &dyn LanguageSupport,
    files: &[(PathBuf, Vec<u8>)],
) -> HashMap<String, UorAddress> {
    let mut table = HashMap::new();
    let mut parser = tree_sitter::Parser::new();
    if parser.set_language(&lang.language()).is_err() {
        return table;
    }
    for (path, source) in files {
        let Some(tree) = parser.parse(source.as_slice(), None) else {
            continue;
        };
        for sym in lang.extract_symbols(source, &tree) {
            let addr = codegenome_identity::lang::graph_builder::symbol_address(
                path,
                &sym.source_kind,
                &sym.name,
            );
            table.insert(sym.name, addr);
        }
    }
    table
}

/// Resolve imports in `importer_files` against `exporter_exports`.
/// Returns Imports edges with confidence 0.7.
///
/// Conservative: an edge is created only when the importer has
/// an import naming a symbol that exists in the exporter's table.
pub fn resolve_cross_repo(
    importer_lang: &dyn LanguageSupport,
    importer_files: &[(PathBuf, Vec<u8>)],
    exporter_exports: &HashMap<String, UorAddress>,
) -> Vec<Edge> {
    let prov = Provenance {
        source: codegenome_identity::graph::node::Source::Inferred,
        actor: "cross-repo-resolver".into(),
        timestamp: Timestamp(0),
        justification: None,
    };

    let mut edges = Vec::new();
    let mut parser = tree_sitter::Parser::new();
    if parser.set_language(&importer_lang.language()).is_err() {
        return edges;
    }
    for (path, source) in importer_files {
        let Some(tree) = parser.parse(source.as_slice(), None) else {
            continue;
        };
        let file_addr = file_address(path);
        for imp in importer_lang.extract_imports(source, &tree) {
            let Some(&target) = exporter_exports.get(&imp.imported_name) else {
                continue;
            };
            edges.push(Edge {
                source: file_addr,
                target,
                relation: Relation::Imports,
                confidence: 0.7,
                provenance: prov.clone(),
                evidence: vec![file_addr, target],
            });
        }
    }
    edges
}

fn file_address(path: &Path) -> UorAddress {
    address_of(format!("file:{}", path.display()).as_bytes())
}
