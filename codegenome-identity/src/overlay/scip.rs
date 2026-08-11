use std::path::Path;

use crate::graph::edge::{Edge, Relation};
use crate::graph::node::{Node, Provenance, Source, Timestamp};
use crate::graph::overlay::{Overlay, OverlayKind};
use crate::identity::{address_of, UorAddress};
use crate::measurement::GroundTruthLevel;

pub struct ScipOverlay {
    nodes: Vec<Node>,
    edges: Vec<Edge>,
}

impl Overlay for ScipOverlay {
    fn kind(&self) -> OverlayKind {
        OverlayKind::Semantic
    }
    fn nodes(&self) -> &[Node] {
        &self.nodes
    }
    fn edges(&self) -> &[Edge] {
        &self.edges
    }
    fn ground_truth(&self) -> GroundTruthLevel {
        GroundTruthLevel::Available
    }
}

/// Minimal SCIP index structures (subset of the full protobuf schema).
/// SCIP encodes: Index → Documents → Occurrences + Symbols.
mod types {
    /// Role of an occurrence in the source.
    #[derive(Clone, Debug, PartialEq)]
    pub enum SymbolRole {
        Definition,
        Reference,
    }

    /// One symbol occurrence in a document.
    #[derive(Clone, Debug)]
    pub struct Occurrence {
        pub symbol: String,
        pub role: SymbolRole,
        pub line: u32,
    }

    /// A parsed SCIP document (one source file).
    #[derive(Clone, Debug)]
    pub struct ScipDocument {
        pub relative_path: String,
        pub occurrences: Vec<Occurrence>,
    }
}

impl ScipOverlay {
    /// Build overlay from a SCIP index file (.scip protobuf).
    /// Currently uses a simplified JSON representation for portability.
    /// Full protobuf decoding via prost can replace this parser.
    pub fn from_scip_file(path: &Path) -> Result<Self, String> {
        let data = std::fs::read(path).map_err(|e| e.to_string())?;
        let docs = parse_scip_simple(&data)?;
        let prov = Provenance {
            source: Source::ToolOutput,
            actor: "scip-indexer".into(),
            timestamp: Timestamp(0),
            justification: None,
        };

        let mut edges = Vec::new();
        for doc in &docs {
            build_doc_edges(doc, &prov, &mut edges);
        }

        Ok(Self {
            nodes: Vec::new(),
            edges,
        })
    }

    /// Build from pre-parsed documents (for testing).
    pub fn from_documents(docs: &[types::ScipDocument]) -> Self {
        let prov = Provenance {
            source: Source::ToolOutput,
            actor: "scip-indexer".into(),
            timestamp: Timestamp(0),
            justification: None,
        };
        let mut edges = Vec::new();
        for doc in docs {
            build_doc_edges(doc, &prov, &mut edges);
        }
        Self {
            nodes: Vec::new(),
            edges,
        }
    }
}

fn build_doc_edges(doc: &types::ScipDocument, prov: &Provenance, edges: &mut Vec<Edge>) {
    let mut defs: std::collections::HashMap<String, UorAddress> = std::collections::HashMap::new();

    // First pass: collect definitions
    for occ in &doc.occurrences {
        if occ.role == types::SymbolRole::Definition {
            let addr = address_of(format!("scip:{}", occ.symbol).as_bytes());
            defs.insert(occ.symbol.clone(), addr);
        }
    }

    // Second pass: references → definitions
    for occ in &doc.occurrences {
        if occ.role == types::SymbolRole::Reference {
            if let Some(&def_addr) = defs.get(&occ.symbol) {
                let ref_addr =
                    address_of(format!("scip-ref:{}:{}", doc.relative_path, occ.line).as_bytes());
                edges.push(Edge {
                    source: ref_addr,
                    target: def_addr,
                    relation: Relation::References,
                    confidence: 1.0,
                    provenance: prov.clone(),
                    evidence: vec![],
                });
            }
        }
    }
}

/// Parse SCIP data. For now, try JSON format first (test-friendly).
/// Full protobuf via prost::Message::decode is the production path.
fn parse_scip_simple(data: &[u8]) -> Result<Vec<types::ScipDocument>, String> {
    // Try JSON parse first (for tests and simple interchange)
    if let Ok(text) = std::str::from_utf8(data) {
        if let Ok(docs) = serde_json::from_str::<Vec<JsonScipDoc>>(text) {
            return Ok(docs.into_iter().map(|d| d.into()).collect());
        }
    }
    // Protobuf decode would go here with prost
    Err("SCIP parse failed: unsupported format".into())
}

#[derive(serde::Deserialize)]
struct JsonScipDoc {
    relative_path: String,
    occurrences: Vec<JsonOccurrence>,
}

#[derive(serde::Deserialize)]
struct JsonOccurrence {
    symbol: String,
    role: String,
    line: u32,
}

impl From<JsonScipDoc> for types::ScipDocument {
    fn from(d: JsonScipDoc) -> Self {
        Self {
            relative_path: d.relative_path,
            occurrences: d
                .occurrences
                .into_iter()
                .map(|o| types::Occurrence {
                    symbol: o.symbol,
                    role: if o.role == "definition" {
                        types::SymbolRole::Definition
                    } else {
                        types::SymbolRole::Reference
                    },
                    line: o.line,
                })
                .collect(),
        }
    }
}

pub use types::{Occurrence, ScipDocument, SymbolRole};
