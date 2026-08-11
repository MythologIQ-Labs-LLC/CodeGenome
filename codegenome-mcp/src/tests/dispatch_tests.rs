use rmcp::model::CallToolRequestParams;

use crate::server::dispatch_tool;
use crate::tools::CodegenomeTools;

fn test_tools() -> CodegenomeTools {
    let store = std::env::temp_dir().join("codegenome-dispatch-tests");
    CodegenomeTools::new(".".to_string(), store.to_string_lossy().into_owned())
}

fn request(name: &'static str, arguments: Option<serde_json::Value>) -> CallToolRequestParams {
    CallToolRequestParams {
        meta: None,
        name: name.into(),
        arguments: arguments.and_then(|v| match v {
            serde_json::Value::Object(m) => Some(m),
            _ => None,
        }),
        task: None,
    }
}

#[test]
fn malformed_args_return_error_not_panic() {
    let tools = test_tools();
    // `line` is required by ContextInput but has the wrong type here.
    let req = request(
        "codegenome_context",
        Some(serde_json::json!({"file": "src/lib.rs", "line": "not-a-number"})),
    );
    let result = dispatch_tool(&tools, &req);
    let err = result.expect_err("malformed arguments must yield an error");
    assert!(
        err.message.contains("invalid arguments"),
        "error should identify invalid arguments, got: {}",
        err.message
    );
}

#[test]
fn missing_required_args_return_error() {
    let tools = test_tools();
    let req = request("codegenome_context", None);
    assert!(dispatch_tool(&tools, &req).is_err());
}

#[test]
fn unknown_tool_returns_error() {
    let tools = test_tools();
    let req = request("codegenome_nonexistent", Some(serde_json::json!({})));
    let err = dispatch_tool(&tools, &req).expect_err("unknown tool must yield an error");
    assert!(err.message.contains("unknown tool"));
}
