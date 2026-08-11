use std::collections::HashMap;

use crate::graph::overlay::Overlay;
use crate::graph::query::Direction;
use crate::identity::UorAddress;
use crate::signal::topo::topological_sort;

/// Propagate impact forward from changed nodes.
/// Each edge attenuates by its confidence.
/// Returns impact map: address -> score [0.0, 1.0].
pub fn propagate_impact(
    changed: &[UorAddress],
    overlays: &[&dyn Overlay],
) -> HashMap<UorAddress, f64> {
    let sorted = topological_sort(changed, overlays, Direction::Downstream);

    let mut impact: HashMap<UorAddress, f64> = HashMap::new();
    for &addr in changed {
        impact.insert(addr, 1.0);
    }

    let edge_index = build_outgoing_index(overlays);

    for &node in &sorted {
        let node_impact = *impact.get(&node).unwrap_or(&0.0);
        if node_impact == 0.0 {
            continue;
        }
        for &(target, confidence) in edge_index.get(&node).unwrap_or(&vec![]) {
            let propagated = node_impact * confidence;
            let entry = impact.entry(target).or_insert(0.0);
            if propagated > *entry {
                *entry = propagated;
            }
        }
    }

    impact
}

fn build_outgoing_index(overlays: &[&dyn Overlay]) -> HashMap<UorAddress, Vec<(UorAddress, f64)>> {
    let mut index: HashMap<UorAddress, Vec<(UorAddress, f64)>> = HashMap::new();
    for overlay in overlays {
        for edge in overlay.edges() {
            index
                .entry(edge.source)
                .or_default()
                .push((edge.target, edge.confidence));
        }
    }
    index
}
