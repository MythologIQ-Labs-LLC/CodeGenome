use crate::graph::node::Span;

#[derive(Clone, Debug)]
pub struct Entrypoint {
    pub name: String,
    pub kind: EntrypointKind,
    pub span: Span,
}

#[derive(Clone, Debug, PartialEq)]
pub enum EntrypointKind {
    Main,
    Test,
    PublicApi,
}

/// Extract entrypoints from top-level function items.
pub fn extract_entrypoints(source: &[u8], tree: &tree_sitter::Tree) -> Vec<Entrypoint> {
    let mut entries = Vec::new();
    let root = tree.root_node();
    let mut cursor = root.walk();

    for child in root.children(&mut cursor) {
        if child.kind() != "function_item" {
            continue;
        }
        let name = fn_name(&child, source);
        let span = node_span(&child);

        if name == "main" {
            entries.push(Entrypoint {
                name,
                kind: EntrypointKind::Main,
                span,
            });
        } else if has_test_attr(&child, source) {
            entries.push(Entrypoint {
                name,
                kind: EntrypointKind::Test,
                span,
            });
        } else if has_pub_visibility(&child) {
            entries.push(Entrypoint {
                name,
                kind: EntrypointKind::PublicApi,
                span,
            });
        }
    }
    entries
}

fn fn_name(node: &tree_sitter::Node, source: &[u8]) -> String {
    node.child_by_field_name("name")
        .and_then(|n| n.utf8_text(source).ok())
        .unwrap_or("unknown")
        .to_string()
}

fn has_test_attr(node: &tree_sitter::Node, source: &[u8]) -> bool {
    // Check children of function_item
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        if child.kind() == "attribute_item" {
            if let Ok(text) = child.utf8_text(source) {
                if text.contains("test") {
                    return true;
                }
            }
        }
    }
    // Check preceding sibling (attribute_item before function_item)
    if let Some(prev) = node.prev_sibling() {
        if prev.kind() == "attribute_item" {
            if let Ok(text) = prev.utf8_text(source) {
                if text.contains("test") {
                    return true;
                }
            }
        }
    }
    false
}

fn has_pub_visibility(node: &tree_sitter::Node) -> bool {
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        if child.kind() == "visibility_modifier" {
            return true;
        }
    }
    false
}

fn node_span(node: &tree_sitter::Node) -> Span {
    Span {
        start_byte: node.start_byte() as u32,
        end_byte: node.end_byte() as u32,
        start_line: node.start_position().row as u32 + 1,
        end_line: node.end_position().row as u32 + 1,
    }
}
