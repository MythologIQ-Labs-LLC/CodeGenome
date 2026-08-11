use schemars::schema_for;

use crate::tools::inputs::*;

#[test]
fn context_input_schema_valid() {
    let schema = schema_for!(ContextInput);
    let obj = schema.schema.object.as_ref().unwrap();
    assert!(
        obj.required.contains("file"),
        "file should be required"
    );
    assert!(
        obj.required.contains("line"),
        "line should be required"
    );
}

#[test]
fn impact_input_schema_valid() {
    let schema = schema_for!(ImpactInput);
    let obj = schema.schema.object.as_ref().unwrap();
    assert!(obj.required.contains("file"));
    assert!(obj.required.contains("line"));
}

#[test]
fn detect_input_defaults_from_ref() {
    let input: DetectInput =
        serde_json::from_str("{}").unwrap();
    assert_eq!(input.from_ref, "HEAD");
    assert!(input.to_ref.is_none());
}

#[test]
fn reindex_input_defaults_actor() {
    let input: ReindexInput =
        serde_json::from_str("{}").unwrap();
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
