use serde::{Deserialize, Serialize};

use crate::graph::node::Provenance;
use crate::identity::UorAddress;

/// An edge is a value. It connects two addresses with a
/// typed relationship. It does not know what overlay created
/// it.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Edge {
    pub source: UorAddress,
    pub target: UorAddress,
    pub relation: Relation,
    pub confidence: f64,
    pub provenance: Provenance,
    pub evidence: Vec<UorAddress>,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Relation {
    Contains,
    Calls,
    Imports,
    Extends,
    Implements,
    References,
    PartOfProcess,
    CoupledWithin,
    // Flow edges (from Joern CPG taxonomy)
    ControlFlow,
    ControlDependence,
    DataFlow,
    Dominates,
    // Reasoning edges (belief → graph artifact)
    AboutSubject,
    Supports,
    Contradicts,
    // Lineage edges (child → parent mutation)
    MutatedFrom,
}
