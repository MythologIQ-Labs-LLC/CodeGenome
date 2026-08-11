use codegenome_identity::graph::edge::Edge;
use codegenome_identity::graph::node::Node;
use codegenome_identity::graph::overlay::{Overlay, OverlayKind};
use codegenome_identity::measurement::GroundTruthLevel;
use codegenome_identity::signal::impact::propagate_impact;
use codegenome_identity::store::backend::StoreBackend;
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

    let target = find_node_at(&overlay, file, line);
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

fn find_node_at(
    overlay: &StoredOverlay,
    _file: &str,
    line: u32,
) -> Option<codegenome_identity::identity::UorAddress> {
    overlay
        .nodes
        .iter()
        .find(|n| {
            n.span
                .as_ref()
                .is_some_and(|s| s.start_line <= line && s.end_line >= line)
        })
        .map(|n| n.address)
}

fn print_human(
    results: &[(&codegenome_identity::identity::UorAddress, &f64)],
    overlay: &StoredOverlay,
) {
    for (addr, score) in results.iter().take(20) {
        let loc = node_location(overlay, addr);
        println!("  {loc} (confidence: {score:.4})");
    }
    if results.len() > 20 {
        println!("  ... and {} more", results.len() - 20);
    }
}

fn print_json(
    results: &[(&codegenome_identity::identity::UorAddress, &f64)],
    overlay: &StoredOverlay,
) {
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

fn node_location(
    overlay: &StoredOverlay,
    addr: &codegenome_identity::identity::UorAddress,
) -> String {
    overlay
        .nodes
        .iter()
        .find(|n| n.address == *addr)
        .and_then(|n| n.span.as_ref())
        .map(|s| format!("line {}:{}", s.start_line, s.end_line))
        .unwrap_or_else(|| format!("{addr:?}"))
}
