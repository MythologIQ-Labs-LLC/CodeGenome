use serde::{Deserialize, Serialize};

use crate::identity::UorAddress;

/// A node is a value. It does not know what graph it belongs
/// to. It does not know what overlays reference it.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Node {
    pub address: UorAddress,
    pub kind: NodeKind,
    pub provenance: Provenance,
    pub confidence: f64,
    pub created_at: Timestamp,
    pub content_hash: UorAddress,
    pub span: Option<Span>,
}

/// Byte range in source. Bridges tree-sitter positions to graph nodes.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Span {
    pub start_byte: u32,
    pub end_byte: u32,
    pub start_line: u32,
    pub end_line: u32,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum NodeKind {
    File,
    Symbol,
    Scope,
    Process,
    Belief,
}

/// Provenance is intrinsic, not bolted on.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Provenance {
    pub source: Source,
    pub actor: String,
    pub timestamp: Timestamp,
    pub justification: Option<UorAddress>,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Source {
    UserStated,
    Inferred,
    ToolOutput,
    Consolidated,
}

/// Milliseconds since Unix epoch. Value type.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct Timestamp(pub u64);

impl Provenance {
    pub fn tool(actor: impl Into<String>, timestamp: Timestamp) -> Self {
        Self {
            source: Source::ToolOutput,
            actor: actor.into(),
            timestamp,
            justification: None,
        }
    }
}
