use crate::graph::edge::Relation;
use crate::graph::node::{NodeKind, Span};
use crate::lang::graph_builder::build_file_graph;
use crate::lang::ir::*;

fn sp(line: u32, start: u32, end: u32) -> Span {
    Span {
        start_byte: start,
        end_byte: end,
        start_line: line,
        end_line: line,
    }
}

#[test]
fn graph_builder_produces_nodes_and_contains_edges() {
    let source = b"fn alpha() {}\nfn beta() {}\n";
    let symbols = vec![
        make_symbol(
            "alpha".into(),
            SymbolKind::Function,
            sp(1, 0, 13),
            "function_item".into(),
        ),
        make_symbol(
            "beta".into(),
            SymbolKind::Function,
            sp(2, 14, 26),
            "function_item".into(),
        ),
    ];

    let (nodes, edges) = build_file_graph(
        std::path::Path::new("test.rs"),
        source,
        "rust",
        &symbols,
        &[],
        &[],
        &[],
    );

    assert_eq!(nodes.len(), 3);
    let file_nodes: Vec<_> = nodes.iter().filter(|n| n.kind == NodeKind::File).collect();
    assert_eq!(file_nodes.len(), 1);

    let contains: Vec<_> = edges
        .iter()
        .filter(|e| e.relation == Relation::Contains)
        .collect();
    assert_eq!(contains.len(), 2);
}

#[test]
fn graph_builder_produces_calls_edge() {
    let source = b"fn caller() { callee(); }\nfn callee() {}";
    let symbols = vec![
        make_symbol(
            "caller".into(),
            SymbolKind::Function,
            sp(1, 0, 25),
            "function_item".into(),
        ),
        make_symbol(
            "callee".into(),
            SymbolKind::Function,
            sp(2, 26, 40),
            "function_item".into(),
        ),
    ];
    let calls = vec![CallRef {
        caller_span: sp(1, 0, 25),
        callee_name: "callee".into(),
        span: sp(1, 14, 23),
    }];

    let (_, edges) = build_file_graph(
        std::path::Path::new("test.rs"),
        source,
        "rust",
        &symbols,
        &[],
        &calls,
        &[],
    );

    let call_edges: Vec<_> = edges
        .iter()
        .filter(|e| e.relation == Relation::Calls)
        .collect();
    assert_eq!(call_edges.len(), 1);
    assert!((call_edges[0].confidence - 0.7).abs() < f64::EPSILON);
}
