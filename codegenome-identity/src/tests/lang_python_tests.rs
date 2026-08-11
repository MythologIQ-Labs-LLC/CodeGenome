use crate::lang::ir::*;
use crate::lang::python::PythonLanguage;
use crate::lang::LanguageSupport;

fn parse(code: &[u8]) -> tree_sitter::Tree {
    let mut parser = tree_sitter::Parser::new();
    parser.set_language(&PythonLanguage.language()).unwrap();
    parser.parse(code, None).unwrap()
}

#[test]
fn py_def_and_class_produce_symbol_defs() {
    let code = b"def greet():\n    pass\n\nclass Foo:\n    pass\n";
    let tree = parse(code);
    let symbols = PythonLanguage.extract_symbols(code, &tree);
    assert_eq!(symbols.len(), 2);
    assert_eq!(symbols[0].name, "greet");
    assert_eq!(symbols[0].kind, SymbolKind::Function);
    assert_eq!(symbols[1].name, "Foo");
    assert_eq!(symbols[1].kind, SymbolKind::Class);
}

#[test]
fn py_from_import_produces_import_ref() {
    let code = b"from os import path\n";
    let tree = parse(code);
    let imports = PythonLanguage.extract_imports(code, &tree);
    assert!(!imports.is_empty(), "Expected at least one ImportRef");
    let names: Vec<&str> = imports.iter().map(|i| i.imported_name.as_str()).collect();
    assert!(names.contains(&"path"), "Expected 'path' import");
}

#[test]
fn py_call_produces_call_ref() {
    let code = b"def main():\n    foo()\n";
    let tree = parse(code);
    let calls = PythonLanguage.extract_calls(code, &tree);
    assert!(!calls.is_empty(), "Expected at least one CallRef");
    assert_eq!(calls[0].callee_name, "foo");
}

#[test]
fn py_decorated_def_unwraps_to_function() {
    let code = b"@decorator\ndef bar():\n    pass\n";
    let tree = parse(code);
    let symbols = PythonLanguage.extract_symbols(code, &tree);
    assert_eq!(symbols.len(), 1);
    assert_eq!(symbols[0].name, "bar");
    assert_eq!(symbols[0].kind, SymbolKind::Function);
}

#[test]
fn py_class_with_base_produces_impl_ref() {
    let code = b"class Dog(Animal):\n    pass\n";
    let tree = parse(code);
    let impls = PythonLanguage.extract_impls(code, &tree);
    assert!(!impls.is_empty(), "Expected at least one ImplRef");
    assert_eq!(impls[0].type_name, "Dog");
    assert_eq!(impls[0].trait_name.as_deref(), Some("Animal"));
}

#[test]
fn py_if_else_produces_branch_edges() {
    let code = b"def f(x):\n    if x:\n        a = 1\n    else:\n        b = 2\n";
    let tree = parse(code);
    let edges = PythonLanguage.extract_control_flow(code, &tree);
    let branches: Vec<_> = edges.iter().filter(|e| e.kind == CfKind::Branch).collect();
    assert!(branches.len() >= 2, "Expected >=2 Branch edges");
}

#[test]
fn py_data_flow_extracts_def_use() {
    let code = b"def demo():\n    x = 1\n    y = x + 1\n";
    let tree = parse(code);
    let edges = PythonLanguage.extract_data_flow(code, &tree);
    assert!(!edges.is_empty(), "Expected data flow edge");
    assert_eq!(edges[0].var_name, "x");
}
