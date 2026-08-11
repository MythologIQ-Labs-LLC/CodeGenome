use serde::{Deserialize, Serialize};

use crate::graph::node::Timestamp;
use crate::identity::{address_of, UorAddress};

/// A structured evidence entry for the Merkle governance ledger.
/// Typed, serializable, with UorAddress-based hashing.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TypedEvidenceEntry {
    pub entry_id: u64,
    pub timestamp: Timestamp,
    pub operation: TypedOperation,
    pub actor: String,
    pub target: String,
    pub content_hash: UorAddress,
    pub previous_hash: UorAddress,
    pub chain_hash: UorAddress,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum TypedOperation {
    Index,
    Query,
    Gate,
    Implement,
    Substantiate,
    Assert,
    DetectChanges,
    Fuse,
}

/// Compute the chain hash for an entry.
pub fn compute_chain_hash(content_hash: UorAddress, previous_hash: UorAddress) -> UorAddress {
    let combined = format!("{content_hash:?}{previous_hash:?}");
    address_of(combined.as_bytes())
}

/// Serialize an entry to JSON.
pub fn to_json(entry: &TypedEvidenceEntry) -> String {
    serde_json::to_string_pretty(entry).unwrap_or_default()
}

/// Deserialize an entry from JSON.
pub fn from_json(json: &str) -> Result<TypedEvidenceEntry, String> {
    serde_json::from_str(json).map_err(|e| e.to_string())
}
