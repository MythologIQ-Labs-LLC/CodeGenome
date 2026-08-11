use std::future::Future;

use crate::tools::inputs::*;
use crate::tools::CodegenomeTools;
use rmcp::handler::server::ServerHandler;
use rmcp::model::*;
use rmcp::service::RequestContext;
use rmcp::{ErrorData as McpError, RoleServer, ServiceExt};
use schemars::JsonSchema;

/// Start the MCP server on stdio.
pub async fn run_stdio(
    source_dir: String,
    store_dir: String,
) -> Result<(), Box<dyn std::error::Error>> {
    let handler = CodegenomeTools::new(source_dir, store_dir);
    let transport = rmcp::transport::stdio();
    let service = handler.serve(transport).await?;
    service.waiting().await?;
    Ok(())
}

impl ServerHandler for CodegenomeTools {
    fn list_tools(
        &self,
        _request: Option<PaginatedRequestParams>,
        _context: RequestContext<RoleServer>,
    ) -> impl Future<Output = Result<ListToolsResult, McpError>> + Send + '_ {
        let tools = vec![
            typed_tool::<ContextInput>(
                "codegenome_context",
                "Retrieve context around a symbol via graph traversal",
            ),
            typed_tool::<ImpactInput>("codegenome_impact", "Blast radius from a symbol change"),
            typed_tool::<DetectInput>(
                "codegenome_detect_changes",
                "Map git diff to affected symbols and impact",
            ),
            typed_tool::<TraceInput>("codegenome_trace", "Trace call chain from entrypoint"),
            typed_tool::<ReindexInput>(
                "codegenome_reindex",
                "Write-gated re-index of source files",
            ),
            typed_tool::<StatusInput>("codegenome_status", "Index status and freshness report"),
            typed_tool::<ExperimentStartInput>(
                "codegenome_experiment_start",
                "Start async experiment loop",
            ),
            typed_tool::<ExperimentStatusInput>(
                "codegenome_experiment_status",
                "Poll experiment progress",
            ),
            typed_tool::<ExperimentResultsInput>(
                "codegenome_experiment_results",
                "Read last N experiment results",
            ),
            typed_tool::<WorkspaceTraceInput>(
                "codegenome_workspace_trace",
                "Trace cross-repo workspace paths",
            ),
            typed_tool::<AssertInput>(
                "codegenome_assert",
                "Write-gated: assert a belief about a code artifact",
            ),
        ];
        std::future::ready(Ok(ListToolsResult::with_all_items(tools)
            .with_ttl_ms(3_600_000)
            .with_cache_scope(CacheScope::Public)))
    }

    fn call_tool(
        &self,
        request: CallToolRequestParams,
        _context: RequestContext<RoleServer>,
    ) -> impl Future<Output = Result<CallToolResponse, McpError>> + Send + '_ {
        let result = dispatch_tool(self, &request).map(Into::into);
        std::future::ready(result)
    }
}

pub(crate) fn dispatch_tool(
    tools: &CodegenomeTools,
    req: &CallToolRequestParams,
) -> Result<CallToolResult, McpError> {
    let text = match req.name.as_ref() {
        "codegenome_context" => {
            let input: ContextInput = deser(req)?;
            tools.context(&input)
        }
        "codegenome_impact" => {
            let input: ImpactInput = deser(req)?;
            tools.impact(&input)
        }
        "codegenome_detect_changes" => {
            let input: DetectInput = deser(req)?;
            tools.detect(&input)
        }
        "codegenome_trace" => {
            let input: TraceInput = deser(req)?;
            tools.trace(&input)
        }
        "codegenome_reindex" => {
            let input: ReindexInput = deser(req)?;
            tools.reindex(&input)
        }
        "codegenome_status" => {
            let input: StatusInput = deser(req)?;
            tools.status_report(&input.source_dir)
        }
        "codegenome_experiment_start" => {
            let input: ExperimentStartInput = deser(req)?;
            tools.experiment_start(&input.source_dir, input.max_iterations as u64)
        }
        "codegenome_experiment_status" => tools.experiment_status(),
        "codegenome_experiment_results" => {
            let input: ExperimentResultsInput = deser(req)?;
            let n = if input.last_n == 0 {
                10
            } else {
                input.last_n as usize
            };
            tools.experiment_results(n)
        }
        "codegenome_workspace_trace" => {
            let input: WorkspaceTraceInput = deser(req)?;
            tools.workspace_trace(&input.workspace_dir, &input.from_repo, &input.to_repo)
        }
        "codegenome_assert" => {
            let input: AssertInput = deser(req)?;
            tools.assert_belief(&input)
        }
        _ => {
            return Err(McpError::invalid_params(
                format!("unknown tool: {}", req.name),
                None,
            ))
        }
    };
    Ok(CallToolResult::success(vec![ContentBlock::text(text)]))
}

fn deser<T: serde::de::DeserializeOwned>(req: &CallToolRequestParams) -> Result<T, McpError> {
    let args = req
        .arguments
        .as_ref()
        .map(|a| serde_json::Value::Object(a.clone()))
        .unwrap_or(serde_json::Value::Object(Default::default()));
    serde_json::from_value(args).map_err(|e| {
        McpError::invalid_params(
            format!("invalid arguments for tool {}: {e}", req.name),
            None,
        )
    })
}

fn typed_tool<T: JsonSchema>(name: &'static str, desc: &'static str) -> Tool {
    let schema = schemars::schema_for!(T);
    let schema_map = match serde_json::to_value(schema) {
        Ok(serde_json::Value::Object(m)) => m,
        _ => serde_json::Map::new(),
    };
    Tool::new(name, desc, schema_map)
}
