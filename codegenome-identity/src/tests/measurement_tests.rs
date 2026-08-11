use crate::measurement::*;

// --- GroundTruth Classification ---

struct SyntaxMeasurement;

impl Measurable for SyntaxMeasurement {
    type Property = String;
    type Evidence = bool;

    fn correct(&self, evidence: &bool) -> bool {
        *evidence
    }

    fn detect_wrong(&self, evidence: &bool) -> Option<Violation> {
        if *evidence {
            None
        } else {
            Some(Violation::new(
                "syntax_parse",
                "AST does not match source",
                FailureMode::Observable,
            ))
        }
    }

    fn failure_mode(&self) -> FailureMode {
        FailureMode::Observable
    }

    fn ground_truth_level(&self) -> GroundTruthLevel {
        GroundTruthLevel::Available
    }
}

struct CompactionMeasurement;

impl Measurable for CompactionMeasurement {
    type Property = String;
    type Evidence = f64;

    fn correct(&self, evidence: &f64) -> bool {
        *evidence >= 0.95
    }

    fn detect_wrong(&self, evidence: &f64) -> Option<Violation> {
        if *evidence >= 0.95 {
            None
        } else {
            Some(Violation::new(
                "compaction_fidelity",
                "Fidelity below threshold",
                FailureMode::Silent,
            ))
        }
    }

    fn failure_mode(&self) -> FailureMode {
        FailureMode::Silent
    }

    fn ground_truth_level(&self) -> GroundTruthLevel {
        GroundTruthLevel::Unsolved
    }
}

#[test]
fn syntax_reports_available_ground_truth() {
    let m = SyntaxMeasurement;
    assert_eq!(m.ground_truth_level(), GroundTruthLevel::Available);
}

#[test]
fn compaction_reports_unsolved_ground_truth() {
    let m = CompactionMeasurement;
    assert_eq!(m.ground_truth_level(), GroundTruthLevel::Unsolved);
}

#[test]
fn violation_detection_correct_input() {
    let m = SyntaxMeasurement;
    assert!(m.detect_wrong(&true).is_none());
}

#[test]
fn violation_detection_wrong_input() {
    let m = SyntaxMeasurement;
    let violation = m.detect_wrong(&false);
    assert!(violation.is_some());
    assert_eq!(violation.unwrap().failure_mode, FailureMode::Observable);
}

#[test]
fn catastrophic_failure_requires_cross_overlay_impact() {
    // FailureMode::Catastrophic should only be used for
    // properties whose violation cascades. Syntax parse
    // failure is Observable (local), not Catastrophic.
    let m = SyntaxMeasurement;
    assert_ne!(m.failure_mode(), FailureMode::Catastrophic);
}
