use crate::graph::Edge;

/// Path confidence: product of edge confidences along a path.
/// Empty path = 1.0 (identity element of multiplication).
pub fn path_confidence(edges: &[&Edge]) -> f64 {
    edges.iter().map(|e| e.confidence).product()
}

/// Multi-path confidence: noisy-OR across independent paths.
/// P(reach) = 1 - product(1 - c(p)) for each path p.
/// Empty paths = 0.0 (no path means no reachability).
pub fn multi_path_confidence(path_confidences: &[f64]) -> f64 {
    if path_confidences.is_empty() {
        return 0.0;
    }
    1.0 - path_confidences
        .iter()
        .map(|&c| 1.0 - c.clamp(0.0, 1.0))
        .product::<f64>()
}

/// Impact score: confidence weighted by domain criticality.
/// Both inputs clamped to [0.0, 1.0].
pub fn impact_score(confidence: f64, criticality: f64) -> f64 {
    confidence.clamp(0.0, 1.0) * criticality.clamp(0.0, 1.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_path_is_identity() {
        assert!((path_confidence(&[]) - 1.0).abs() < f64::EPSILON);
    }

    #[test]
    fn empty_multi_path_is_zero() {
        assert!((multi_path_confidence(&[]) - 0.0).abs() < f64::EPSILON);
    }
}
