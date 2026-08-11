pub mod assert_belief;
pub mod context;
pub mod detect;
pub mod experiment_tool;
pub mod gate;
pub mod impact;
pub mod inputs;
pub mod reindex;
pub mod status_tool;
pub mod trace;
pub mod workspace_trace;

use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use codegenome_identity::graph::edge::Edge;
use codegenome_identity::graph::node::Node;
use codegenome_identity::graph::overlay::{Overlay, OverlayKind};
use codegenome_identity::graph::resolve::FileIndex;
use codegenome_identity::measurement::GroundTruthLevel;
use codegenome_identity::store::backend::StoreBackend;
use codegenome_identity::store::ondisk::OnDiskStore;

/// Shared server state.
pub struct CodegenomeTools {
    pub source_dir: PathBuf,
    pub store_dir: PathBuf,
    pub run_manager: Arc<Mutex<RunManager>>,
}

impl Clone for CodegenomeTools {
    fn clone(&self) -> Self {
        Self {
            source_dir: self.source_dir.clone(),
            store_dir: self.store_dir.clone(),
            run_manager: Arc::clone(&self.run_manager),
        }
    }
}

pub struct RunManager {
    pub active_run: Option<RunHandle>,
}

pub struct RunHandle {
    pub run_id: String,
    pub max_iterations: u64,
    pub log_path: PathBuf,
    pub completed: Arc<std::sync::atomic::AtomicBool>,
}

impl CodegenomeTools {
    pub fn new(source_dir: impl Into<PathBuf>, store_dir: impl Into<PathBuf>) -> Self {
        Self {
            source_dir: source_dir.into(),
            store_dir: store_dir.into(),
            run_manager: Arc::new(Mutex::new(RunManager { active_run: None })),
        }
    }

    pub fn load_overlay(&self) -> Option<StoredOverlay> {
        let store = OnDiskStore::new(&self.store_dir);
        let (mut nodes, mut edges) = store
            .read_overlay(&OverlayKind::Custom("fused".into()))
            .ok()??;

        // Merge persisted beliefs into query context
        let (belief_nodes, belief_edges) =
            codegenome_substrate::belief::store::try_load_beliefs(&store);
        nodes.extend(belief_nodes);
        edges.extend(belief_edges);

        Some(StoredOverlay { nodes, edges })
    }

    /// Load fused overlay and build a FileIndex for resolution.
    pub fn load_with_index(&self) -> Option<(StoredOverlay, FileIndex)> {
        let overlay = self.load_overlay()?;
        let index = FileIndex::build(&self.source_dir, overlay.nodes(), overlay.edges());
        Some((overlay, index))
    }

    pub fn load_federated_overlay(&self, store_dir: Option<&str>) -> Option<StoredOverlay> {
        let root = store_dir
            .filter(|s| !s.is_empty())
            .map(PathBuf::from)
            .unwrap_or_else(|| self.store_dir.clone());
        let store = OnDiskStore::new(root);
        let (nodes, edges) = store.read_overlay(&OverlayKind::Federated).ok()??;
        Some(StoredOverlay { nodes, edges })
    }

    /// Build provenance metadata for tool responses. The toolchain
    /// string is derived from the registered language backends rather
    /// than hardcoded, so provenance stays truthful as backends change.
    pub fn response_meta(&self) -> serde_json::Value {
        let freshness =
            codegenome_identity::store::meta::check_freshness(&self.store_dir, &self.source_dir);
        let langs: Vec<String> = codegenome_identity::lang::all_languages()
            .iter()
            .map(|l| l.name().to_string())
            .collect();
        serde_json::json!({
            "source_fresh": freshness.is_fresh,
            "last_indexed": freshness.last_indexed,
            "toolchain": format!("tree-sitter({}) + heuristic-resolver", langs.join(",")),
        })
    }
}

/// Wrapper to make stored data implement Overlay trait.
pub struct StoredOverlay {
    pub nodes: Vec<Node>,
    pub edges: Vec<Edge>,
}

impl Overlay for StoredOverlay {
    fn kind(&self) -> OverlayKind {
        OverlayKind::Custom("stored".into())
    }
    fn nodes(&self) -> &[Node] {
        &self.nodes
    }
    fn edges(&self) -> &[Edge] {
        &self.edges
    }
    fn ground_truth(&self) -> GroundTruthLevel {
        GroundTruthLevel::Constructible
    }
}
