use std::path::Path;

use crate::experiments::config::{ExperimentParams, FitnessFunction};
use codegenome_identity::graph::edge::Relation;
use codegenome_identity::graph::node::NodeKind;
use codegenome_identity::graph::overlay::Overlay;
use codegenome_identity::identity::UorAddress;
use codegenome_identity::overlay::flow::FlowOverlay;
use codegenome_identity::overlay::fused::{self, FusedOverlay};
use codegenome_identity::overlay::semantic::SemanticOverlay;
use codegenome_identity::overlay::syntax::SyntaxOverlay;

pub use crate::experiments::fitness_fns;

/// Dispatch a fitness function by enum variant.
pub fn dispatch(func: &FitnessFunction, source_dir: &Path, params: &ExperimentParams) -> f64 {
    match func {
        FitnessFunction::ImpactAccuracy => impact_accuracy(source_dir, params),
        FitnessFunction::PropagationDepth => fitness_fns::propagation_depth(source_dir, params),
        FitnessFunction::CycleTime => fitness_fns::cycle_time(source_dir, params),
        FitnessFunction::GraphDensity => fitness_fns::graph_density(source_dir, params),
        FitnessFunction::Custom(_) => 0.0,
    }
}

/// Impact prediction accuracy: for each sampled symbol,
/// propagate from it with attenuation and check what
/// fraction of its siblings are reached above threshold.
pub fn impact_accuracy(source_dir: &Path, params: &ExperimentParams) -> f64 {
    let Some(fused) = build_overlays(source_dir) else {
        return 0.0;
    };
    let symbols = symbols_with_spans(&fused);
    if symbols.is_empty() {
        return 0.0;
    }

    let threshold = param_or(params, "confidence_threshold", 0.5);
    let atten = param_or(params, "attenuation_factor", 0.8);
    let overlays: Vec<&dyn Overlay> = vec![&fused];
    let sample_size = symbols.len().min(10);
    let mut total_score = 0.0;
    let mut total_tests = 0u32;

    for symbol in symbols.iter().take(sample_size) {
        let siblings = find_siblings(&fused, symbol.address);
        if siblings.is_empty() {
            continue;
        }
        let parent = find_parent(&fused, symbol.address);
        let root = parent.unwrap_or(symbol.address);
        let impact = depth_propagate(root, &overlays, atten, threshold);
        let sibling_hits: f64 = siblings
            .iter()
            .filter_map(|addr| impact.get(addr))
            .map(|&score| score.clamp(0.0, 1.0))
            .sum();
        total_score += sibling_hits / siblings.len() as f64;
        total_tests += 1;
    }

    if total_tests == 0 {
        return 0.0;
    }
    total_score / total_tests as f64
}

/// Stability: run depth-attenuated propagation with two
/// different attenuation factors and measure how many nodes'
/// inclusion changes. Uses BFS depth to apply attenuation.
pub fn stability(source_dir: &Path, params: &ExperimentParams) -> f64 {
    let Some(fused) = build_overlays(source_dir) else {
        return 1.0;
    };
    let file_nodes: Vec<_> = fused
        .nodes()
        .iter()
        .filter(|n| n.kind == NodeKind::File)
        .collect();
    if file_nodes.is_empty() {
        return 1.0;
    }

    let atten = param_or(params, "attenuation_factor", 0.8);
    let threshold = param_or(params, "confidence_threshold", 0.5);
    let overlays: Vec<&dyn Overlay> = vec![&fused];
    let root = file_nodes[0].address;

    let set_a = depth_propagate(root, &overlays, atten, threshold);
    let perturbed = atten * 0.9;
    let set_b = depth_propagate(root, &overlays, perturbed, threshold);

    let all_keys: std::collections::HashSet<_> = set_a.keys().chain(set_b.keys()).collect();
    let total = all_keys.len().max(1) as f64;
    let sum_diff: f64 = all_keys
        .iter()
        .map(|addr| {
            let a = set_a.get(addr).unwrap_or(&0.0);
            let b = set_b.get(addr).unwrap_or(&0.0);
            (a - b).abs()
        })
        .sum();
    let mean_diff = sum_diff / total;

    (1.0 - mean_diff).clamp(0.0, 1.0)
}

/// Build all three overlays and fuse into one. Returns None if empty.
/// Uses the multi-language pipeline: the fitness signal observes every
/// supported language, not only Rust, so the engine optimizes against
/// the same graph the indexer produces.
pub(crate) fn build_overlays(source_dir: &Path) -> Option<FusedOverlay> {
    let files = codegenome_identity::index::orchestrator::collect_source_files(source_dir);
    if files.is_empty() {
        return None;
    }
    let languages = codegenome_identity::lang::all_languages();
    let groups = codegenome_identity::lang::detect::group_by_language(&files);
    let parsed = codegenome_identity::index::parser::parse_files_multi(&groups, &languages);
    let syntax = SyntaxOverlay::from_parsed(&parsed);
    let resolved =
        codegenome_identity::index::resolver_multi::resolve_multi(&parsed, &groups, &languages);
    let semantic = SemanticOverlay::from_resolved(&resolved);
    let flow_result = codegenome_identity::index::flow::extract_flow_multi(&groups, &languages);
    let flow = FlowOverlay::from_flow_result(&flow_result);
    Some(fused::fuse(&[&syntax, &semantic, &flow]))
}

/// BFS propagation that applies attenuation per hop.
pub(crate) fn depth_propagate(
    root: UorAddress,
    overlays: &[&dyn Overlay],
    attenuation: f64,
    threshold: f64,
) -> std::collections::HashMap<UorAddress, f64> {
    use std::collections::{HashMap, VecDeque};
    let mut scores: HashMap<UorAddress, f64> = HashMap::new();
    scores.insert(root, 1.0);
    let mut queue = VecDeque::new();
    queue.push_back((root, 1.0));

    while let Some((node, score)) = queue.pop_front() {
        for overlay in overlays {
            for edge in overlay.edges() {
                if edge.source != node {
                    continue;
                }
                let child_score = (score * edge.confidence * attenuation).min(1.0);
                if child_score < threshold {
                    continue;
                }
                let entry = scores.entry(edge.target).or_insert(0.0);
                if child_score > *entry {
                    *entry = child_score;
                    queue.push_back((edge.target, child_score));
                }
            }
        }
    }
    scores
}

fn find_parent(overlay: &dyn Overlay, addr: UorAddress) -> Option<UorAddress> {
    overlay
        .edges()
        .iter()
        .find(|e| e.target == addr && e.relation == Relation::Contains)
        .map(|e| e.source)
}

/// Find sibling symbols: nodes sharing a Contains parent
/// with `addr` in the same file.
fn find_siblings(overlay: &dyn Overlay, addr: UorAddress) -> Vec<UorAddress> {
    let parents: Vec<UorAddress> = overlay
        .edges()
        .iter()
        .filter(|e| e.target == addr && e.relation == Relation::Contains)
        .map(|e| e.source)
        .collect();

    let mut siblings = Vec::new();
    for edge in overlay.edges() {
        if edge.relation == Relation::Contains
            && parents.contains(&edge.source)
            && edge.target != addr
        {
            siblings.push(edge.target);
        }
    }
    siblings
}

fn symbols_with_spans(overlay: &dyn Overlay) -> Vec<codegenome_identity::graph::node::Node> {
    overlay
        .nodes()
        .iter()
        .filter(|n| n.kind == NodeKind::Symbol && n.span.is_some())
        .cloned()
        .collect()
}

pub(crate) fn collect_rs_files(dir: &Path) -> Vec<(std::path::PathBuf, Vec<u8>)> {
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
                .is_some_and(codegenome_identity::lang::detect::is_excluded_dir);
            if !excluded {
                files.extend(collect_rs_files(&path));
            }
        } else if path.extension().is_some_and(|e| e == "rs") {
            if let Ok(content) = std::fs::read(&path) {
                files.push((path, content));
            }
        }
    }
    files
}

pub(crate) fn param_or(params: &ExperimentParams, key: &str, default: f64) -> f64 {
    params.values.get(key).copied().unwrap_or(default)
}
