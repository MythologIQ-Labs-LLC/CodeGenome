use serde::{Deserialize, Serialize};

use crate::graph::edge::{Edge, Relation};
use crate::graph::node::Node;
use crate::identity::UorAddress;

/// Declarative query description. Describes WHAT to find,
/// not HOW to find it. Execution is separate.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Query {
    pub target: UorAddress,
    pub direction: Direction,
    pub max_depth: u32,
    pub min_confidence: f64,
    pub relation_filter: Option<Vec<Relation>>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Direction {
    Upstream,
    Downstream,
    Both,
}

/// Result of a query. Pure value — no handles, no state.
#[derive(Clone, Debug, Default)]
pub struct QueryResult {
    pub nodes: Vec<Node>,
    pub edges: Vec<Edge>,
    pub paths: Vec<Vec<UorAddress>>,
    pub confidence: f64,
}

impl Query {
    pub fn downstream(target: UorAddress, max_depth: u32) -> Self {
        Self {
            target,
            direction: Direction::Downstream,
            max_depth,
            min_confidence: 0.0,
            relation_filter: None,
        }
    }
}
