use crate::experiments::review::{random_params, Action, ReviewState};

#[test]
fn improving_fitness_returns_continue() {
    let mut reviewer = ReviewState::new(10, 3, 0.1);
    for i in 1..=5 {
        let action = reviewer.assess(i as f64 * 0.1);
        assert!(matches!(action, Action::Continue));
    }
}

#[test]
fn plateau_triggers_widen_search() {
    let mut reviewer = ReviewState::new(5, 3, 0.1);
    reviewer.assess(0.5); // baseline
                          // 5 non-improving iterations — 5th triggers widen
    let mut last = Action::Continue;
    for _ in 0..5 {
        last = reviewer.assess(0.3);
    }
    match last {
        Action::WidenSearch(scale) => {
            assert!(scale > 0.1, "Scale should increase, got {scale}");
        }
        other => panic!("Expected WidenSearch, got {other:?}"),
    }
}

#[test]
fn repeated_plateaus_trigger_restart() {
    let mut reviewer = ReviewState::new(3, 2, 0.1);
    reviewer.assess(0.5); // baseline

    // First plateau (3 iters) → widen
    let mut last = Action::Continue;
    for _ in 0..3 {
        last = reviewer.assess(0.3);
    }
    assert!(matches!(last, Action::WidenSearch(_)));

    // Second plateau (3 iters) → restart
    for _ in 0..3 {
        last = reviewer.assess(0.3);
    }
    assert!(matches!(last, Action::Restart));
}

#[test]
fn improvement_after_plateau_resets_counts() {
    let mut reviewer = ReviewState::new(5, 3, 0.1);
    reviewer.assess(0.5); // baseline
                          // Partial plateau (4 of 5 needed)
    for _ in 0..4 {
        reviewer.assess(0.3);
    }
    // Improve — should reset counts
    let action = reviewer.assess(0.6);
    assert!(matches!(action, Action::Continue));
    // Now need full 5 again — 4 should still be Continue
    for _ in 0..4 {
        let a = reviewer.assess(0.3);
        assert!(matches!(a, Action::Continue));
    }
    // 5th triggers widen
    let a = reviewer.assess(0.3);
    assert!(matches!(a, Action::WidenSearch(_)));
}

#[test]
fn widen_scale_doubles_each_time() {
    let mut reviewer = ReviewState::new(2, 5, 0.1);
    reviewer.assess(0.5); // baseline
    let expected_scales = [0.2, 0.4, 0.8];
    for expected in expected_scales {
        // Trigger plateau (2 iters)
        let mut last = Action::Continue;
        for _ in 0..2 {
            last = reviewer.assess(0.3);
        }
        match last {
            Action::WidenSearch(scale) => {
                assert!(
                    (scale - expected).abs() < 0.001,
                    "Expected {expected}, got {scale}"
                );
            }
            other => panic!("Expected WidenSearch({expected}), got {other:?}"),
        }
    }
}

#[test]
fn random_params_within_bounds() {
    for _ in 0..10 {
        let params = random_params();
        let ct = params.values.get("confidence_threshold").unwrap();
        let af = params.values.get("attenuation_factor").unwrap();
        let md = params.values.get("max_depth").unwrap();
        assert!(*ct >= 0.01 && *ct <= 0.99, "confidence_threshold={ct}");
        assert!(*af >= 0.1 && *af <= 2.0, "attenuation_factor={af}");
        assert!(*md >= 1.0 && *md <= 20.0, "max_depth={md}");
    }
}
