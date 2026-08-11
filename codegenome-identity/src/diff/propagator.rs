use crate::diff::mapper::find_changed_nodes;
use crate::diff::types::{ChangeSet, OwnedDiff};
use crate::graph::overlay::Overlay;
use crate::identity::UorAddress;
use crate::signal::impact::propagate_impact;
use crate::signal::staleness::propagate_staleness;

/// Map a diff to affected symbols, propagate impact forward
/// and staleness backward through overlays.
pub fn propagate(diff: &OwnedDiff, overlays: &[&dyn Overlay]) -> ChangeSet {
    let changed_nodes = find_changed_nodes(diff, overlays);

    let impact = propagate_impact(&changed_nodes, overlays);
    let staleness = propagate_staleness(&changed_nodes, overlays);

    let touched: std::collections::HashSet<UorAddress> = changed_nodes.iter().copied().collect();

    let mut affected_edges = Vec::new();
    for overlay in overlays {
        for edge in overlay.edges() {
            if touched.contains(&edge.source) || touched.contains(&edge.target) {
                affected_edges.push(edge.source);
            }
        }
    }

    ChangeSet {
        changed_nodes,
        affected_edges,
        impact,
        staleness,
    }
}
