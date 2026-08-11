use crate::diff::types::{ChangeSet, OwnedDiff};
use crate::graph::node::Span;
use crate::graph::overlay::Overlay;
use crate::identity::UorAddress;
use crate::signal::impact::propagate_impact;
use crate::signal::staleness::propagate_staleness;

/// Maps a diff to affected symbol addresses, then propagates
/// impact forward and staleness backward through overlays.
pub fn detect_changes(diff: &OwnedDiff, overlays: &[&dyn Overlay]) -> ChangeSet {
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

pub fn find_changed_nodes(diff: &OwnedDiff, overlays: &[&dyn Overlay]) -> Vec<UorAddress> {
    let mut changed = Vec::new();
    for diff_file in &diff.files {
        let hunk_ranges: Vec<(u32, u32)> = diff_file
            .hunks
            .iter()
            .map(|h| (h.new_start, h.new_start + h.new_lines))
            .collect();

        for overlay in overlays {
            for node in overlay.nodes() {
                if let Some(span) = &node.span {
                    if hunk_overlaps_span(&hunk_ranges, span) {
                        changed.push(node.address);
                    }
                }
            }
        }
    }
    changed
}

fn hunk_overlaps_span(hunk_ranges: &[(u32, u32)], span: &Span) -> bool {
    hunk_ranges
        .iter()
        .any(|&(start, end)| span.start_line < end && span.end_line >= start)
}
