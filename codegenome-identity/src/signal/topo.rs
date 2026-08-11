use std::collections::{HashMap, HashSet, VecDeque};

use crate::graph::overlay::Overlay;
use crate::graph::query::Direction;
use crate::identity::UorAddress;

/// Topological ordering of nodes reachable from roots.
/// Pure function: overlays in -> sorted addresses out.
pub fn topological_sort(
    roots: &[UorAddress],
    overlays: &[&dyn Overlay],
    direction: Direction,
) -> Vec<UorAddress> {
    let adj = build_adjacency(overlays, &direction);
    let reachable = bfs_reachable(roots, &adj);
    topo_order(&reachable, &adj)
}

fn build_adjacency(
    overlays: &[&dyn Overlay],
    direction: &Direction,
) -> HashMap<UorAddress, Vec<UorAddress>> {
    let mut adj: HashMap<UorAddress, Vec<UorAddress>> = HashMap::new();
    for overlay in overlays {
        for edge in overlay.edges() {
            let (from, to) = match direction {
                Direction::Downstream | Direction::Both => (edge.source, edge.target),
                Direction::Upstream => (edge.target, edge.source),
            };
            adj.entry(from).or_default().push(to);
            adj.entry(to).or_default();
        }
    }
    adj
}

fn bfs_reachable(
    roots: &[UorAddress],
    adj: &HashMap<UorAddress, Vec<UorAddress>>,
) -> HashSet<UorAddress> {
    let mut visited = HashSet::new();
    let mut queue = VecDeque::new();
    for &root in roots {
        if adj.contains_key(&root) {
            visited.insert(root);
            queue.push_back(root);
        }
    }
    while let Some(node) = queue.pop_front() {
        for &neighbor in adj.get(&node).unwrap_or(&vec![]) {
            if visited.insert(neighbor) {
                queue.push_back(neighbor);
            }
        }
    }
    visited
}

fn topo_order(
    nodes: &HashSet<UorAddress>,
    adj: &HashMap<UorAddress, Vec<UorAddress>>,
) -> Vec<UorAddress> {
    let mut visited = HashSet::new();
    let mut order = Vec::new();
    for &node in nodes {
        if !visited.contains(&node) {
            dfs_post(node, adj, nodes, &mut visited, &mut order);
        }
    }
    order.reverse();
    order
}

fn dfs_post(
    node: UorAddress,
    adj: &HashMap<UorAddress, Vec<UorAddress>>,
    scope: &HashSet<UorAddress>,
    visited: &mut HashSet<UorAddress>,
    order: &mut Vec<UorAddress>,
) {
    visited.insert(node);
    for &neighbor in adj.get(&node).unwrap_or(&vec![]) {
        if scope.contains(&neighbor) && !visited.contains(&neighbor) {
            dfs_post(neighbor, adj, scope, visited, order);
        }
    }
    order.push(node);
}
