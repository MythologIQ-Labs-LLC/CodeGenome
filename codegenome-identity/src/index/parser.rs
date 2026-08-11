use std::collections::HashMap;
use std::path::{Path, PathBuf};

use crate::graph::edge::{Edge, Relation};
use crate::graph::node::{Node, NodeKind, Provenance, Source, Span, Timestamp};
use crate::identity::{address_of, UorAddress};
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
    let mut parser = tree_sitter::Parser::new();
    parser
        .set_language(&tree_sitter_rust::LANGUAGE.into())
        .expect("Failed to load Rust grammar");

    files
        .iter()
        .map(|(path, source)| parse_one(&mut parser, path, source))
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
            parsed.push(parse_one(&mut parser, path, source));
        }
    }
    parsed
}

/// Parse a single file with an existing parser instance.
fn parse_one(parser: &mut tree_sitter::Parser, path: &Path, source: &[u8]) -> ParsedFile {
    let file_content = format!("file:{}", path.display());
    let file_address = address_of(file_content.as_bytes());
    let content_hash = address_of(source);

    let provenance = Provenance {
        source: Source::ToolOutput,
        actor: "tree-sitter-rust".into(),
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
        let (sym_nodes, sym_edges) = extract_symbols(file_address, source, &tree, &provenance);
        nodes.extend(sym_nodes);
        edges.extend(sym_edges);
    }

    ParsedFile {
        path: path.to_path_buf(),
        file_address,
        content_hash,
        nodes,
        edges,
    }
}

const SYMBOL_KINDS: &[&str] = &[
    "function_item",
    "struct_item",
    "enum_item",
    "impl_item",
    "use_declaration",
    "mod_item",
    "trait_item",
];

fn extract_symbols(
    file_address: UorAddress,
    source: &[u8],
    tree: &tree_sitter::Tree,
    provenance: &Provenance,
) -> (Vec<Node>, Vec<Edge>) {
    let mut nodes = Vec::new();
    let mut edges = Vec::new();
    let root = tree.root_node();
    let mut cursor = root.walk();

    for child in root.children(&mut cursor) {
        if !SYMBOL_KINDS.contains(&child.kind()) {
            continue;
        }
        let name = symbol_name(&child, source);
        let span = node_span(&child);
        let content = format!("{}:{}", child.kind(), name);
        let address = address_of(content.as_bytes());

        nodes.push(Node {
            address,
            kind: NodeKind::Symbol,
            provenance: provenance.clone(),
            confidence: 1.0,
            created_at: provenance.timestamp,
            content_hash: address_of(&source[child.start_byte()..child.end_byte()]),
            span: Some(span),
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

    (nodes, edges)
}

fn symbol_name(node: &tree_sitter::Node, source: &[u8]) -> String {
    node.child_by_field_name("name")
        .and_then(|n| n.utf8_text(source).ok())
        .unwrap_or(node.kind())
        .to_string()
}

fn node_span(node: &tree_sitter::Node) -> Span {
    Span {
        start_byte: node.start_byte() as u32,
        end_byte: node.end_byte() as u32,
        start_line: node.start_position().row as u32 + 1,
        end_line: node.end_position().row as u32 + 1,
    }
}
