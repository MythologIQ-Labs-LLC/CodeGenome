use crate::embedding::store::EmbeddingEntry;
use codegenome_identity::identity::UorAddress;

/// Find k nearest neighbors to a query address by cosine similarity.
/// Returns (address, similarity_score) pairs sorted descending.
pub fn k_nearest(
    query: UorAddress,
    entries: &[EmbeddingEntry],
    k: usize,
) -> Vec<(UorAddress, f32)> {
    let query_vec = match entries.iter().find(|e| e.address == query) {
        Some(e) => &e.vector,
        None => return Vec::new(),
    };

    let mut scored: Vec<(UorAddress, f32)> = entries
        .iter()
        .filter(|e| e.address != query)
        .map(|e| (e.address, cosine_similarity(query_vec, &e.vector)))
        .collect();

    scored.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
    scored.truncate(k);
    scored
}

/// Cosine similarity between two vectors.
/// Returns 0.0 for empty or zero-magnitude vectors.
pub fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
    if a.len() != b.len() || a.is_empty() {
        return 0.0;
    }
    let dot: f32 = a.iter().zip(b).map(|(x, y)| x * y).sum();
    let mag_a: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt();
    let mag_b: f32 = b.iter().map(|x| x * x).sum::<f32>().sqrt();
    if mag_a == 0.0 || mag_b == 0.0 {
        return 0.0;
    }
    dot / (mag_a * mag_b)
}
