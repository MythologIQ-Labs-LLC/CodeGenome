use crate::graph::node::Span;
use crate::lang::ir::*;
use crate::lang::typescript_flow;
use crate::lang::LanguageSupport;

pub struct TypeScriptLanguage {
    tsx: bool,
}

impl TypeScriptLanguage {
    pub fn ts() -> Self {
        Self { tsx: false }
    }
    pub fn tsx() -> Self {
        Self { tsx: true }
    }
}

impl LanguageSupport for TypeScriptLanguage {
    fn name(&self) -> &str {
        if self.tsx {
            "typescript-tsx"
        } else {
            "typescript"
        }
    }
    fn extensions(&self) -> &[&str] {
        if self.tsx {
            &["tsx"]
        } else {
            &["ts"]
        }
    }
    fn language(&self) -> tree_sitter::Language {
        if self.tsx {
            tree_sitter_typescript::LANGUAGE_TSX.into()
        } else {
            tree_sitter_typescript::LANGUAGE_TYPESCRIPT.into()
        }
    }
    fn extract_symbols(&self, source: &[u8], tree: &tree_sitter::Tree) -> Vec<SymbolDef> {
        let mut out = Vec::new();
        walk_for_symbols(&tree.root_node(), source, &mut out);
        out
    }
    fn extract_imports(&self, source: &[u8], tree: &tree_sitter::Tree) -> Vec<ImportRef> {
        let mut out = Vec::new();
        let root = tree.root_node();
        let mut c = root.walk();
        for child in root.children(&mut c) {
            if child.kind() == "import_statement" {
                collect_import_names(&child, source, &mut out);
            }
        }
        out
    }
    fn extract_calls(&self, source: &[u8], tree: &tree_sitter::Tree) -> Vec<CallRef> {
        let mut out = Vec::new();
        walk_for_calls(&tree.root_node(), source, Span::default(), &mut out);
        out
    }
    fn extract_impls(&self, source: &[u8], tree: &tree_sitter::Tree) -> Vec<ImplRef> {
        let mut out = Vec::new();
        walk_for_class_heritage(&tree.root_node(), source, &mut out);
        out
    }
    fn extract_control_flow(&self, source: &[u8], tree: &tree_sitter::Tree) -> Vec<CfEdge> {
        typescript_flow::extract_control_flow(source, tree)
    }
    fn extract_data_flow(&self, source: &[u8], tree: &tree_sitter::Tree) -> Vec<DfEdge> {
        typescript_flow::extract_data_flow(source, tree)
    }
}

const SYMBOL_KINDS: &[&str] = &[
    "function_declaration",
    "class_declaration",
    "interface_declaration",
    "enum_declaration",
    "type_alias_declaration",
];

fn walk_for_symbols(node: &tree_sitter::Node, source: &[u8], symbols: &mut Vec<SymbolDef>) {
    if SYMBOL_KINDS.contains(&node.kind()) {
        let name = node
            .child_by_field_name("name")
            .and_then(|n| n.utf8_text(source).ok())
            .unwrap_or(node.kind())
            .to_string();
        let kind = match node.kind() {
            "function_declaration" => SymbolKind::Function,
            "class_declaration" => SymbolKind::Class,
            "interface_declaration" => SymbolKind::Trait,
            "enum_declaration" => SymbolKind::Enum,
            _ => SymbolKind::Other(node.kind().into()),
        };
        symbols.push(make_symbol(
            name,
            kind,
            node_span(node),
            node.kind().to_string(),
        ));
    }
    let mut c = node.walk();
    for child in node.children(&mut c) {
        walk_for_symbols(&child, source, symbols);
    }
}

fn collect_import_names(node: &tree_sitter::Node, source: &[u8], imports: &mut Vec<ImportRef>) {
    if node.kind() == "identifier" {
        if let Ok(name) = node.utf8_text(source) {
            imports.push(ImportRef {
                imported_name: name.to_string(),
                span: node_span(node),
            });
            return;
        }
    }
    if node.kind() == "import_specifier" {
        let name_node = node
            .child_by_field_name("alias")
            .or_else(|| node.child_by_field_name("name"));
        if let Some(n) = name_node {
            if let Ok(name) = n.utf8_text(source) {
                imports.push(ImportRef {
                    imported_name: name.to_string(),
                    span: node_span(node),
                });
            }
        }
        return;
    }
    let mut c = node.walk();
    for child in node.children(&mut c) {
        collect_import_names(&child, source, imports);
    }
}

fn walk_for_calls(
    node: &tree_sitter::Node,
    source: &[u8],
    fn_span: Span,
    calls: &mut Vec<CallRef>,
) {
    let current_span = match node.kind() {
        "function_declaration" | "arrow_function" | "method_definition" => node_span(node),
        _ => fn_span,
    };
    if node.kind() == "call_expression" {
        if let Some(callee) = ts_call_name(node, source) {
            calls.push(CallRef {
                caller_span: current_span,
                callee_name: callee,
                span: node_span(node),
            });
        }
    }
    let mut c = node.walk();
    for child in node.children(&mut c) {
        walk_for_calls(&child, source, current_span, calls);
    }
}

fn ts_call_name(node: &tree_sitter::Node, source: &[u8]) -> Option<String> {
    let func = node.child_by_field_name("function")?;
    match func.kind() {
        "identifier" => func.utf8_text(source).ok().map(String::from),
        "member_expression" => func
            .child_by_field_name("property")
            .and_then(|p| p.utf8_text(source).ok().map(String::from)),
        _ => func.utf8_text(source).ok().map(String::from),
    }
}

fn walk_for_class_heritage(node: &tree_sitter::Node, source: &[u8], impls: &mut Vec<ImplRef>) {
    if node.kind() == "class_declaration" {
        let type_name = node
            .child_by_field_name("name")
            .and_then(|n| n.utf8_text(source).ok())
            .unwrap_or("")
            .to_string();
        let mut c = node.walk();
        for child in node.children(&mut c) {
            if child.kind() == "class_heritage" {
                if let Some(parent) = heritage_name(&child, source) {
                    impls.push(ImplRef {
                        type_name: type_name.clone(),
                        trait_name: Some(parent),
                        span: node_span(node),
                    });
                }
            }
        }
    }
    let mut c = node.walk();
    for child in node.children(&mut c) {
        walk_for_class_heritage(&child, source, impls);
    }
}

fn heritage_name(node: &tree_sitter::Node, source: &[u8]) -> Option<String> {
    let mut c = node.walk();
    for child in node.children(&mut c) {
        match child.kind() {
            "identifier" | "type_identifier" => {
                return child.utf8_text(source).ok().map(String::from);
            }
            "extends_clause" | "implements_clause" => {
                if let Some(name) = heritage_name(&child, source) {
                    return Some(name);
                }
            }
            _ => {}
        }
    }
    None
}
