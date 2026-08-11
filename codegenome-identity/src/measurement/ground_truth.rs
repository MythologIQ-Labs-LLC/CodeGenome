use serde::{Deserialize, Serialize};

/// How available is ground truth for a given property?
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum GroundTruthLevel {
    /// Oracle exists (e.g., Tree-sitter AST for syntax).
    Available,
    /// Can be built via experiment (e.g., make a change, observe).
    Constructible,
    /// No oracle — hypothesis only (e.g., compaction fidelity).
    Unsolved,
}

/// How does failure manifest?
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum FailureMode {
    /// Wrong but undetectable without oracle.
    Silent,
    /// Wrong and detectable via invariant violation.
    Observable,
    /// Wrong and cascades to other components.
    Catastrophic,
}

/// A detected violation of a measured property.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Violation {
    pub property: String,
    pub description: String,
    pub failure_mode: FailureMode,
}

impl Violation {
    pub fn new(
        property: impl Into<String>,
        description: impl Into<String>,
        failure_mode: FailureMode,
    ) -> Self {
        Self {
            property: property.into(),
            description: description.into(),
            failure_mode,
        }
    }
}
