use crate::graph::node::Span;
use crate::lang::ir::*;
use crate::lang::python_flow;
use crate::lang::LanguageSupport;

pub struct PythonLanguage;

impl LanguageSupport for PythonLanguage {
    fn name(&self) -> &str {
        "python"
    }
    fn extensions(&self) -> &[&str] {
        &["py"]
    }
    fn language(&self) -> tree_sitter::Language {
        tree_sitter_python::LANGUAGE.into()
    }
    fn extract_symbols(&self, source: &[u8], tree: &tree_sitter::Tree) -> Vec<SymbolDef> {
        let mut out = Vec::new();
        walk_py_symbols(&tree.root_node(), source, &mut out);
        out
    }
    fn extract_imports(&self, source: &[u8], tree: &tree_sitter::Tree) -> Vec<ImportRef> {
        let mut out = Vec::new();
        let root = tree.root_node();
        let mut c = root.walk();
        for child in root.children(&mut c) {
            match child.kind() {
                "import_statement" => collect_dotted(&child, source, &mut out),
                "import_from_statement" => collect_from(&child, source, &mut out),
                _ => {}
            }
        }
        out
    }
    fn extract_calls(&self, source: &[u8], tree: &tree_sitter::Tree) -> Vec<CallRef> {
        let mut out = Vec::new();
        walk_py_calls(&tree.root_node(), source, Span::default(), &mut out);
        out
    }
    fn extract_impls(&self, source: &[u8], tree: &tree_sitter::Tree) -> Vec<ImplRef> {
        let mut out = Vec::new();
        walk_py_class_bases(&tree.root_node(), source, &mut out);
        out
    }
    fn extract_control_flow(&self, source: &[u8], tree: &tree_sitter::Tree) -> Vec<CfEdge> {
        python_flow::extract_control_flow(source, tree)
    }
    fn extract_data_flow(&self, source: &[u8], tree: &tree_sitter::Tree) -> Vec<DfEdge> {
        python_flow::extract_data_flow(source, tree)
    }
}

fn walk_py_symbols(node: &tree_sitter::Node, source: &[u8], symbols: &mut Vec<SymbolDef>) {
    match node.kind() {
        "function_definition" => {
            if let Some(name) = node_name(node, source) {
                symbols.push(make_symbol(
                    name,
                    SymbolKind::Function,
                    node_span(node),
                    "function_definition".into(),
                ));
            }
        }
        "class_definition" => {
            if let Some(name) = node_name(node, source) {
                symbols.push(make_symbol(
                    name,
                    SymbolKind::Class,
                    node_span(node),
                    "class_definition".into(),
                ));
            }
        }
        "decorated_definition" => {
            let mut c = node.walk();
            for child in node.children(&mut c) {
                if child.kind() == "function_definition" || child.kind() == "class_definition" {
                    walk_py_symbols(&child, source, symbols);
                }
            }
            return;
        }
        _ => {}
    }
    let mut c = node.walk();
    for child in node.children(&mut c) {
        walk_py_symbols(&child, source, symbols);
    }
}

fn collect_dotted(node: &tree_sitter::Node, source: &[u8], imports: &mut Vec<ImportRef>) {
    let mut c = node.walk();
    for child in node.children(&mut c) {
        if child.kind() == "dotted_name" {
            if let Ok(name) = child.utf8_text(source) {
                imports.push(ImportRef {
                    imported_name: name.to_string(),
                    span: node_span(&child),
                });
            }
        }
    }
}

fn collect_from(node: &tree_sitter::Node, source: &[u8], imports: &mut Vec<ImportRef>) {
    let mut c = node.walk();
    for child in node.children(&mut c) {
        if child.kind() == "dotted_name" || child.kind() == "identifier" {
            if let Ok(name) = child.utf8_text(source) {
                imports.push(ImportRef {
                    imported_name: name.to_string(),
                    span: node_span(&child),
                });
            }
        }
    }
}

fn walk_py_calls(node: &tree_sitter::Node, source: &[u8], fn_span: Span, calls: &mut Vec<CallRef>) {
    let current_span = if node.kind() == "function_definition" {
        node_span(node)
    } else {
        fn_span
    };
    if node.kind() == "call" {
        if let Some(callee) = py_call_name(node, source) {
            calls.push(CallRef {
                caller_span: current_span,
                callee_name: callee,
                span: node_span(node),
            });
        }
    }
    let mut c = node.walk();
    for child in node.children(&mut c) {
        walk_py_calls(&child, source, current_span, calls);
    }
}

fn py_call_name(node: &tree_sitter::Node, source: &[u8]) -> Option<String> {
    let func = node.child_by_field_name("function")?;
    match func.kind() {
        "identifier" => func.utf8_text(source).ok().map(String::from),
        "attribute" => func
            .child_by_field_name("attribute")
            .and_then(|a| a.utf8_text(source).ok().map(String::from)),
        _ => func.utf8_text(source).ok().map(String::from),
    }
}

fn walk_py_class_bases(node: &tree_sitter::Node, source: &[u8], impls: &mut Vec<ImplRef>) {
    if node.kind() == "class_definition" {
        let type_name = node_name(node, source).unwrap_or_default();
        if let Some(args) = node.child_by_field_name("superclasses") {
            let mut c = args.walk();
            for child in args.children(&mut c) {
                if child.kind() == "identifier" {
                    if let Ok(base) = child.utf8_text(source) {
                        impls.push(ImplRef {
                            type_name: type_name.clone(),
                            trait_name: Some(base.to_string()),
                            span: node_span(node),
                        });
                    }
                }
            }
        }
    }
    let mut c = node.walk();
    for child in node.children(&mut c) {
        walk_py_class_bases(&child, source, impls);
    }
}

fn node_name(node: &tree_sitter::Node, source: &[u8]) -> Option<String> {
    node.child_by_field_name("name")
        .and_then(|n| n.utf8_text(source).ok())
        .map(String::from)
}
