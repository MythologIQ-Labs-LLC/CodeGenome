use crate::embedding::similarity::{cosine_similarity, k_nearest};
use crate::embedding::store::EmbeddingEntry;
use codegenome_identity::graph::node::Timestamp;
use codegenome_identity::identity::address_of;

fn entry(name: &str, vec: Vec<f32>) -> EmbeddingEntry {
    EmbeddingEntry {
        address: address_of(name.as_bytes()),
        vector: vec,
        model: "test".into(),
        timestamp: Timestamp(0),
    }
}

#[test]
fn identical_vectors_have_similarity_one() {
    let a = vec![1.0, 0.0, 0.0];
    let b = vec![1.0, 0.0, 0.0];
    assert!((cosine_similarity(&a, &b) - 1.0).abs() < 0.001);
}

#[test]
fn orthogonal_vectors_have_similarity_zero() {
    let a = vec![1.0, 0.0];
    let b = vec![0.0, 1.0];
    assert!(cosine_similarity(&a, &b).abs() < 0.001);
}

#[test]
fn k_nearest_returns_k_closest() {
    let entries = vec![
        entry("query", vec![1.0, 0.0, 0.0]),
        entry("close", vec![0.9, 0.1, 0.0]),
        entry("medium", vec![0.5, 0.5, 0.0]),
        entry("far", vec![0.0, 0.0, 1.0]),
    ];
    let result = k_nearest(address_of(b"query"), &entries, 2);
    assert_eq!(result.len(), 2);
    assert_eq!(result[0].0, address_of(b"close"));
}

#[test]
fn k_nearest_empty_entries_returns_empty() {
    let entries: Vec<EmbeddingEntry> = vec![];
    let result = k_nearest(address_of(b"missing"), &entries, 5);
    assert!(result.is_empty());
}

#[test]
fn k_nearest_missing_query_returns_empty() {
    let entries = vec![entry("A", vec![1.0, 0.0]), entry("B", vec![0.0, 1.0])];
    let result = k_nearest(address_of(b"missing"), &entries, 2);
    assert!(result.is_empty());
}
