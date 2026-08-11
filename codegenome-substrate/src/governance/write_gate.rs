use crate::governance::policy::Decision;
use codegenome_identity::store::meta::FreshnessReport;

/// Policy configuration for write privilege evaluation.
pub struct WriteGatePolicy {
    pub confidence_floor: f64,
    pub require_provenance: bool,
    pub require_freshness: bool,
}

/// A request to write to the canonical graph.
pub struct WriteRequest {
    pub actor: String,
    pub toolchain_version: String,
    pub source_freshness: FreshnessReport,
    pub min_edge_confidence: f64,
}

impl WriteGatePolicy {
    /// Default policy: require provenance + freshness,
    /// confidence floor at 0.5.
    pub fn default_policy() -> Self {
        Self {
            confidence_floor: 0.5,
            require_provenance: true,
            require_freshness: true,
        }
    }

    /// Evaluate a write request against this policy.
    pub fn evaluate(&self, request: &WriteRequest) -> Decision {
        if self.require_provenance && request.actor.is_empty() {
            return Decision::Deny("missing provenance: actor must be identified".into());
        }

        if self.require_freshness && !request.source_freshness.is_fresh {
            return Decision::Deny(format!(
                "index stale: {} files changed, {} added, {} removed — reindex first",
                request.source_freshness.files_changed,
                request.source_freshness.files_added,
                request.source_freshness.files_removed,
            ));
        }

        if request.min_edge_confidence < self.confidence_floor {
            return Decision::Deny(format!(
                "confidence {:.2} below floor {:.2}",
                request.min_edge_confidence, self.confidence_floor,
            ));
        }

        Decision::Allow
    }
}
