pub mod canonical;
pub mod detect;
pub mod graph_builder;
pub mod ir;
pub mod python;
pub mod python_flow;
pub mod rust;
pub mod rust_flow;
pub mod typescript;
pub mod typescript_flow;

use ir::*;

/// A language backend extracts IR from parsed tree-sitter trees.
/// Shared pipeline orchestration calls these methods; each language
/// only implements what's actually different.
pub trait LanguageSupport: Send + Sync {
    fn name(&self) -> &str;
    fn extensions(&self) -> &[&str];
    fn language(&self) -> tree_sitter::Language;

    fn extract_symbols(&self, source: &[u8], tree: &tree_sitter::Tree) -> Vec<SymbolDef>;
    fn extract_imports(&self, source: &[u8], tree: &tree_sitter::Tree) -> Vec<ImportRef>;
    fn extract_calls(&self, source: &[u8], tree: &tree_sitter::Tree) -> Vec<CallRef>;
    fn extract_impls(&self, source: &[u8], tree: &tree_sitter::Tree) -> Vec<ImplRef>;
    fn extract_control_flow(&self, source: &[u8], tree: &tree_sitter::Tree) -> Vec<CfEdge>;
    fn extract_data_flow(&self, source: &[u8], tree: &tree_sitter::Tree) -> Vec<DfEdge>;
}

/// All built-in language backends.
pub fn all_languages() -> Vec<Box<dyn LanguageSupport>> {
    vec![
        Box::new(rust::RustLanguage),
        Box::new(typescript::TypeScriptLanguage::ts()),
        Box::new(typescript::TypeScriptLanguage::tsx()),
        Box::new(python::PythonLanguage),
    ]
}
