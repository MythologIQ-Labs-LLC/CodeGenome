use std::path::Path;

use codegenome_identity::graph::edge::{Edge, Relation};
use codegenome_identity::graph::node::Node;
use codegenome_identity::graph::overlay::{Overlay, OverlayKind};
use codegenome_identity::identity::{address_of, UorAddress};
use codegenome_identity::measurement::GroundTruthLevel;
use codegenome_identity::signal::impact::propagate_impact;
use codegenome_identity::store::backend::StoreBackend;
use codegenome_identity::store::meta;
use codegenome_identity::store::ondisk::OnDiskStore;

struct StoredOverlay {
    nodes: Vec<Node>,
    edges: Vec<Edge>,
}

impl Overlay for StoredOverlay {
    fn kind(&self) -> OverlayKind {
        OverlayKind::Custom("stored".into())
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

pub fn run(store_dir: &str, file: &str, line: u32, direction: &str, json: bool) {
    let store = OnDiskStore::new(store_dir);
    let overlay = load_fused(&store);
    let Some(overlay) = overlay else {
        eprintln!("No fused index found at {store_dir}. Run `codegenome index` first.");
        return;
    };

    let indexed_file = match resolve_indexed_file(Path::new(store_dir), file) {
        Ok(path) => path,
        Err(error) => {
            eprintln!("Cannot resolve query file {file}: {error}");
            return;
        }
    };

    let target = match find_node_at(&overlay, &indexed_file, line) {
        Ok(target) => target,
        Err(error) => {
            eprintln!("Cannot resolve symbol at {file}:{line}: {error}");
            return;
        }
    };
    let Some(target_addr) = target else {
        eprintln!("No symbol found at {file}:{line}");
        return;
    };

    let overlays: Vec<&dyn Overlay> = vec![&overlay];
    let impact = propagate_impact(&[target_addr], &overlays);

    let mut results: Vec<_> = impact.iter().filter(|(_, &score)| score > 0.01).collect();
    results.sort_by(|a, b| b.1.partial_cmp(a.1).unwrap_or(std::cmp::Ordering::Equal));

    if json {
        print_json(&results, &overlay);
    } else {
        println!("Impact from {file}:{line} ({direction}):");
        print_human(&results, &overlay);
    }
}

fn load_fused(store: &OnDiskStore) -> Option<StoredOverlay> {
    let (nodes, edges) = store
        .read_overlay(&OverlayKind::Custom("fused".into()))
        .ok()??;
    Some(StoredOverlay { nodes, edges })
}

fn normalize_path(value: &str) -> String {
    let normalized = value.replace('\\', "/");
    normalized
        .strip_prefix("./")
        .unwrap_or(&normalized)
        .trim_end_matches('/')
        .to_string()
}

fn resolve_indexed_file(store_dir: &Path, requested_file: &str) -> Result<String, String> {
    let index = meta::load(store_dir)?
        .ok_or_else(|| format!("no index metadata found at {}", store_dir.display()))?;
    let requested = normalize_path(requested_file);
    if requested.is_empty() {
        return Err("requested file path is empty".into());
    }

    let mut exact = Vec::new();
    let mut suffix = Vec::new();
    for indexed in index.source_hashes.keys() {
        let normalized = normalize_path(indexed);
        if normalized == requested {
            exact.push(indexed.clone());
        } else if normalized.ends_with(&format!("/{requested}")) {
            suffix.push(indexed.clone());
        }
    }

    let matches = if exact.is_empty() { suffix } else { exact };
    match matches.as_slice() {
        [only] => Ok(only.clone()),
        [] => Err(format!("file is not present in the index: {requested_file}")),
        many => Err(format!(
            "file path is ambiguous across {} indexed sources: {}",
            many.len(),
            many.join(", ")
        )),
    }
}

fn find_node_at(
    overlay: &StoredOverlay,
    indexed_file: &str,
    line: u32,
) -> Result<Option<UorAddress>, String> {
    let file_addr = address_of(format!("file:{indexed_file}").as_bytes());
    let contained: std::collections::HashSet<UorAddress> = overlay
        .edges
        .iter()
        .filter(|edge| edge.source == file_addr && edge.relation == Relation::Contains)
        .map(|edge| edge.target)
        .collect();

    if contained.is_empty() {
        return Ok(None);
    }

    let mut candidates: Vec<&Node> = overlay
        .nodes
        .iter()
        .filter(|node| contained.contains(&node.address))
        .filter(|node| {
            node.span
                .as_ref()
                .is_some_and(|span| span.start_line <= line && span.end_line >= line)
        })
        .collect();

    if candidates.is_empty() {
        return Ok(None);
    }

    candidates.sort_by_key(|node| {
        node.span
            .as_ref()
            .map(|span| span.end_line.saturating_sub(span.start_line))
            .unwrap_or(u32::MAX)
    });
    let best_width = candidates[0]
        .span
        .as_ref()
        .map(|span| span.end_line.saturating_sub(span.start_line))
        .unwrap_or(u32::MAX);
    let best: Vec<&Node> = candidates
        .into_iter()
        .take_while(|node| {
            node.span
                .as_ref()
                .map(|span| span.end_line.saturating_sub(span.start_line))
                .unwrap_or(u32::MAX)
                == best_width
        })
        .collect();

    match best.as_slice() {
        [only] => Ok(Some(only.address)),
        many => Err(format!(
            "{} equally specific symbols contain line {line} in {indexed_file}",
            many.len()
        )),
    }
}

fn print_human(results: &[(&UorAddress, &f64)], overlay: &StoredOverlay) {
    for (addr, score) in results.iter().take(20) {
        let loc = node_location(overlay, addr);
        println!("  {loc} (confidence: {score:.4})");
    }
    if results.len() > 20 {
        println!("  ... and {} more", results.len() - 20);
    }
}

fn print_json(results: &[(&UorAddress, &f64)], overlay: &StoredOverlay) {
    let items: Vec<_> = results
        .iter()
        .take(100)
        .map(|(addr, score)| {
            let loc = node_location(overlay, addr);
            serde_json::json!({"node": loc, "confidence": score})
        })
        .collect();
    println!(
        "{}",
        serde_json::to_string_pretty(&items).unwrap_or_default()
    );
}

fn node_location(overlay: &StoredOverlay, addr: &UorAddress) -> String {
    overlay
        .nodes
        .iter()
        .find(|n| n.address == *addr)
        .and_then(|n| n.span.as_ref())
        .map(|s| format!("line {}:{}", s.start_line, s.end_line))
        .unwrap_or_else(|| format!("{addr:?}"))
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use std::fs;
    use std::time::{SystemTime, UNIX_EPOCH};

    use codegenome_identity::graph::node::{NodeKind, Provenance, Timestamp};
    use codegenome_identity::store::meta::IndexMeta;

    use super::*;

    fn test_node(address: UorAddress, span: Option<codegenome_identity::graph::node::Span>) -> Node {
        Node {
            address,
            kind: if span.is_some() {
                NodeKind::Symbol
            } else {
                NodeKind::File
            },
            provenance: Provenance::tool("test", Timestamp(0)),
            confidence: 1.0,
            created_at: Timestamp(0),
            content_hash: address_of(b"content"),
            span,
        }
    }

    fn contains_edge(file: UorAddress, symbol: UorAddress) -> Edge {
        Edge {
            source: file,
            target: symbol,
            relation: Relation::Contains,
            confidence: 1.0,
            provenance: Provenance::tool("test", Timestamp(0)),
            evidence: vec![],
        }
    }

    #[test]
    fn find_node_at_is_scoped_to_requested_file() {
        let a_file = "src/a.rs";
        let b_file = "src/b.rs";
        let a_file_addr = address_of(format!("file:{a_file}").as_bytes());
        let b_file_addr = address_of(format!("file:{b_file}").as_bytes());
        let alpha = address_of(b"alpha");
        let beta = address_of(b"beta");
        let span = codegenome_identity::graph::node::Span {
            start_byte: 0,
            end_byte: 20,
            start_line: 1,
            end_line: 3,
        };
        let overlay = StoredOverlay {
            nodes: vec![
                test_node(a_file_addr, None),
                test_node(alpha, Some(span)),
                test_node(b_file_addr, None),
                test_node(beta, Some(span)),
            ],
            edges: vec![contains_edge(a_file_addr, alpha), contains_edge(b_file_addr, beta)],
        };

        assert_eq!(find_node_at(&overlay, b_file, 2).unwrap(), Some(beta));
        assert_eq!(find_node_at(&overlay, a_file, 2).unwrap(), Some(alpha));
    }

    #[test]
    fn find_node_at_prefers_innermost_symbol_in_file() {
        let file = "src/lib.rs";
        let file_addr = address_of(format!("file:{file}").as_bytes());
        let outer = address_of(b"outer");
        let inner = address_of(b"inner");
        let overlay = StoredOverlay {
            nodes: vec![
                test_node(file_addr, None),
                test_node(
                    outer,
                    Some(codegenome_identity::graph::node::Span {
                        start_byte: 0,
                        end_byte: 100,
                        start_line: 1,
                        end_line: 20,
                    }),
                ),
                test_node(
                    inner,
                    Some(codegenome_identity::graph::node::Span {
                        start_byte: 20,
                        end_byte: 60,
                        start_line: 5,
                        end_line: 8,
                    }),
                ),
            ],
            edges: vec![contains_edge(file_addr, outer), contains_edge(file_addr, inner)],
        };

        assert_eq!(find_node_at(&overlay, file, 6).unwrap(), Some(inner));
    }

    #[test]
    fn resolve_indexed_file_supports_unique_relative_suffix_and_rejects_ambiguity() {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos();
        let dir = std::env::temp_dir().join(format!("codegenome-query-file-{nonce}"));
        fs::create_dir_all(&dir).unwrap();

        let mut hashes = HashMap::new();
        hashes.insert("/workspace/project/src/a.rs".to_string(), "a".into());
        hashes.insert("/workspace/project/src/b.rs".to_string(), "b".into());
        meta::save(
            &dir,
            &IndexMeta {
                timestamp: 0,
                file_count: 2,
                node_count: 0,
                edge_count: 0,
                source_hashes: hashes,
            },
        )
        .unwrap();

        assert_eq!(
            resolve_indexed_file(&dir, "src/b.rs").unwrap(),
            "/workspace/project/src/b.rs"
        );
        assert!(resolve_indexed_file(&dir, "missing.rs").is_err());

        let mut ambiguous = meta::load(&dir).unwrap().unwrap();
        ambiguous
            .source_hashes
            .insert("/other/project/src/b.rs".to_string(), "c".into());
        meta::save(&dir, &ambiguous).unwrap();
        assert!(resolve_indexed_file(&dir, "src/b.rs").is_err());

        let _ = fs::remove_dir_all(&dir);
    }
}
