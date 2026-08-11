use crate::lang::ir::*;
use crate::lang::rust::RustLanguage;
use crate::lang::LanguageSupport;

fn parse(code: &[u8]) -> tree_sitter::Tree {
    let mut parser = tree_sitter::Parser::new();
    parser.set_language(&RustLanguage.language()).unwrap();
    parser.parse(code, None).unwrap()
}

#[test]
fn three_functions_produce_three_symbol_defs() {
    let code = b"fn alpha() {}\nfn beta() {}\nfn gamma() {}";
    let tree = parse(code);
    let symbols = RustLanguage.extract_symbols(code, &tree);
    assert_eq!(symbols.len(), 3);
    assert_eq!(symbols[0].name, "alpha");
    assert_eq!(symbols[0].kind, SymbolKind::Function);
    assert_eq!(symbols[1].name, "beta");
    assert_eq!(symbols[2].name, "gamma");
}

#[test]
fn use_declaration_produces_import_ref() {
    let code = b"use crate::helper;";
    let tree = parse(code);
    let imports = RustLanguage.extract_imports(code, &tree);
    assert_eq!(imports.len(), 1);
    assert_eq!(imports[0].imported_name, "helper");
}

#[test]
fn function_call_produces_call_ref() {
    let code = b"fn helper() {}\nfn main() { helper(); }";
    let tree = parse(code);
    let calls = RustLanguage.extract_calls(code, &tree);
    assert!(!calls.is_empty(), "Expected at least one CallRef");
    assert_eq!(calls[0].callee_name, "helper");
}

#[test]
fn impl_trait_produces_impl_ref() {
    let code = b"trait Greet {}\nstruct Bot;\nimpl Greet for Bot {}";
    let tree = parse(code);
    let impls = RustLanguage.extract_impls(code, &tree);
    assert!(!impls.is_empty(), "Expected at least one ImplRef");
    assert_eq!(impls[0].type_name, "Bot");
    assert_eq!(impls[0].trait_name.as_deref(), Some("Greet"));
}

#[test]
fn control_flow_extracts_branch_edges() {
    let code = b"fn check(x: bool) { if x { let a = 1; } else { let b = 2; } }";
    let tree = parse(code);
    let edges = RustLanguage.extract_control_flow(code, &tree);
    let branches: Vec<_> = edges.iter().filter(|e| e.kind == CfKind::Branch).collect();
    assert!(
        branches.len() >= 2,
        "Expected >=2 Branch edges, got {}",
        branches.len()
    );
}

#[test]
fn data_flow_extracts_def_use() {
    let code = b"fn demo() { let x = 1; let y = x + 1; }";
    let tree = parse(code);
    let edges = RustLanguage.extract_data_flow(code, &tree);
    assert!(!edges.is_empty(), "Expected data flow edge");
    assert_eq!(edges[0].var_name, "x");
}
