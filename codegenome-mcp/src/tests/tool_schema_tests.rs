use schemars::schema_for;

use crate::tools::inputs::*;

/// schemars 1.x exposes schemas as JSON values; assert on the
/// serialized form, which is also exactly what MCP clients see.
fn required_fields(schema: &schemars::Schema) -> Vec<String> {
    let value = serde_json::to_value(schema).unwrap();
    value["required"]
        .as_array()
        .map(|a| {
            a.iter()
                .filter_map(|v| v.as_str().map(String::from))
                .collect()
        })
        .unwrap_or_default()
}

#[test]
fn context_input_schema_valid() {
    let schema = schema_for!(ContextInput);
    let required = required_fields(&schema);
    assert!(
        required.contains(&"file".to_string()),
        "file should be required"
    );
    assert!(
        required.contains(&"line".to_string()),
        "line should be required"
    );
}

#[test]
fn impact_input_schema_valid() {
    let schema = schema_for!(ImpactInput);
    let required = required_fields(&schema);
    assert!(required.contains(&"file".to_string()));
    assert!(required.contains(&"line".to_string()));
}

#[test]
fn detect_input_defaults_from_ref() {
    let input: DetectInput = serde_json::from_str("{}").unwrap();
    assert_eq!(input.from_ref, "HEAD");
    assert!(input.to_ref.is_none());
}

#[test]
fn reindex_input_defaults_actor() {
    let input: ReindexInput = serde_json::from_str("{}").unwrap();
    assert_eq!(input.actor, "claude-code");
}

#[test]
fn all_schemas_produce_valid_json() {
    // These should not panic
    let _ = schema_for!(ContextInput);
    let _ = schema_for!(ImpactInput);
    let _ = schema_for!(DetectInput);
    let _ = schema_for!(ReindexInput);
    let _ = schema_for!(TraceInput);
    let _ = schema_for!(StatusInput);
    let _ = schema_for!(ExperimentStartInput);
    let _ = schema_for!(ExperimentResultsInput);
    let _ = schema_for!(WorkspaceTraceInput);
}
