mod ground_truth;

pub use ground_truth::{FailureMode, GroundTruthLevel, Violation};

/// For any property P of the system, define what
/// correct/wrong/failure looks like. This is not a test
/// framework — it is the algebra of evaluation.
pub trait Measurable {
    type Property;
    type Evidence;

    /// What does "correct" look like for this property?
    fn correct(&self, evidence: &Self::Evidence) -> bool;

    /// How do you detect when this property is wrong?
    fn detect_wrong(&self, evidence: &Self::Evidence) -> Option<Violation>;

    /// What does the failure mode look like?
    fn failure_mode(&self) -> FailureMode;

    /// Ground truth availability.
    fn ground_truth_level(&self) -> GroundTruthLevel;
}
