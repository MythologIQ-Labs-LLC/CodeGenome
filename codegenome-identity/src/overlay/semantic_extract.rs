// Re-exports for backward compatibility.
// Extraction logic now lives in crate::index::extract.
pub use crate::index::extract::{
    extract_call_sites, extract_impl_targets, extract_use_targets, node_span, CallSite, ImplTarget,
    UseTarget,
};
