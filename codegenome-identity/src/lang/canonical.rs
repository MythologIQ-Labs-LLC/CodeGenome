use serde::{Deserialize, Serialize};

use crate::identity::{address_of, UorAddress};
use crate::lang::ir::SymbolKind;

/// Language-neutral canonical kind for cross-language comparison.
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CanonicalKind {
    Function,
    Class,
    Interface,
    Enum,
    Module,
    TypeAlias,
    Other,
}

/// Map a language-specific source_kind to canonical kind.
pub fn canonicalize(kind: &SymbolKind, source_kind: &str) -> CanonicalKind {
    match source_kind {
        "function_item" | "function_declaration" | "function_definition" => CanonicalKind::Function,
        "struct_item" | "class_declaration" | "class_definition" => CanonicalKind::Class,
        "trait_item" | "interface_declaration" => CanonicalKind::Interface,
        "enum_item" | "enum_declaration" => CanonicalKind::Enum,
        "mod_item" => CanonicalKind::Module,
        "type_alias_declaration" => CanonicalKind::TypeAlias,
        _ => match kind {
            SymbolKind::Function => CanonicalKind::Function,
            SymbolKind::Class => CanonicalKind::Class,
            SymbolKind::Trait => CanonicalKind::Interface,
            SymbolKind::Enum => CanonicalKind::Enum,
            SymbolKind::Module => CanonicalKind::Module,
            SymbolKind::Other(_) => CanonicalKind::Other,
        },
    }
}

/// Compute a normalized address from canonical kind + symbol name.
/// Same canonical kind + same name → same address, regardless
/// of source language.
pub fn normalized_address(canonical: &CanonicalKind, name: &str) -> UorAddress {
    let content = format!("canonical:{canonical:?}:{name}");
    address_of(content.as_bytes())
}
