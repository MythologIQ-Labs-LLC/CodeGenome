use std::collections::{HashMap, HashSet};

use crate::graph::edge::{Edge, Relation};
use crate::graph::node::Node;
use crate::graph::query::Direction;
use crate::identity::UorAddress;

/// Narrow trait for query execution. Provides graph access
/// without exposing storage concerns. Single-repo and
/// federated contexts both implement this.
pub trait QueryContext {
    fn neighbors(
        &self,
        addr: UorAddress,
        direction: &Direction,
        relation_filter: &Option<Vec<Relation>>,
        min_confidence: f64,
    ) -> Vec<(UorAddress, f64, Relation)>;

    fn node(&self, addr: UorAddress) -> Option<Node>;

    fn collect_nodes(&self, addrs: &HashSet<UorAddress>) -> Vec<Node>;

    fn collect_edges(&self, addrs: &HashSet<UorAddress>) -> Vec<Edge>;
}

/// QueryContext over a single fused overlay (nodes + edges).
/// Pre-builds adjacency indexes on construction.
pub struct LocalQueryContext<'a> {
    nodes: &'a [Node],
    edges: &'a [Edge],
    fwd: AdjIndex,
    rev: AdjIndex,
}

type AdjEntry = (UorAddress, f64, Relation);
type AdjIndex = HashMap<UorAddress, Vec<AdjEntry>>;

impl<'a> LocalQueryContext<'a> {
    pub fn new(nodes: &'a [Node], edges: &'a [Edge]) -> Self {
        let mut fwd: AdjIndex = HashMap::new();
        let mut rev: AdjIndex = HashMap::new();
        for e in edges {
            fwd.entry(e.source)
                .or_default()
                .push((e.target, e.confidence, e.relation.clone()));
            rev.entry(e.target)
                .or_default()
                .push((e.source, e.confidence, e.relation.clone()));
        }
        Self { nodes, edges, fwd, rev }
    }
}

impl<'a> QueryContext for LocalQueryContext<'a> {
    fn neighbors(
        &self,
        addr: UorAddress,
        direction: &Direction,
        relation_filter: &Option<Vec<Relation>>,
        min_confidence: f64,
    ) -> Vec<(UorAddress, f64, Relation)> {
        let raw = match direction {
            Direction::Downstream => self.fwd.get(&addr),
            Direction::Upstream => self.rev.get(&addr),
            Direction::Both => {
                let mut combined = Vec::new();
                if let Some(f) = self.fwd.get(&addr) {
                    combined.extend(f.iter().cloned());
                }
                if let Some(r) = self.rev.get(&addr) {
                    combined.extend(r.iter().cloned());
                }
                return filter_entries(combined, relation_filter, min_confidence);
            }
        };
        let entries = raw.cloned().unwrap_or_default();
        filter_entries(entries, relation_filter, min_confidence)
    }

    fn node(&self, addr: UorAddress) -> Option<Node> {
        self.nodes.iter().find(|n| n.address == addr).cloned()
    }

    fn collect_nodes(&self, addrs: &HashSet<UorAddress>) -> Vec<Node> {
        self.nodes
            .iter()
            .filter(|n| addrs.contains(&n.address))
            .cloned()
            .collect()
    }

    fn collect_edges(&self, addrs: &HashSet<UorAddress>) -> Vec<Edge> {
        self.edges
            .iter()
            .filter(|e| addrs.contains(&e.source) && addrs.contains(&e.target))
            .cloned()
            .collect()
    }
}

fn filter_entries(
    entries: Vec<AdjEntry>,
    relation_filter: &Option<Vec<Relation>>,
    min_confidence: f64,
) -> Vec<AdjEntry> {
    entries
        .into_iter()
        .filter(|(_, conf, rel)| {
            *conf >= min_confidence
                && relation_filter
                    .as_ref()
                    .is_none_or(|f| f.contains(rel))
        })
        .collect()
}
