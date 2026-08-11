use std::collections::HashMap;

use crate::graph::overlay::Overlay;
use crate::graph::query::Direction;
use crate::identity::UorAddress;
use crate::signal::topo::topological_sort;

/// Propagate staleness backward from stale nodes.
/// Returns staleness map: address -> score [0.0, 1.0].
pub fn propagate_staleness(
    stale_roots: &[UorAddress],
    overlays: &[&dyn Overlay],
) -> HashMap<UorAddress, f64> {
    let sorted = topological_sort(stale_roots, overlays, Direction::Upstream);

    let mut staleness: HashMap<UorAddress, f64> = HashMap::new();
    for &addr in stale_roots {
        staleness.insert(addr, 1.0);
    }

    let incoming_index = build_incoming_index(overlays);

    for &node in &sorted {
        let node_staleness = *staleness.get(&node).unwrap_or(&0.0);
        if node_staleness == 0.0 {
            continue;
        }
        for &(source, confidence) in incoming_index.get(&node).unwrap_or(&vec![]) {
            let propagated = node_staleness * confidence;
            let entry = staleness.entry(source).or_insert(0.0);
            if propagated > *entry {
                *entry = propagated;
            }
        }
    }

    staleness
}

fn build_incoming_index(overlays: &[&dyn Overlay]) -> HashMap<UorAddress, Vec<(UorAddress, f64)>> {
    let mut index: HashMap<UorAddress, Vec<(UorAddress, f64)>> = HashMap::new();
    for overlay in overlays {
        for edge in overlay.edges() {
            index
                .entry(edge.target)
                .or_default()
                .push((edge.source, edge.confidence));
        }
    }
    index
}
