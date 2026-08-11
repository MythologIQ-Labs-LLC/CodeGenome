// Re-exports for backward compatibility.
// CFG extraction logic now lives in crate::lang::rust_flow.

pub use crate::lang::ir::{node_span, CfEdge as CfgEdge, CfKind as CfgKind, DfEdge as DfgEdge};
pub use crate::lang::rust_flow::extract_control_flow;
