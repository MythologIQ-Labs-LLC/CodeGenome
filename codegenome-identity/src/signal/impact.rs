use std::collections::HashMap;

use crate::graph::overlay::Overlay;
use crate::graph::query::Direction;
use crate::identity::UorAddress;
use crate::signal::topo::topological_sort;

/// Propagate impact forward from changed nodes using stored edge orientation.
/// Each edge attenuates by its confidence.
/// Returns impact map: address -> score [0.0, 1.0].
pub fn propagate_impact(
    changed: &[UorAddress],
    overlays: &[&dyn Overlay],
) -> HashMap<UorAddress, f64> {
    propagate_impact_directional(changed, overlays, Direction::Downstream)
}

/// Propagate impact according to the graph's declared direction semantics.
///
/// - `Downstream` follows stored `source -> target` edges.
/// - `Upstream` follows the reverse `target -> source` orientation.
/// - `Both` computes both directions independently and keeps the maximum
///   confidence for nodes reachable from either side.
///
/// The calculation preserves the graph relation and confidence evidence. It
/// does not reinterpret individual relation kinds or manufacture new edges.
pub fn propagate_impact_directional(
    changed: &[UorAddress],
    overlays: &[&dyn Overlay],
    direction: Direction,
) -> HashMap<UorAddress, f64> {
    match direction {
        Direction::Downstream => propagate_one_direction(changed, overlays, Direction::Downstream),
        Direction::Upstream => propagate_one_direction(changed, overlays, Direction::Upstream),
        Direction::Both => {
            let mut combined =
                propagate_one_direction(changed, overlays, Direction::Downstream);
            let upstream = propagate_one_direction(changed, overlays, Direction::Upstream);
            for (addr, score) in upstream {
                let entry = combined.entry(addr).or_insert(0.0);
                if score > *entry {
                    *entry = score;
                }
            }
            combined
        }
    }
}

fn propagate_one_direction(
    changed: &[UorAddress],
    overlays: &[&dyn Overlay],
    direction: Direction,
) -> HashMap<UorAddress, f64> {
    debug_assert!(direction != Direction::Both);
    let sorted = topological_sort(changed, overlays, direction.clone());

    let mut impact: HashMap<UorAddress, f64> = HashMap::new();
    for &addr in changed {
        impact.insert(addr, 1.0);
    }

    let edge_index = build_directional_index(overlays, &direction);

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

fn build_directional_index(
    overlays: &[&dyn Overlay],
    direction: &Direction,
) -> HashMap<UorAddress, Vec<(UorAddress, f64)>> {
    let mut index: HashMap<UorAddress, Vec<(UorAddress, f64)>> = HashMap::new();
    for overlay in overlays {
        for edge in overlay.edges() {
            let (from, to) = match direction {
                Direction::Downstream => (edge.source, edge.target),
                Direction::Upstream => (edge.target, edge.source),
                Direction::Both => unreachable!("Both is composed from independent directional runs"),
            };
            index.entry(from).or_default().push((to, edge.confidence));
        }
    }
    index
}
