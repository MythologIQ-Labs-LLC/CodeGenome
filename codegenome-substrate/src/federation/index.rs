use crate::federation::config::WorkspaceConfig;
use crate::federation::evidence;
use crate::federation::workspace::{RepositoryMember, WorkspaceGraph};
use codegenome_identity::graph::edge::{Edge, Relation};
use codegenome_identity::graph::node::{Node, NodeKind, Provenance, Timestamp};
use codegenome_identity::graph::overlay::OverlayKind;
use codegenome_identity::identity::{address_of, UorAddress};
use codegenome_identity::store::backend::StoreBackend;
use codegenome_identity::store::meta::{self, WorkspaceMeta};
use codegenome_identity::store::ondisk::OnDiskStore;

pub fn build_workspace(
    cfg: &WorkspaceConfig,
    store_dir: &std::path::Path,
) -> Result<WorkspaceGraph, String> {
    let repos = repo_nodes(cfg);
    let repo_overlays = load_repo_nodes(cfg)?;
    let mut edges = collect_edges(cfg, &repos, &repo_overlays);
    edges.sort_by(|a, b| {
        format!("{:?}{:?}{:?}", a.source, a.target, a.relation)
            .cmp(&format!("{:?}{:?}{:?}", b.source, b.target, b.relation))
    });
    let graph = WorkspaceGraph {
        workspace_id: cfg.workspace_id.clone(),
        repositories: ordered_members(cfg, &repos),
        aggregate_nodes: cfg
            .repositories
            .iter()
            .filter_map(|repo| repos.get(&repo.name).cloned())
            .collect(),
        federated_edges: edges,
        symbol_edges: evidence::symbol_edges(cfg),
    };
    persist_workspace(&graph, store_dir)?;
    Ok(graph)
}

fn ordered_members(
    cfg: &WorkspaceConfig,
    repos: &std::collections::HashMap<String, Node>,
) -> Vec<RepositoryMember> {
    cfg.repositories
        .iter()
        .filter_map(|repo| {
            repos.get(&repo.name).map(|node| RepositoryMember {
                name: repo.name.clone(),
                node: node.address,
            })
        })
        .collect()
}

fn repo_nodes(cfg: &WorkspaceConfig) -> std::collections::HashMap<String, Node> {
    cfg.repositories
        .iter()
        .map(|repo| {
            let addr = address_of(repo.name.as_bytes());
            (
                repo.name.clone(),
                Node {
                    address: addr,
                    kind: NodeKind::Scope,
                    provenance: Provenance::tool("federation", Timestamp(1)),
                    confidence: 1.0,
                    created_at: Timestamp(1),
                    content_hash: addr,
                    span: None,
                },
            )
        })
        .collect()
}

fn load_repo_nodes(cfg: &WorkspaceConfig) -> Result<Vec<(String, Vec<Node>)>, String> {
    cfg.repositories
        .iter()
        .map(|repo| {
            let store = OnDiskStore::new(&repo.store_dir);
            let nodes = store
                .read_overlay(&OverlayKind::Custom("fused".into()))?
                .map(|(nodes, _)| nodes)
                .unwrap_or_default();
            Ok((repo.name.clone(), nodes))
        })
        .collect()
}

fn collect_edges(
    cfg: &WorkspaceConfig,
    repos: &std::collections::HashMap<String, Node>,
    repo_overlays: &[(String, Vec<Node>)],
) -> Vec<Edge> {
    let mut out = Vec::new();
    for (source, target, actor) in evidence::dependency_edges(cfg)
        .into_iter()
        .chain(evidence::identity_edges(repo_overlays))
    {
        if let (Some(src), Some(dst)) = (repos.get(&source), repos.get(&target)) {
            out.push(edge(src.address, dst.address, actor));
        }
    }
    out
}

fn edge(source: UorAddress, target: UorAddress, actor: String) -> Edge {
    Edge {
        source,
        target,
        relation: Relation::References,
        confidence: 1.0,
        provenance: Provenance::tool(actor, Timestamp(1)),
        evidence: vec![source, target],
    }
}

fn persist_workspace(graph: &WorkspaceGraph, store_dir: &std::path::Path) -> Result<(), String> {
    let store = OnDiskStore::new(store_dir);
    store.write_overlay(
        &OverlayKind::Federated,
        &graph.aggregate_nodes,
        &graph.federated_edges,
    )?;
    meta::save_workspace(
        store_dir,
        &WorkspaceMeta {
            workspace_id: graph.workspace_id.clone(),
            repositories: graph.repositories.iter().map(|r| r.name.clone()).collect(),
        },
    )
}
