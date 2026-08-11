//! File-style projection of the graph: a markdown "repo map."
//!
//! Agents built around file-semantics memory (memory tools, CLAUDE.md
//! conventions, Aider-style repo maps) consume a compact text
//! projection better than a graph API. This renders each indexed file
//! with its symbols' signature lines plus fused-graph fan-in/fan-out,
//! so the map carries evidence the flat file listing cannot: which
//! symbols are hubs, and how strongly the graph believes it.

use std::collections::HashMap;
use std::path::Path;

use crate::graph::edge::{Edge, Relation};
use crate::graph::node::{Node, NodeKind};
use crate::identity::{address_of, UorAddress};

/// Render a markdown repo map from source files plus the fused graph.
pub fn render_map(source_files: &[(std::path::PathBuf, Vec<u8>)], edges: &[Edge]) -> String {
    render_map_nodes(source_files, &[], edges)
}

/// Render with explicit nodes (spans come from Symbol nodes; when a
/// file has no symbol nodes it is listed without detail).
pub fn render_map_nodes(
    source_files: &[(std::path::PathBuf, Vec<u8>)],
    nodes: &[Node],
    edges: &[Edge],
) -> String {
    let node_by_addr: HashMap<UorAddress, &Node> = nodes.iter().map(|n| (n.address, n)).collect();

    // Fan-in / fan-out over call edges in the fused graph.
    let mut fan_out: HashMap<UorAddress, usize> = HashMap::new();
    let mut fan_in: HashMap<UorAddress, usize> = HashMap::new();
    for e in edges {
        if e.relation == Relation::Calls {
            *fan_out.entry(e.source).or_default() += 1;
            *fan_in.entry(e.target).or_default() += 1;
        }
    }

    // file address -> contained symbol addresses
    let mut contains: HashMap<UorAddress, Vec<UorAddress>> = HashMap::new();
    for e in edges {
        if e.relation == Relation::Contains {
            contains.entry(e.source).or_default().push(e.target);
        }
    }

    let mut out = String::from("# Repo Map (CODEGENOME projection)\n\n");
    out.push_str(
        "Signature lines from the syntax overlay; `in`/`out` are call-edge \
         fan-in/fan-out from the fused graph.\n\n",
    );

    for (path, source) in source_files {
        let file_addr = file_address(path);
        out.push_str(&format!("## {}\n\n", path.display()));
        let Some(symbols) = contains.get(&file_addr) else {
            out.push_str("*(no symbols indexed)*\n\n");
            continue;
        };
        let mut rows: Vec<(u32, String)> = Vec::new();
        for sym_addr in symbols {
            let Some(node) = node_by_addr.get(sym_addr) else {
                continue;
            };
            if node.kind != NodeKind::Symbol {
                continue;
            }
            let Some(span) = &node.span else { continue };
            let sig = signature_line(source, span.start_byte as usize);
            let fi = fan_in.get(sym_addr).copied().unwrap_or(0);
            let fo = fan_out.get(sym_addr).copied().unwrap_or(0);
            let mut row = format!("- `{}` (L{}", sig, span.start_line);
            if span.end_line > span.start_line {
                row.push_str(&format!("–{}", span.end_line));
            }
            row.push(')');
            if fi > 0 || fo > 0 {
                row.push_str(&format!(" — in:{fi} out:{fo}"));
            }
            rows.push((span.start_line, row));
        }
        rows.sort();
        rows.dedup();
        for (_, row) in rows {
            out.push_str(&row);
            out.push('\n');
        }
        out.push('\n');
    }
    out
}

/// First line of the symbol's source span, trimmed — its signature.
fn signature_line(source: &[u8], start_byte: usize) -> String {
    let rest = &source[start_byte.min(source.len())..];
    let end = rest.iter().position(|&b| b == b'\n').unwrap_or(rest.len());
    String::from_utf8_lossy(&rest[..end])
        .trim()
        .trim_end_matches('{')
        .trim()
        .to_string()
}

fn file_address(path: &Path) -> UorAddress {
    address_of(format!("file:{}", path.display()).as_bytes())
}
