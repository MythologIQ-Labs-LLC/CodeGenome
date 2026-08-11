use crate::lang::ir::*;

/// Extract control flow edges within Rust function bodies.
pub fn extract_control_flow(source: &[u8], tree: &tree_sitter::Tree) -> Vec<CfEdge> {
    let mut edges = Vec::new();
    let root = tree.root_node();
    let mut cursor = root.walk();
    for child in root.children(&mut cursor) {
        if child.kind() == "function_item" {
            if let Some(body) = child.child_by_field_name("body") {
                extract_block_flow(&body, source, &mut edges);
            }
        }
    }
    edges
}

/// Extract data flow edges within Rust function bodies.
pub fn extract_data_flow(source: &[u8], tree: &tree_sitter::Tree) -> Vec<DfEdge> {
    let mut edges = Vec::new();
    let root = tree.root_node();
    let mut cursor = root.walk();
    for child in root.children(&mut cursor) {
        if child.kind() == "function_item" {
            if let Some(body) = child.child_by_field_name("body") {
                extract_fn_data_flow(&body, source, &mut edges);
            }
        }
    }
    edges
}

fn extract_block_flow(block: &tree_sitter::Node, source: &[u8], edges: &mut Vec<CfEdge>) {
    let stmts = block_statements(block);
    for pair in stmts.windows(2) {
        edges.push(CfEdge {
            source_span: node_span(&pair[0]),
            target_span: node_span(&pair[1]),
            kind: CfKind::Sequential,
        });
    }
    for stmt in &stmts {
        extract_control_structure(stmt, source, edges);
    }
}

fn extract_control_structure(node: &tree_sitter::Node, source: &[u8], edges: &mut Vec<CfEdge>) {
    match node.kind() {
        "if_expression" => extract_if_flow(node, source, edges),
        "match_expression" => extract_match_flow(node, edges),
        "loop_expression" | "while_expression" | "for_expression" => {
            extract_loop_flow(node, source, edges);
        }
        "return_expression" => {
            edges.push(CfEdge {
                source_span: node_span(node),
                target_span: node_span(node),
                kind: CfKind::Return,
            });
        }
        "expression_statement" => {
            if let Some(inner) = node.child(0) {
                extract_control_structure(&inner, source, edges);
            }
        }
        _ => {}
    }
}

fn extract_if_flow(node: &tree_sitter::Node, source: &[u8], edges: &mut Vec<CfEdge>) {
    let if_span = node_span(node);
    if let Some(cons) = node.child_by_field_name("consequence") {
        edges.push(CfEdge {
            source_span: if_span,
            target_span: node_span(&cons),
            kind: CfKind::Branch,
        });
        extract_block_flow(&cons, source, edges);
    }
    if let Some(alt) = node.child_by_field_name("alternative") {
        edges.push(CfEdge {
            source_span: if_span,
            target_span: node_span(&alt),
            kind: CfKind::Branch,
        });
        if alt.kind() == "else_clause" {
            let mut c = alt.walk();
            for child in alt.children(&mut c) {
                if child.kind() == "block" {
                    extract_block_flow(&child, source, edges);
                } else if child.kind() == "if_expression" {
                    extract_if_flow(&child, source, edges);
                }
            }
        }
    }
}

fn extract_match_flow(node: &tree_sitter::Node, edges: &mut Vec<CfEdge>) {
    let span = node_span(node);
    if let Some(body) = node.child_by_field_name("body") {
        let mut c = body.walk();
        for child in body.children(&mut c) {
            if child.kind() == "match_arm" {
                edges.push(CfEdge {
                    source_span: span,
                    target_span: node_span(&child),
                    kind: CfKind::Branch,
                });
            }
        }
    }
}

fn extract_loop_flow(node: &tree_sitter::Node, source: &[u8], edges: &mut Vec<CfEdge>) {
    let loop_span = node_span(node);
    if let Some(body) = node.child_by_field_name("body") {
        edges.push(CfEdge {
            source_span: node_span(&body),
            target_span: loop_span,
            kind: CfKind::BackEdge,
        });
        extract_block_flow(&body, source, edges);
    }
}

fn extract_fn_data_flow(body: &tree_sitter::Node, source: &[u8], edges: &mut Vec<DfEdge>) {
    let mut defs: Vec<(String, crate::graph::node::Span)> = Vec::new();
    collect_let_defs(body, source, &mut defs);
    let mut uses: Vec<(String, crate::graph::node::Span)> = Vec::new();
    collect_ident_uses(body, source, &mut uses);

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
}

fn collect_let_defs(
    node: &tree_sitter::Node,
    source: &[u8],
    defs: &mut Vec<(String, crate::graph::node::Span)>,
) {
    if node.kind() == "let_declaration" {
        if let Some(pat) = node.child_by_field_name("pattern") {
            if let Ok(name) = pat.utf8_text(source) {
                defs.push((name.to_string(), node_span(&pat)));
            }
        }
    }
    let mut c = node.walk();
    for child in node.children(&mut c) {
        collect_let_defs(&child, source, defs);
    }
}

fn collect_ident_uses(
    node: &tree_sitter::Node,
    source: &[u8],
    uses: &mut Vec<(String, crate::graph::node::Span)>,
) {
    if node.kind() == "identifier" && !is_def_site(node) {
        if let Ok(name) = node.utf8_text(source) {
            uses.push((name.to_string(), node_span(node)));
        }
    }
    let mut c = node.walk();
    for child in node.children(&mut c) {
        collect_ident_uses(&child, source, uses);
    }
}

fn is_def_site(node: &tree_sitter::Node) -> bool {
    node.parent()
        .map(|p| p.kind() == "let_declaration" && p.child_by_field_name("pattern") == Some(*node))
        .unwrap_or(false)
}

fn block_statements<'a>(block: &'a tree_sitter::Node<'a>) -> Vec<tree_sitter::Node<'a>> {
    let mut stmts = Vec::new();
    let mut c = block.walk();
    for child in block.children(&mut c) {
        match child.kind() {
            "{" | "}" => continue,
            _ => stmts.push(child),
        }
    }
    stmts
}
