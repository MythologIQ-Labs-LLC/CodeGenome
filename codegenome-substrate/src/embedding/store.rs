use std::path::Path;

use serde::{Deserialize, Serialize};

use codegenome_identity::graph::node::Timestamp;
use codegenome_identity::identity::UorAddress;

/// An embedding vector associated with a UOR address.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EmbeddingEntry {
    pub address: UorAddress,
    pub vector: Vec<f32>,
    pub model: String,
    pub timestamp: Timestamp,
}

/// JSON ingestion format for a single entry.
#[derive(Deserialize)]
struct RawEntry {
    address: String,
    vector: Vec<f32>,
    model: String,
}

/// Ingest embeddings from a JSON file.
/// Format: [{ "address": "hex...", "vector": [f32...], "model": "..." }]
pub fn ingest_from_json(path: &Path) -> Result<Vec<EmbeddingEntry>, String> {
    let data = std::fs::read_to_string(path).map_err(|e| e.to_string())?;
    let raw: Vec<RawEntry> = serde_json::from_str(&data).map_err(|e| e.to_string())?;
    let now = now_timestamp();
    Ok(raw
        .into_iter()
        .map(|r| EmbeddingEntry {
            address: codegenome_identity::identity::address_of(r.address.as_bytes()),
            vector: r.vector,
            model: r.model,
            timestamp: now,
        })
        .collect())
}

/// Persist embeddings to the graph store.
pub fn persist_embeddings(
    store: &codegenome_identity::store::ondisk::OnDiskStore,
    entries: &[EmbeddingEntry],
) -> Result<(), String> {
    let data = codegenome_identity::codec::to_vec(entries).map_err(|e| e.to_string())?;
    let path = store.base_dir().join("embeddings.bin");
    std::fs::write(&path, data).map_err(|e| e.to_string())
}

/// Load embeddings from the store.
pub fn load_embeddings(
    store: &codegenome_identity::store::ondisk::OnDiskStore,
) -> Result<Vec<EmbeddingEntry>, String> {
    let path = store.base_dir().join("embeddings.bin");
    if !path.exists() {
        return Ok(Vec::new());
    }
    let data = std::fs::read(&path).map_err(|e| e.to_string())?;
    codegenome_identity::codec::from_slice(&data).map_err(|e| e.to_string())
}

fn now_timestamp() -> Timestamp {
    let ms = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64;
    Timestamp(ms)
}
