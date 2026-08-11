use std::collections::{HashMap, HashSet, VecDeque};
use std::path::PathBuf;

use crate::graph::edge::{Edge, Relation};
use crate::graph::node::{Node, NodeKind, Provenance, Source, Timestamp};
use crate::graph::overlay::{Overlay, OverlayKind};
use crate::identity::{address_of, UorAddress};
use crate::measurement::GroundTruthLevel;
use crate::overlay::process_extract::extract_entrypoints;
use crate::overlay::semantic::SemanticOverlay;
use crate::overlay::syntax::SyntaxOverlay;

pub struct ProcessOverlay {
    nodes: Vec<Node>,
    edges: Vec<Edge>,
}

impl Overlay for ProcessOverlay {
    fn kind(&self) -> OverlayKind {
        OverlayKind::Custom("process".into())
    }
    fn nodes(&self) -> &[Node] {
        &self.nodes
    }
    fn edges(&self) -> &[Edge] {
        &self.edges
    }
    fn ground_truth(&self) -> GroundTruthLevel {
        GroundTruthLevel::Constructible
    }
}

impl ProcessOverlay {
    pub fn from_semantic(
        semantic: &SemanticOverlay,
        syntax: &SyntaxOverlay,
        files: &[(PathBuf, Vec<u8>)],
    ) -> Self {
        let prov = Provenance {
            source: Source::Inferred,
            actor: "process-tracer".into(),
            timestamp: Timestamp(0),
            justification: None,
        };
        let name_index = build_name_index(syntax, files);
        let call_graph = build_call_graph(semantic);
        let mut nodes = Vec::new();
        let mut edges = Vec::new();

        for (_, source) in files {
            let Some(tree) = parse_file(source) else {
                continue;
            };
            for ep in extract_entrypoints(source, &tree) {
                let proc_addr = address_of(format!("process:{}", ep.name).as_bytes());
                nodes.push(Node {
                    address: proc_addr,
                    kind: NodeKind::Process,
                    provenance: prov.clone(),
                    confidence: 1.0,
                    created_at: Timestamp(0),
                    content_hash: proc_addr,
                    span: Some(ep.span),
                });
                trace_calls(
                    proc_addr,
                    &ep.name,
                    &name_index,
                    &call_graph,
                    &prov,
                    &mut edges,
                );
            }
        }

        Self { nodes, edges }
    }
}

fn trace_calls(
    proc_addr: UorAddress,
    entry_name: &str,
    names: &HashMap<String, UorAddress>,
    calls: &HashMap<UorAddress, Vec<UorAddress>>,
    prov: &Provenance,
    edges: &mut Vec<Edge>,
) {
    let Some(&start) = names.get(entry_name) else {
        return;
    };
    let mut visited = HashSet::new();
    let mut queue = VecDeque::new();
    queue.push_back((start, 0u32));
    visited.insert(start);

    while let Some((addr, depth)) = queue.pop_front() {
        if depth > 10 {
            continue;
        }
        let conf = 0.9_f64.powi(depth as i32);
        edges.push(Edge {
            source: proc_addr,
            target: addr,
            relation: Relation::PartOfProcess,
            confidence: conf,
            provenance: prov.clone(),
            evidence: vec![],
        });
        for callee in calls.get(&addr).unwrap_or(&vec![]) {
            if visited.insert(*callee) {
                queue.push_back((*callee, depth + 1));
            }
        }
    }
}

fn build_name_index(
    syntax: &SyntaxOverlay,
    files: &[(PathBuf, Vec<u8>)],
) -> HashMap<String, UorAddress> {
    let mut index = HashMap::new();
    for (_, source) in files {
        let Some(tree) = parse_file(source) else {
            continue;
        };
        let root = tree.root_node();
        let mut cursor = root.walk();
        for child in root.children(&mut cursor) {
            if let Some(name) = child
                .child_by_field_name("name")
                .and_then(|n| n.utf8_text(source).ok())
            {
                let content = format!("{}:{}", child.kind(), name);
                let addr = address_of(content.as_bytes());
                if syntax.nodes().iter().any(|n| n.address == addr) {
                    index.insert(name.to_string(), addr);
                }
            }
        }
    }
    index
}

fn build_call_graph(semantic: &SemanticOverlay) -> HashMap<UorAddress, Vec<UorAddress>> {
    let mut graph = HashMap::new();
    for edge in semantic.edges() {
        if edge.relation == Relation::Calls {
            graph
                .entry(edge.source)
                .or_insert_with(Vec::new)
                .push(edge.target);
        }
    }
    graph
}

fn parse_file(source: &[u8]) -> Option<tree_sitter::Tree> {
    let mut parser = tree_sitter::Parser::new();
    parser
        .set_language(&tree_sitter_rust::LANGUAGE.into())
        .ok()?;
    parser.parse(source, None)
}
