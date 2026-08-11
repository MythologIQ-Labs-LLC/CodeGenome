pub mod cache;
pub mod dynamic;
pub mod extract;
pub mod flow;
pub mod flow_cfg;
pub mod flow_dfg;
pub mod merger;
pub mod orchestrator;
pub mod parser;
pub mod resolver;
pub mod resolver_multi;

use std::path::{Path, PathBuf};

use crate::graph::overlay::{Overlay, OverlayKind};
use crate::overlay::flow::FlowOverlay;
use crate::overlay::semantic::SemanticOverlay;
use crate::overlay::syntax::SyntaxOverlay;
use crate::store::backend::StoreBackend;
use crate::store::meta::{self, IndexMeta};
use crate::store::ondisk::OnDiskStore;

pub struct IndexResult {
    pub file_count: usize,
    pub node_count: usize,
    pub edge_count: usize,
    pub elapsed_ms: u64,
    pub is_fresh: bool,
}

/// Shared index pipeline: check freshness, build overlays, fuse, store.
/// Returns immediately if index is fresh.
pub fn run_pipeline(source_dir: &Path, store_dir: &Path) -> Result<IndexResult, String> {
    let freshness = meta::check_freshness(store_dir, source_dir);
    if freshness.is_fresh {
        if let Ok(Some(m)) = meta::load(store_dir) {
            return Ok(IndexResult {
                file_count: m.file_count,
                node_count: m.node_count,
                edge_count: m.edge_count,
                elapsed_ms: 0,
                is_fresh: true,
            });
        }
    }

    let start = std::time::Instant::now();
    let files = collect_source_files(source_dir);
    if files.is_empty() {
        return Err(format!("No source files in {}", source_dir.display()));
    }

    let languages = crate::lang::all_languages();
    let groups = crate::lang::detect::group_by_language(&files);
    let parsed = parser::parse_files_multi(&groups, &languages);
    let syntax = SyntaxOverlay::from_parsed(&parsed);
    let resolved = resolver::resolve_multi(&parsed, &groups, &languages);
    let semantic = SemanticOverlay::from_resolved(&resolved);
    let flow_result = flow::extract_flow_multi(&groups, &languages);
    let flow_overlay = FlowOverlay::from_flow_result(&flow_result);
    let fused = merger::fuse(&[&syntax, &semantic, &flow_overlay]);

    let store = OnDiskStore::new(store_dir);
    store.write_overlay(
        &OverlayKind::Custom("fused".into()),
        fused.nodes(),
        fused.edges(),
    )?;

    let source_hashes = meta::hash_source_files(source_dir);
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    let im = IndexMeta {
        timestamp: now,
        file_count: files.len(),
        node_count: fused.nodes().len(),
        edge_count: fused.edges().len(),
        source_hashes,
    };
    meta::save(store_dir, &im)?;

    Ok(IndexResult {
        file_count: files.len(),
        node_count: fused.nodes().len(),
        edge_count: fused.edges().len(),
        elapsed_ms: start.elapsed().as_millis() as u64,
        is_fresh: false,
    })
}

fn collect_source_files(dir: &Path) -> Vec<(PathBuf, Vec<u8>)> {
    let supported = crate::lang::detect::supported_extensions();
    let mut files = Vec::new();
    let Ok(entries) = std::fs::read_dir(dir) else {
        return files;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            files.extend(collect_source_files(&path));
        } else if path
            .extension()
            .and_then(|e| e.to_str())
            .is_some_and(|e| supported.contains(&e))
        {
            if let Ok(content) = std::fs::read(&path) {
                files.push((path, content));
            }
        }
    }
    files
}
