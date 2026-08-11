use crate::graph::node::Span;
use crate::lang::ir::*;

/// Extract control flow edges from TypeScript/TSX source.
pub fn extract_control_flow(source: &[u8], tree: &tree_sitter::Tree) -> Vec<CfEdge> {
    let mut edges = Vec::new();
    let root = tree.root_node();
    walk_ts_flow(&root, source, &mut edges);
    edges
}

fn walk_ts_flow(node: &tree_sitter::Node, _source: &[u8], edges: &mut Vec<CfEdge>) {
    match node.kind() {
        "if_statement" => {
            let span = node_span(node);
            if let Some(cons) = node.child_by_field_name("consequence") {
                edges.push(CfEdge {
                    source_span: span,
                    target_span: node_span(&cons),
                    kind: CfKind::Branch,
                });
            }
            if let Some(alt) = node.child_by_field_name("alternative") {
                edges.push(CfEdge {
                    source_span: span,
                    target_span: node_span(&alt),
                    kind: CfKind::Branch,
                });
            }
        }
        "for_statement" | "for_in_statement" | "while_statement" => {
            if let Some(body) = node.child_by_field_name("body") {
                edges.push(CfEdge {
                    source_span: node_span(&body),
                    target_span: node_span(node),
                    kind: CfKind::BackEdge,
                });
            }
        }
        "switch_statement" => {
            let span = node_span(node);
            if let Some(body) = node.child_by_field_name("body") {
                let mut c = body.walk();
                for child in body.children(&mut c) {
                    if child.kind() == "switch_case" || child.kind() == "switch_default" {
                        edges.push(CfEdge {
                            source_span: span,
                            target_span: node_span(&child),
                            kind: CfKind::Branch,
                        });
                    }
                }
            }
        }
        "return_statement" => {
            edges.push(CfEdge {
                source_span: node_span(node),
                target_span: node_span(node),
                kind: CfKind::Return,
            });
        }
        _ => {}
    }
    let mut c = node.walk();
    for child in node.children(&mut c) {
        walk_ts_flow(&child, _source, edges);
    }
}

/// Extract data flow edges from TypeScript/TSX source.
pub fn extract_data_flow(source: &[u8], tree: &tree_sitter::Tree) -> Vec<DfEdge> {
    let mut edges = Vec::new();
    let root = tree.root_node();
    let mut defs: Vec<(String, Span)> = Vec::new();
    let mut uses: Vec<(String, Span)> = Vec::new();
    collect_ts_defs(&root, source, &mut defs);
    collect_ts_uses(&root, source, &mut uses);

    for (use_name, use_span) in &uses {
        for (def_name, def_span) in &defs {
            if use_name == def_name && use_span.start_byte > def_span.end_byte {
                edges.push(DfEdge {
                    def_span: *def_span,
                    use_span: *use_span,
                    var_name: use_name.clone(),
                });
                break;
            }
        }
    }
    edges
}

fn collect_ts_defs(node: &tree_sitter::Node, source: &[u8], defs: &mut Vec<(String, Span)>) {
    if node.kind() == "variable_declarator" {
        if let Some(name_node) = node.child_by_field_name("name") {
            if let Ok(name) = name_node.utf8_text(source) {
                defs.push((name.to_string(), node_span(&name_node)));
            }
        }
    }
    let mut c = node.walk();
    for child in node.children(&mut c) {
        collect_ts_defs(&child, source, defs);
    }
}

fn collect_ts_uses(node: &tree_sitter::Node, source: &[u8], uses: &mut Vec<(String, Span)>) {
    if node.kind() == "identifier" && !is_ts_def_site(node) {
        if let Ok(name) = node.utf8_text(source) {
            uses.push((name.to_string(), node_span(node)));
        }
    }
    let mut c = node.walk();
    for child in node.children(&mut c) {
        collect_ts_uses(&child, source, uses);
    }
}

fn is_ts_def_site(node: &tree_sitter::Node) -> bool {
    node.parent()
        .map(|p| p.kind() == "variable_declarator" && p.child_by_field_name("name") == Some(*node))
        .unwrap_or(false)
}
