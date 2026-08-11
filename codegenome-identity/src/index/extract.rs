// Re-exports for backward compatibility.
// Extraction logic now lives in crate::lang::rust.
//
// The old types (UseTarget, CallSite, ImplTarget) map to
// the language-neutral IR types (ImportRef, CallRef, ImplRef).

pub use crate::lang::ir::{
    node_span, CallRef as CallSite, ImplRef as ImplTarget, ImportRef as UseTarget,
};

// Re-export extraction functions from the Rust backend.
// These create a RustLanguage instance and delegate.
use crate::lang::rust::RustLanguage;
use crate::lang::LanguageSupport;

pub fn extract_use_targets(source: &[u8], tree: &tree_sitter::Tree) -> Vec<UseTarget> {
    RustLanguage
        .extract_imports(source, tree)
        .into_iter()
        .map(|r| UseTarget {
            imported_name: r.imported_name,
            span: r.span,
        })
        .collect()
}

pub fn extract_call_sites(source: &[u8], tree: &tree_sitter::Tree) -> Vec<CallSite> {
    RustLanguage.extract_calls(source, tree)
}

pub fn extract_impl_targets(source: &[u8], tree: &tree_sitter::Tree) -> Vec<ImplTarget> {
    RustLanguage.extract_impls(source, tree)
}
