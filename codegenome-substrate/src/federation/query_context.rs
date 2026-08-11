use std::collections::{HashMap, HashSet};

use crate::federation::config::WorkspaceConfig;
use crate::federation::workspace::WorkspaceGraph;
use codegenome_identity::graph::edge::{Edge, Relation};
use codegenome_identity::graph::node::Node;
use codegenome_identity::graph::overlay::OverlayKind;
use codegenome_identity::graph::query::Direction;
use codegenome_identity::graph::query_context::QueryContext;
use codegenome_identity::identity::UorAddress;
use codegenome_identity::store::backend::StoreBackend;
use codegenome_identity::store::ondisk::OnDiskStore;

/// QueryContext that composes repo-local graphs with cross-repo
/// edges. Queries seamlessly traverse repo boundaries via
/// federated + symbol edges. No special traversal rules.
pub struct FederatedQueryContext {
    nodes: Vec<Node>,
    fwd: HashMap<UorAddress, Vec<(UorAddress, f64, Relation)>>,
    rev: HashMap<UorAddress, Vec<(UorAddress, f64, Relation)>>,
    all_edges: Vec<Edge>,
}

impl FederatedQueryContext {
    pub fn from_workspace(graph: &WorkspaceGraph, cfg: &WorkspaceConfig) -> Self {
        let mut nodes = graph.aggregate_nodes.clone();
        let mut all_edges: Vec<Edge> = graph.federated_edges.clone();
        all_edges.extend(graph.symbol_edges.iter().cloned());

        // Load per-repo fused overlays
        for repo in &cfg.repositories {
            let store = OnDiskStore::new(&repo.store_dir);
            if let Ok(Some((repo_nodes, repo_edges))) =
                store.read_overlay(&OverlayKind::Custom("fused".into()))
            {
                nodes.extend(repo_nodes);
                all_edges.extend(repo_edges);
            }
        }

        let mut fwd: HashMap<UorAddress, Vec<_>> = HashMap::new();
        let mut rev: HashMap<UorAddress, Vec<_>> = HashMap::new();
        for e in &all_edges {
            fwd.entry(e.source)
                .or_default()
                .push((e.target, e.confidence, e.relation.clone()));
            rev.entry(e.target)
                .or_default()
                .push((e.source, e.confidence, e.relation.clone()));
        }

        Self {
            nodes,
            fwd,
            rev,
            all_edges,
        }
    }

    /// Build from pre-loaded data (for testing without disk I/O).
    pub fn from_parts(nodes: Vec<Node>, local_edges: Vec<Edge>, cross_edges: Vec<Edge>) -> Self {
        let mut all_edges = local_edges;
        all_edges.extend(cross_edges);

        let mut fwd: HashMap<UorAddress, Vec<_>> = HashMap::new();
        let mut rev: HashMap<UorAddress, Vec<_>> = HashMap::new();
        for e in &all_edges {
            fwd.entry(e.source)
                .or_default()
                .push((e.target, e.confidence, e.relation.clone()));
            rev.entry(e.target)
                .or_default()
                .push((e.source, e.confidence, e.relation.clone()));
        }

        Self {
            nodes,
            fwd,
            rev,
            all_edges,
        }
    }
}

impl QueryContext for FederatedQueryContext {
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
                return filter(combined, relation_filter, min_confidence);
            }
        };
        filter(
            raw.cloned().unwrap_or_default(),
            relation_filter,
            min_confidence,
        )
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
        self.all_edges
            .iter()
            .filter(|e| addrs.contains(&e.source) && addrs.contains(&e.target))
            .cloned()
            .collect()
    }
}

fn filter(
    entries: Vec<(UorAddress, f64, Relation)>,
    relation_filter: &Option<Vec<Relation>>,
    min_confidence: f64,
) -> Vec<(UorAddress, f64, Relation)> {
    entries
        .into_iter()
        .filter(|(_, conf, rel)| {
            *conf >= min_confidence && relation_filter.as_ref().is_none_or(|f| f.contains(rel))
        })
        .collect()
}
