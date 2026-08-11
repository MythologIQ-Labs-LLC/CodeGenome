use serde::{Deserialize, Serialize};

/// A parameterized view over fixed content.
/// Content is immutable (identified by UOR); the frame
/// describes HOW to observe it (embedding model, analysis
/// tool, query perspective). Changing frames never changes
/// identity.
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ObserverFrame {
    pub name: String,
    pub version: String,
    pub parameters: Vec<(String, String)>,
}

impl ObserverFrame {
    pub fn new(name: impl Into<String>, version: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            version: version.into(),
            parameters: Vec::new(),
        }
    }

    pub fn with_param(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.parameters.push((key.into(), value.into()));
        self
    }
}
