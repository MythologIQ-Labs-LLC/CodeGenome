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

// --- Recursive extraction (Shadow Genome-era gap: top-level-only walks) ---

#[test]
fn impl_methods_are_extracted_as_symbols() {
    let code = b"struct S;\nimpl S {\n    fn method_a(&self) {}\n    fn method_b(&self) {}\n}";
    let tree = parse(code);
    let symbols = RustLanguage.extract_symbols(code, &tree);
    let names: Vec<_> = symbols.iter().map(|s| s.name.as_str()).collect();
    assert!(names.contains(&"method_a"), "impl methods must be symbols");
    assert!(names.contains(&"method_b"), "impl methods must be symbols");
}

#[test]
fn symbols_inside_inline_modules_are_extracted() {
    let code = b"mod inner {\n    pub fn hidden() {}\n    pub struct Deep;\n}";
    let tree = parse(code);
    let symbols = RustLanguage.extract_symbols(code, &tree);
    let names: Vec<_> = symbols.iter().map(|s| s.name.as_str()).collect();
    assert!(
        names.contains(&"hidden"),
        "fns inside mod blocks must be symbols"
    );
    assert!(
        names.contains(&"Deep"),
        "structs inside mod blocks must be symbols"
    );
}

#[test]
fn impl_methods_produce_call_refs() {
    let code = b"fn helper() {}\nstruct S;\nimpl S {\n    fn method(&self) { helper(); }\n}";
    let tree = parse(code);
    let calls = RustLanguage.extract_calls(code, &tree);
    assert!(
        calls.iter().any(|c| c.callee_name == "helper"),
        "calls made inside impl methods must produce CallRefs"
    );
}

#[test]
fn nested_function_calls_attributed_to_innermost_fn() {
    let code = b"fn helper() {}\nfn outer() {\n    fn inner() { helper(); }\n    inner();\n}";
    let tree = parse(code);
    let calls = RustLanguage.extract_calls(code, &tree);
    let helper_call = calls
        .iter()
        .find(|c| c.callee_name == "helper")
        .expect("helper call extracted");
    let symbols = RustLanguage.extract_symbols(code, &tree);
    let inner = symbols.iter().find(|s| s.name == "inner").unwrap();
    assert_eq!(
        helper_call.caller_span.start_line, inner.span.start_line,
        "call inside nested fn must carry the nested fn's span, not the outer's"
    );
}

#[test]
fn imports_inside_functions_are_extracted() {
    let code = b"fn f() {\n    use std::collections::HashMap;\n    let _m: HashMap<u8, u8> = HashMap::new();\n}";
    let tree = parse(code);
    let imports = RustLanguage.extract_imports(code, &tree);
    assert!(
        imports.iter().any(|i| i.imported_name == "HashMap"),
        "use declarations inside fn bodies must be extracted"
    );
}

// --- Symbol identity: file-scoped addresses ---

#[test]
fn same_name_symbols_in_different_files_get_distinct_addresses() {
    use crate::lang::graph_builder::symbol_address;
    use std::path::Path;
    let a = symbol_address(Path::new("src/commands/index.rs"), "function_item", "run");
    let b = symbol_address(Path::new("src/commands/query.rs"), "function_item", "run");
    assert_ne!(
        a, b,
        "same-named symbols in different files must not share an identity"
    );
}
