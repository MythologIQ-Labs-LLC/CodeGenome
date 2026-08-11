use crate::lang::ir::*;
use crate::lang::typescript::TypeScriptLanguage;
use crate::lang::LanguageSupport;

fn parse_ts(code: &[u8]) -> tree_sitter::Tree {
    let lang = TypeScriptLanguage::ts();
    let mut parser = tree_sitter::Parser::new();
    parser.set_language(&lang.language()).unwrap();
    parser.parse(code, None).unwrap()
}

fn parse_tsx(code: &[u8]) -> tree_sitter::Tree {
    let lang = TypeScriptLanguage::tsx();
    let mut parser = tree_sitter::Parser::new();
    parser.set_language(&lang.language()).unwrap();
    parser.parse(code, None).unwrap()
}

#[test]
fn ts_function_and_class_produce_symbol_defs() {
    let code = b"function greet() {}\nclass Foo {}";
    let tree = parse_ts(code);
    let lang = TypeScriptLanguage::ts();
    let symbols = lang.extract_symbols(code, &tree);
    assert_eq!(symbols.len(), 2);
    assert_eq!(symbols[0].name, "greet");
    assert_eq!(symbols[0].kind, SymbolKind::Function);
    assert_eq!(symbols[1].name, "Foo");
    assert_eq!(symbols[1].kind, SymbolKind::Class);
}

#[test]
fn ts_import_produces_import_ref() {
    let code = b"import { x } from 'y';";
    let tree = parse_ts(code);
    let lang = TypeScriptLanguage::ts();
    let imports = lang.extract_imports(code, &tree);
    assert!(!imports.is_empty(), "Expected at least one ImportRef");
    assert_eq!(imports[0].imported_name, "x");
}

#[test]
fn tsx_component_produces_symbol_def() {
    let code = b"function App() { return <div/>; }";
    let tree = parse_tsx(code);
    let lang = TypeScriptLanguage::tsx();
    let symbols = lang.extract_symbols(code, &tree);
    assert_eq!(symbols.len(), 1);
    assert_eq!(symbols[0].name, "App");
    assert_eq!(symbols[0].kind, SymbolKind::Function);
}

#[test]
fn ts_interface_produces_trait_symbol() {
    let code = b"interface Bar { name: string; }";
    let tree = parse_ts(code);
    let lang = TypeScriptLanguage::ts();
    let symbols = lang.extract_symbols(code, &tree);
    assert_eq!(symbols.len(), 1);
    assert_eq!(symbols[0].name, "Bar");
    assert_eq!(symbols[0].kind, SymbolKind::Trait);
}

#[test]
fn ts_call_produces_call_ref() {
    let code = b"function main() { greet(); }";
    let tree = parse_ts(code);
    let lang = TypeScriptLanguage::ts();
    let calls = lang.extract_calls(code, &tree);
    assert!(!calls.is_empty(), "Expected at least one CallRef");
    assert_eq!(calls[0].callee_name, "greet");
}

#[test]
fn ts_class_extends_produces_impl_ref() {
    let code = b"class Dog extends Animal {}";
    let tree = parse_ts(code);
    let lang = TypeScriptLanguage::ts();
    let impls = lang.extract_impls(code, &tree);
    assert!(!impls.is_empty(), "Expected at least one ImplRef");
    assert_eq!(impls[0].type_name, "Dog");
    assert_eq!(impls[0].trait_name.as_deref(), Some("Animal"));
}

#[test]
fn ts_if_else_produces_branch_edges() {
    let code = b"function f(x: boolean) { if (x) { 1; } else { 2; } }";
    let tree = parse_ts(code);
    let lang = TypeScriptLanguage::ts();
    let edges = lang.extract_control_flow(code, &tree);
    let branches: Vec<_> = edges.iter().filter(|e| e.kind == CfKind::Branch).collect();
    assert!(branches.len() >= 2, "Expected >=2 Branch edges");
}

#[test]
fn ts_data_flow_extracts_def_use() {
    let code = b"function f() { const x = 1; const y = x + 1; }";
    let tree = parse_ts(code);
    let lang = TypeScriptLanguage::ts();
    let edges = lang.extract_data_flow(code, &tree);
    assert!(!edges.is_empty(), "Expected data flow edge");
    assert_eq!(edges[0].var_name, "x");
}
