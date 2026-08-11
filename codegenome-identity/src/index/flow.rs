use std::collections::HashMap;
use std::path::PathBuf;

use crate::graph::edge::{Edge, Relation};
use crate::graph::node::{Node, NodeKind, Provenance, Source, Span, Timestamp};
use crate::identity::{address_of, UorAddress};
use crate::index::flow_cfg;
use crate::index::flow_dfg;
use crate::lang::LanguageSupport;

/// Result of flow extraction: nodes and edges for CFG + DFG.
pub struct FlowResult {
    pub nodes: Vec<Node>,
    pub edges: Vec<Edge>,
}

/// Extract control flow and data flow from source files.
pub fn extract_flow(files: &[(PathBuf, Vec<u8>)]) -> FlowResult {
    let prov = Provenance {
        source: Source::ToolOutput,
        actor: "flow-extractor".into(),
        timestamp: Timestamp(0),
        justification: None,
    };

    let mut nodes = Vec::new();
    let mut edges = Vec::new();

    for (path, source) in files {
        let Some(tree) = parse_file(source) else {
            continue;
        };
        let cfg = flow_cfg::extract_control_flow(source, &tree);
        let dfg = flow_dfg::extract_data_flow(source, &tree);

        for cf in &cfg {
            let (src, tgt) =
                stmt_node_pair(path, &cf.source_span, &cf.target_span, &prov, &mut nodes);
            edges.push(Edge {
                source: src,
                target: tgt,
                relation: Relation::ControlFlow,
                confidence: 1.0,
                provenance: prov.clone(),
                evidence: vec![],
            });
        }
        for df in &dfg {
            let (src, tgt) = stmt_node_pair(path, &df.def_span, &df.use_span, &prov, &mut nodes);
            edges.push(Edge {
                source: src,
                target: tgt,
                relation: Relation::DataFlow,
                confidence: 1.0,
                provenance: prov.clone(),
                evidence: vec![],
            });
        }
    }

    FlowResult { nodes, edges }
}

fn stmt_node_pair(
    path: &std::path::Path,
    src_span: &Span,
    tgt_span: &Span,
    prov: &Provenance,
    nodes: &mut Vec<Node>,
) -> (UorAddress, UorAddress) {
    let src_addr = stmt_address(path, src_span);
    let tgt_addr = stmt_address(path, tgt_span);
    ensure_node(src_addr, src_span, prov, nodes);
    ensure_node(tgt_addr, tgt_span, prov, nodes);
    (src_addr, tgt_addr)
}

fn ensure_node(addr: UorAddress, span: &Span, prov: &Provenance, nodes: &mut Vec<Node>) {
    if !nodes.iter().any(|n| n.address == addr) {
        nodes.push(Node {
            address: addr,
            kind: NodeKind::Symbol,
            provenance: prov.clone(),
            confidence: 1.0,
            created_at: Timestamp(0),
            content_hash: addr,
            span: Some(*span),
        });
    }
}

fn stmt_address(path: &std::path::Path, span: &Span) -> UorAddress {
    let content = format!(
        "stmt:{}:{}:{}",
        path.display(),
        span.start_byte,
        span.end_byte,
    );
    address_of(content.as_bytes())
}

fn parse_file(source: &[u8]) -> Option<tree_sitter::Tree> {
    let mut parser = tree_sitter::Parser::new();
    parser
        .set_language(&tree_sitter_rust::LANGUAGE.into())
        .ok()?;
    parser.parse(source, None)
}

/// Extract flow using language-specific backends per file group.
pub fn extract_flow_multi(
    file_groups: &HashMap<&str, Vec<(PathBuf, Vec<u8>)>>,
    languages: &[Box<dyn LanguageSupport>],
) -> FlowResult {
    let lang_map: HashMap<&str, &dyn LanguageSupport> =
        languages.iter().map(|l| (l.name(), l.as_ref())).collect();

    let prov = Provenance {
        source: Source::ToolOutput,
        actor: "flow-extractor".into(),
        timestamp: Timestamp(0),
        justification: None,
    };

    let mut nodes = Vec::new();
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
            let Some(tree) = parser.parse(source, None) else {
                continue;
            };
            let cfg = backend.extract_control_flow(source, &tree);
            let dfg = backend.extract_data_flow(source, &tree);

            for cf in &cfg {
                let (src, tgt) =
                    stmt_node_pair(path, &cf.source_span, &cf.target_span, &prov, &mut nodes);
                edges.push(Edge {
                    source: src,
                    target: tgt,
                    relation: Relation::ControlFlow,
                    confidence: 1.0,
                    provenance: prov.clone(),
                    evidence: vec![],
                });
            }
            for df in &dfg {
                let (src, tgt) =
                    stmt_node_pair(path, &df.def_span, &df.use_span, &prov, &mut nodes);
                edges.push(Edge {
                    source: src,
                    target: tgt,
                    relation: Relation::DataFlow,
                    confidence: 1.0,
                    provenance: prov.clone(),
                    evidence: vec![],
                });
            }
        }
    }

    FlowResult { nodes, edges }
}
