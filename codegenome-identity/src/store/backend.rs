use crate::graph::edge::Edge;
use crate::graph::node::Node;
use crate::graph::overlay::OverlayKind;

/// Overlay data: nodes and edges as owned vectors.
pub type OverlayData = (Vec<Node>, Vec<Edge>);

/// Pluggable persistence. External systems implement this.
pub trait StoreBackend: Send + Sync {
    fn write_overlay(
        &self,
        kind: &OverlayKind,
        nodes: &[Node],
        edges: &[Edge],
    ) -> Result<(), String>;

    fn read_overlay(&self, kind: &OverlayKind) -> Result<Option<OverlayData>, String>;

    fn list_overlays(&self) -> Result<Vec<OverlayKind>, String>;
}
