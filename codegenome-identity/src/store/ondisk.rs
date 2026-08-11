use std::fs;
use std::path::{Path, PathBuf};

use crate::graph::edge::Edge;
use crate::graph::node::Node;
use crate::graph::overlay::OverlayKind;
use crate::store::backend::StoreBackend;

/// On-disk store: one directory per OverlayKind.
/// Bincode serialization.
pub struct OnDiskStore {
    root: PathBuf,
}

impl OnDiskStore {
    pub fn new(root: impl Into<PathBuf>) -> Self {
        Self { root: root.into() }
    }

    pub fn base_dir(&self) -> &Path {
        &self.root
    }

    fn overlay_dir(&self, kind: &OverlayKind) -> PathBuf {
        self.root.join(overlay_dir_name(kind))
    }
}

impl StoreBackend for OnDiskStore {
    fn write_overlay(
        &self,
        kind: &OverlayKind,
        nodes: &[Node],
        edges: &[Edge],
    ) -> Result<(), String> {
        let dir = self.overlay_dir(kind);
        fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
        write_bincode(&dir.join("nodes.bin"), nodes)?;
        write_bincode(&dir.join("edges.bin"), edges)?;
        Ok(())
    }

    fn read_overlay(&self, kind: &OverlayKind) -> Result<Option<(Vec<Node>, Vec<Edge>)>, String> {
        let dir = self.overlay_dir(kind);
        if !dir.exists() {
            return Ok(None);
        }
        let nodes = read_bincode(&dir.join("nodes.bin"))?;
        let edges = read_bincode(&dir.join("edges.bin"))?;
        Ok(Some((nodes, edges)))
    }

    fn list_overlays(&self) -> Result<Vec<OverlayKind>, String> {
        if !self.root.exists() {
            return Ok(vec![]);
        }
        let mut kinds = Vec::new();
        let entries = fs::read_dir(&self.root).map_err(|e| e.to_string())?;
        for entry in entries.flatten() {
            if entry.path().is_dir() {
                let name = entry.file_name().to_string_lossy().into_owned();
                kinds.push(dir_name_to_kind(&name));
            }
        }
        Ok(kinds)
    }
}

fn overlay_dir_name(kind: &OverlayKind) -> String {
    match kind {
        OverlayKind::Syntax => "syntax".into(),
        OverlayKind::Semantic => "semantic".into(),
        OverlayKind::Flow => "flow".into(),
        OverlayKind::Runtime => "runtime".into(),
        OverlayKind::Federated => "federated".into(),
        OverlayKind::Custom(s) => format!("custom_{s}"),
    }
}

fn dir_name_to_kind(name: &str) -> OverlayKind {
    match name {
        "syntax" => OverlayKind::Syntax,
        "semantic" => OverlayKind::Semantic,
        "flow" => OverlayKind::Flow,
        "runtime" => OverlayKind::Runtime,
        "federated" => OverlayKind::Federated,
        s => OverlayKind::Custom(s.strip_prefix("custom_").unwrap_or(s).into()),
    }
}

fn write_bincode<T: serde::Serialize + ?Sized>(path: &Path, data: &T) -> Result<(), String> {
    let bytes = crate::codec::to_vec(data).map_err(|e| e.to_string())?;
    fs::write(path, bytes).map_err(|e| e.to_string())
}

fn read_bincode<T: serde::de::DeserializeOwned>(path: &Path) -> Result<T, String> {
    let bytes = fs::read(path).map_err(|e| e.to_string())?;
    crate::codec::from_slice(&bytes).map_err(|e| e.to_string())
}
