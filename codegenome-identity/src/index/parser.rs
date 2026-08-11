use std::collections::HashMap;
use std::path::{Path, PathBuf};

use crate::graph::edge::{Edge, Relation};
use crate::graph::node::{Node, NodeKind, Provenance, Source, Timestamp};
use crate::identity::{address_of, UorAddress};
use crate::lang::graph_builder::symbol_address;
use crate::lang::LanguageSupport;

/// A parsed source file: file-level node plus extracted symbols.
/// Pure value — no handles, no state.
#[derive(Clone, Debug)]
pub struct ParsedFile {
    pub path: PathBuf,
    pub file_address: UorAddress,
    pub content_hash: UorAddress,
    pub nodes: Vec<Node>,
    pub edges: Vec<Edge>,
}

/// Parse multiple Rust source files (backward-compatible wrapper).
pub fn parse_files(files: &[(PathBuf, Vec<u8>)]) -> Vec<ParsedFile> {
    let backend = crate::lang::rust::RustLanguage;
    let mut parser = tree_sitter::Parser::new();
    parser
        .set_language(&backend.language())
        .expect("Failed to load Rust grammar");

    files
        .iter()
        .map(|(path, source)| parse_one(&mut parser, &backend, path, source))
        .collect()
}

/// Parse files grouped by language. Each group uses the matching
/// language backend's grammar.
pub fn parse_files_multi(
    file_groups: &HashMap<&str, Vec<(PathBuf, Vec<u8>)>>,
    languages: &[Box<dyn LanguageSupport>],
) -> Vec<ParsedFile> {
    let lang_map: HashMap<&str, &dyn LanguageSupport> =
        languages.iter().map(|l| (l.name(), l.as_ref())).collect();

    let mut parsed = Vec::new();
    for (lang_name, files) in file_groups {
        let Some(&backend) = lang_map.get(lang_name) else {
            continue;
        };
        let mut parser = tree_sitter::Parser::new();
        if parser.set_language(&backend.language()).is_err() {
            continue;
        }
        for (path, source) in files {
            parsed.push(parse_one(&mut parser, backend, path, source));
        }
    }
    parsed
}

/// Parse a single file with an existing parser instance. Symbol
/// extraction is delegated to the language backend, so every language
/// (not just Rust) contributes symbol nodes, and addresses come from
/// the one shared `symbol_address` function.
fn parse_one(
    parser: &mut tree_sitter::Parser,
    backend: &dyn LanguageSupport,
    path: &Path,
    source: &[u8],
) -> ParsedFile {
    let file_content = format!("file:{}", path.display());
    let file_address = address_of(file_content.as_bytes());
    let content_hash = address_of(source);

    let provenance = Provenance {
        source: Source::ToolOutput,
        actor: format!("tree-sitter-{}", backend.name()),
        timestamp: Timestamp(0),
        justification: None,
    };

    let mut nodes = vec![Node {
        address: file_address,
        kind: NodeKind::File,
        provenance: provenance.clone(),
        confidence: 1.0,
        created_at: Timestamp(0),
        content_hash,
        span: None,
    }];
    let mut edges = Vec::new();

    if let Some(tree) = parser.parse(source, None) {
        for sym in backend.extract_symbols(source, &tree) {
            let address = symbol_address(path, &sym.source_kind, &sym.name);
            let start = sym.span.start_byte as usize;
            let end = (sym.span.end_byte as usize).min(source.len());
            nodes.push(Node {
                address,
                kind: NodeKind::Symbol,
                provenance: provenance.clone(),
                confidence: 1.0,
                created_at: provenance.timestamp,
                content_hash: address_of(&source[start..end]),
                span: Some(sym.span),
            });
            edges.push(Edge {
                source: file_address,
                target: address,
                relation: Relation::Contains,
                confidence: 1.0,
                provenance: provenance.clone(),
                evidence: vec![],
            });
        }
    }

    ParsedFile {
        path: path.to_path_buf(),
        file_address,
        content_hash,
        nodes,
        edges,
    }
}
