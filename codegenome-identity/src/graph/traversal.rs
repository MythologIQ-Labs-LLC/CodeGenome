use std::collections::{HashSet, VecDeque};

use crate::graph::query::{Query, QueryResult};
use crate::graph::query_context::QueryContext;
use crate::identity::UorAddress;

/// Execute a query against a QueryContext. Pure function.
pub fn execute(query: &Query, ctx: &dyn QueryContext) -> QueryResult {
    let (reachable, paths) = bfs_paths(query, ctx);
    let node_set: HashSet<UorAddress> = reachable.iter().copied().collect();

    QueryResult {
        nodes: ctx.collect_nodes(&node_set),
        edges: ctx.collect_edges(&node_set),
        confidence: if paths.is_empty() { 0.0 } else { 1.0 },
        paths,
    }
}

fn bfs_paths(query: &Query, ctx: &dyn QueryContext) -> (Vec<UorAddress>, Vec<Vec<UorAddress>>) {
    let mut visited = HashSet::new();
    let mut queue: VecDeque<(UorAddress, Vec<UorAddress>, f64)> = VecDeque::new();
    let mut reachable = Vec::new();
    let mut paths = Vec::new();

    visited.insert(query.target);
    reachable.push(query.target);
    queue.push_back((query.target, vec![query.target], 1.0));

    while let Some((node, path, conf)) = queue.pop_front() {
        let depth = path.len() - 1;
        if depth >= query.max_depth as usize {
            continue;
        }
        let neighbors = ctx.neighbors(
            node,
            &query.direction,
            &query.relation_filter,
            0.0, // raw neighbors; we apply cumulative threshold below
        );
        for (next, edge_conf, _) in neighbors {
            let new_conf = conf * edge_conf;
            if new_conf < query.min_confidence {
                continue;
            }
            if visited.insert(next) {
                reachable.push(next);
                let mut new_path = path.clone();
                new_path.push(next);
                if new_path.len() > 1 {
                    paths.push(new_path.clone());
                }
                queue.push_back((next, new_path, new_conf));
            }
        }
    }

    (reachable, paths)
}
