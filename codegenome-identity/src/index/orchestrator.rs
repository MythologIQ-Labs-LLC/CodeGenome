use std::path::{Path, PathBuf};
use std::sync::Mutex;

use crate::graph::overlay::{Overlay, OverlayKind};
use crate::index::cache::{CachedEntry, FileCache};
use crate::index::{dynamic, flow, merger, parser, resolver, IndexResult};
use crate::overlay::flow::FlowOverlay;
use crate::overlay::runtime::RuntimeOverlay;
use crate::overlay::semantic::SemanticOverlay;
use crate::overlay::syntax::SyntaxOverlay;
use crate::store::backend::StoreBackend;
use crate::store::meta::{self, IndexMeta};
use crate::store::ondisk::OnDiskStore;

pub struct PipelineConfig {
    pub source_dir: PathBuf,
    pub store_dir: PathBuf,
    pub trace_path: Option<PathBuf>,
}

pub fn run(config: &PipelineConfig) -> Result<IndexResult, String> {
    let freshness = meta::check_freshness(&config.store_dir, &config.source_dir);
    if freshness.is_fresh {
        if let Ok(Some(m)) = meta::load(&config.store_dir) {
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
    let files = collect_source_files(&config.source_dir);
    if files.is_empty() {
        return Err(format!(
            "No source files in {}",
            config.source_dir.display()
        ));
    }

    let languages = crate::lang::all_languages();
    let groups = crate::lang::detect::group_by_language(&files);

    // Stage 1: Parse (incremental via cache for Rust, fresh for others)
    let parsed = parse_with_cache_multi(&files, &languages, &config.store_dir);

    // Stage 2: Build syntax overlay (needed before fan-out)
    let syntax = SyntaxOverlay::from_parsed(&parsed);

    // Stage 3: Rayon fan-out for independent overlays
    let semantic = Mutex::new(None);
    let flow_overlay = Mutex::new(None);
    let runtime = Mutex::new(None);

    let trace = config.trace_path.clone();
    let files_ref = &files;
    let parsed_ref = &parsed;
    let groups_ref = &groups;
    let langs_ref = &languages;

    rayon::scope(|s| {
        s.spawn(|_| {
            let resolved = resolver::resolve_multi(parsed_ref, groups_ref, langs_ref);
            *semantic.lock().unwrap() = Some(SemanticOverlay::from_resolved(&resolved));
        });
        s.spawn(|_| {
            let result = flow::extract_flow_multi(groups_ref, langs_ref);
            *flow_overlay.lock().unwrap() = Some(FlowOverlay::from_flow_result(&result));
        });
        if let Some(ref tp) = trace {
            s.spawn(|_| {
                if let Ok(result) = dynamic::ingest_trace(tp, files_ref) {
                    *runtime.lock().unwrap() = Some(RuntimeOverlay::from_trace(&result));
                }
            });
        }
    });

    // Stage 4: Collect + fuse
    let sem = semantic.into_inner().unwrap().unwrap();
    let fl = flow_overlay.into_inner().unwrap().unwrap();
    let rt = runtime.into_inner().unwrap();

    let fused = if let Some(ref rt_overlay) = rt {
        merger::fuse(&[&syntax, &sem, &fl, rt_overlay])
    } else {
        merger::fuse(&[&syntax, &sem, &fl])
    };

    // Stage 5: Store
    let store = OnDiskStore::new(&config.store_dir);
    store.write_overlay(
        &OverlayKind::Custom("fused".into()),
        fused.nodes(),
        fused.edges(),
    )?;

    let source_hashes = meta::hash_source_files(&config.source_dir);
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
    meta::save(&config.store_dir, &im)?;

    Ok(IndexResult {
        file_count: files.len(),
        node_count: fused.nodes().len(),
        edge_count: fused.edges().len(),
        elapsed_ms: start.elapsed().as_millis() as u64,
        is_fresh: false,
    })
}

fn parse_with_cache_multi(
    files: &[(PathBuf, Vec<u8>)],
    languages: &[Box<dyn crate::lang::LanguageSupport>],
    store_dir: &Path,
) -> Vec<parser::ParsedFile> {
    let cache = FileCache::new(store_dir);
    let mut parsed = Vec::with_capacity(files.len());
    let mut dirty: Vec<(PathBuf, Vec<u8>, String)> = Vec::new();

    for (path, source) in files {
        let hash = blake3::hash(source).to_hex()[..16].to_string();
        if let Some(entry) = cache.get(path, &hash) {
            if let (Ok(nodes), Ok(edges)) = (
                bincode::deserialize(&entry.nodes_bin),
                bincode::deserialize(&entry.edges_bin),
            ) {
                let file_content = format!("file:{}", path.display());
                parsed.push(parser::ParsedFile {
                    path: path.clone(),
                    file_address: crate::identity::address_of(file_content.as_bytes()),
                    content_hash: crate::identity::address_of(source),
                    nodes,
                    edges,
                });
                continue;
            }
        }
        dirty.push((path.clone(), source.clone(), hash));
    }

    if !dirty.is_empty() {
        let dirty_files: Vec<_> = dirty
            .iter()
            .map(|(p, s, _)| (p.clone(), s.clone()))
            .collect();
        let dirty_groups = crate::lang::detect::group_by_language(&dirty_files);
        let fresh = parser::parse_files_multi(&dirty_groups, languages);
        for pf in fresh {
            let hash = dirty
                .iter()
                .find(|(p, _, _)| *p == pf.path)
                .map(|(_, _, h)| h.clone())
                .unwrap_or_default();
            let entry = CachedEntry {
                content_hash: hash,
                nodes_bin: bincode::serialize(&pf.nodes).unwrap_or_default(),
                edges_bin: bincode::serialize(&pf.edges).unwrap_or_default(),
            };
            let _ = cache.put(&pf.path, &entry);
            parsed.push(pf);
        }
    }

    parsed
}

/// Recursively collect all supported source files, skipping excluded
/// directories (hidden, target/, node_modules/, __pycache__/, venv/,
/// vendor/). Public so callers outside the pipeline (e.g. the
/// experiment engine's fitness functions) collect the same file set
/// the indexer sees.
pub fn collect_source_files(dir: &Path) -> Vec<(PathBuf, Vec<u8>)> {
    let supported = crate::lang::detect::supported_extensions();
    let mut files = Vec::new();
    let Ok(entries) = std::fs::read_dir(dir) else {
        return files;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            let excluded = path
                .file_name()
                .and_then(|n| n.to_str())
                .is_some_and(crate::lang::detect::is_excluded_dir);
            if !excluded {
                files.extend(collect_source_files(&path));
            }
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
