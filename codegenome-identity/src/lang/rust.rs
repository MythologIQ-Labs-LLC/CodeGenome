use crate::graph::node::Span;
use crate::lang::ir::*;
use crate::lang::rust_flow;
use crate::lang::LanguageSupport;

pub struct RustLanguage;

impl LanguageSupport for RustLanguage {
    fn name(&self) -> &str {
        "rust"
    }
    fn extensions(&self) -> &[&str] {
        &["rs"]
    }

    fn language(&self) -> tree_sitter::Language {
        tree_sitter_rust::LANGUAGE.into()
    }

    fn extract_symbols(&self, source: &[u8], tree: &tree_sitter::Tree) -> Vec<SymbolDef> {
        extract_rust_symbols(source, tree)
    }

    fn extract_imports(&self, source: &[u8], tree: &tree_sitter::Tree) -> Vec<ImportRef> {
        extract_rust_imports(source, tree)
    }

    fn extract_calls(&self, source: &[u8], tree: &tree_sitter::Tree) -> Vec<CallRef> {
        extract_rust_calls(source, tree)
    }

    fn extract_impls(&self, source: &[u8], tree: &tree_sitter::Tree) -> Vec<ImplRef> {
        extract_rust_impls(source, tree)
    }

    fn extract_control_flow(&self, source: &[u8], tree: &tree_sitter::Tree) -> Vec<CfEdge> {
        rust_flow::extract_control_flow(source, tree)
    }

    fn extract_data_flow(&self, source: &[u8], tree: &tree_sitter::Tree) -> Vec<DfEdge> {
        rust_flow::extract_data_flow(source, tree)
    }
}

const SYMBOL_KINDS: &[&str] = &[
    "function_item",
    "struct_item",
    "enum_item",
    "impl_item",
    "use_declaration",
    "mod_item",
    "trait_item",
];

fn extract_rust_symbols(source: &[u8], tree: &tree_sitter::Tree) -> Vec<SymbolDef> {
    let mut symbols = Vec::new();
    let root = tree.root_node();
    let mut cursor = root.walk();
    for child in root.children(&mut cursor) {
        if !SYMBOL_KINDS.contains(&child.kind()) {
            continue;
        }
        let name = child
            .child_by_field_name("name")
            .and_then(|n| n.utf8_text(source).ok())
            .unwrap_or(child.kind())
            .to_string();
        let kind = match child.kind() {
            "function_item" => SymbolKind::Function,
            "struct_item" => SymbolKind::Class,
            "enum_item" => SymbolKind::Enum,
            "trait_item" => SymbolKind::Trait,
            "mod_item" => SymbolKind::Module,
            other => SymbolKind::Other(other.into()),
        };
        symbols.push(make_symbol(
            name,
            kind,
            node_span(&child),
            child.kind().to_string(),
        ));
    }
    symbols
}

fn extract_rust_imports(source: &[u8], tree: &tree_sitter::Tree) -> Vec<ImportRef> {
    let mut imports = Vec::new();
    let root = tree.root_node();
    let mut cursor = root.walk();
    for child in root.children(&mut cursor) {
        if child.kind() != "use_declaration" {
            continue;
        }
        if let Some(name) = use_leaf_name(&child, source) {
            imports.push(ImportRef {
                imported_name: name,
                span: node_span(&child),
            });
        }
    }
    imports
}

fn extract_rust_calls(source: &[u8], tree: &tree_sitter::Tree) -> Vec<CallRef> {
    let mut calls = Vec::new();
    let root = tree.root_node();
    let mut cursor = root.walk();
    for child in root.children(&mut cursor) {
        if child.kind() != "function_item" {
            continue;
        }
        let fn_span = node_span(&child);
        collect_calls(&child, source, fn_span, &mut calls);
    }
    calls
}

fn extract_rust_impls(source: &[u8], tree: &tree_sitter::Tree) -> Vec<ImplRef> {
    let mut impls = Vec::new();
    let root = tree.root_node();
    let mut cursor = root.walk();
    for child in root.children(&mut cursor) {
        if child.kind() != "impl_item" {
            continue;
        }
        if let Some(imp) = parse_impl_item(&child, source) {
            impls.push(imp);
        }
    }
    impls
}

fn use_leaf_name(node: &tree_sitter::Node, source: &[u8]) -> Option<String> {
    let mut best: Option<String> = None;
    walk_for_ident(node, source, &mut best);
    best
}

fn walk_for_ident(node: &tree_sitter::Node, source: &[u8], best: &mut Option<String>) {
    if node.kind() == "identifier" || node.kind() == "type_identifier" {
        if let Ok(text) = node.utf8_text(source) {
            *best = Some(text.to_string());
        }
    }
    let mut c = node.walk();
    for child in node.children(&mut c) {
        walk_for_ident(&child, source, best);
    }
}

fn collect_calls(node: &tree_sitter::Node, source: &[u8], fn_span: Span, calls: &mut Vec<CallRef>) {
    if node.kind() == "call_expression" {
        if let Some(callee) = call_callee_name(node, source) {
            calls.push(CallRef {
                caller_span: fn_span,
                callee_name: callee,
                span: node_span(node),
            });
        }
    }
    let mut c = node.walk();
    for child in node.children(&mut c) {
        collect_calls(&child, source, fn_span, calls);
    }
}

fn call_callee_name(node: &tree_sitter::Node, source: &[u8]) -> Option<String> {
    let func = node.child_by_field_name("function")?;
    match func.kind() {
        "identifier" => func.utf8_text(source).ok().map(String::from),
        "scoped_identifier" | "field_expression" => last_identifier(&func, source),
        _ => func.utf8_text(source).ok().map(String::from),
    }
}

fn last_identifier(node: &tree_sitter::Node, source: &[u8]) -> Option<String> {
    let mut c = node.walk();
    let mut last = None;
    for child in node.children(&mut c) {
        if child.kind() == "identifier" || child.kind() == "type_identifier" {
            last = child.utf8_text(source).ok().map(String::from);
        }
    }
    last
}

fn parse_impl_item(node: &tree_sitter::Node, source: &[u8]) -> Option<ImplRef> {
    let type_node = node.child_by_field_name("type")?;
    let type_name = type_text(&type_node, source)?;
    let trait_name = node
        .child_by_field_name("trait")
        .and_then(|t| type_text(&t, source));
    Some(ImplRef {
        type_name,
        trait_name,
        span: node_span(node),
    })
}

fn type_text(node: &tree_sitter::Node, source: &[u8]) -> Option<String> {
    match node.kind() {
        "type_identifier" | "identifier" => node.utf8_text(source).ok().map(String::from),
        _ => {
            last_identifier(node, source).or_else(|| node.utf8_text(source).ok().map(String::from))
        }
    }
}
