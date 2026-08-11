use schemars::JsonSchema;
use serde::Deserialize;

fn default_depth() -> u32 {
    1
}
fn default_direction() -> String {
    "downstream".into()
}
fn default_head() -> String {
    "HEAD".into()
}
fn default_actor() -> String {
    "claude-code".into()
}

#[derive(Deserialize, JsonSchema)]
pub struct ContextInput {
    /// File path relative to source root
    pub file: String,
    /// Line number
    pub line: u32,
    /// Traversal depth (default: 1)
    #[serde(default = "default_depth")]
    pub depth: u32,
    /// Direction: "downstream", "upstream", or "both"
    #[serde(default = "default_direction")]
    pub direction: String,
}

#[derive(Deserialize, JsonSchema)]
pub struct ImpactInput {
    /// File path relative to source root
    pub file: String,
    /// Line number
    pub line: u32,
}

#[derive(Deserialize, JsonSchema)]
pub struct DetectInput {
    /// Git ref to diff from (default: HEAD)
    #[serde(default = "default_head")]
    pub from_ref: String,
    /// Git ref to diff to (default: working tree)
    pub to_ref: Option<String>,
}

#[derive(Deserialize, JsonSchema)]
pub struct ReindexInput {
    /// Actor name for provenance (default: "claude-code")
    #[serde(default = "default_actor")]
    pub actor: String,
}

#[derive(Deserialize, JsonSchema)]
pub struct TraceInput {
    /// Entrypoint function or process name
    pub entrypoint: String,
    /// Max traversal depth (default: 1)
    #[serde(default = "default_depth")]
    pub max_depth: u32,
}

#[derive(Deserialize, JsonSchema)]
pub struct StatusInput {
    /// Source directory to check freshness against (default: the
    /// directory the server was started with)
    #[serde(default)]
    pub source_dir: String,
}

#[derive(Deserialize, JsonSchema)]
pub struct ExperimentStatusInput {}

#[derive(Deserialize, JsonSchema)]
pub struct ExperimentStartInput {
    /// Source directory to index
    pub source_dir: String,
    /// Maximum iterations
    pub max_iterations: u32,
}

#[derive(Deserialize, JsonSchema)]
pub struct ExperimentResultsInput {
    /// Number of recent results to return
    #[serde(default = "default_depth")]
    pub last_n: u32,
}

#[derive(Deserialize, JsonSchema)]
pub struct AssertInput {
    /// The claim text (content-addressed for identity)
    pub claim: String,
    /// File path of the subject artifact
    pub subject_file: String,
    /// Line number of the subject artifact
    pub subject_line: u32,
    /// Confidence in the claim (0.0 to 1.0)
    pub confidence: f64,
    /// Actor name for provenance
    #[serde(default = "default_actor")]
    pub actor: String,
}

#[derive(Deserialize, JsonSchema)]
pub struct WorkspaceTraceInput {
    /// Workspace root directory
    pub workspace_dir: String,
    /// Source repository name
    pub from_repo: String,
    /// Target repository name
    pub to_repo: String,
}
