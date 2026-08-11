// Re-exports for backward compatibility.
// Extraction logic now lives in crate::index::flow_cfg.
pub use crate::index::flow_cfg::{extract_control_flow, node_span, CfgEdge, CfgKind, DfgEdge};
