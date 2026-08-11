use std::path::Path;

use serde::Deserialize;

#[derive(Clone, Debug, PartialEq)]
pub enum Decision {
    Allow,
    Deny(String),
    RequireApproval(String),
}

pub struct PolicyContext {
    pub operation: String,
    pub impact_nodes: usize,
    pub changed_files: usize,
}

#[derive(Deserialize)]
struct PolicyFile {
    #[serde(default)]
    rules: Vec<RuleConfig>,
}

#[derive(Deserialize, Clone)]
struct RuleConfig {
    operation: String,
    condition: String,
    action: String,
}

pub struct PolicyEngine {
    rules: Vec<RuleConfig>,
}

impl PolicyEngine {
    pub fn load(path: &Path) -> Result<Self, String> {
        if !path.exists() {
            return Ok(Self { rules: Vec::new() });
        }
        let content = std::fs::read_to_string(path).map_err(|e| e.to_string())?;
        let file: PolicyFile = toml::from_str(&content).map_err(|e| e.to_string())?;
        Ok(Self { rules: file.rules })
    }

    pub fn evaluate(&self, context: &PolicyContext) -> Decision {
        for rule in &self.rules {
            if rule.operation != context.operation {
                continue;
            }
            if !condition_matches(&rule.condition, context) {
                continue;
            }
            return match rule.action.as_str() {
                "deny" => Decision::Deny(rule.condition.clone()),
                "require-approval" => Decision::RequireApproval(rule.condition.clone()),
                _ => Decision::Allow,
            };
        }
        Decision::Allow
    }
}

fn condition_matches(condition: &str, ctx: &PolicyContext) -> bool {
    if condition == "always" {
        return true;
    }
    let parts: Vec<&str> = condition.split_whitespace().collect();
    if parts.len() != 3 {
        return false;
    }
    let field = parts[0];
    let op = parts[1];
    let Ok(threshold) = parts[2].parse::<usize>() else {
        return false;
    };
    let value = match field {
        "impact_nodes" => ctx.impact_nodes,
        "changed_files" => ctx.changed_files,
        _ => return false,
    };
    match op {
        ">" => value > threshold,
        ">=" => value >= threshold,
        "<" => value < threshold,
        "==" => value == threshold,
        _ => false,
    }
}
