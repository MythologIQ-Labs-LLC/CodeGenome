use crate::experiments::advisor::{build_prompt, parse_advice};
use crate::experiments::config::ExperimentStatus;
use crate::experiments::log::ExperimentResult;
use crate::experiments::review::Action;

fn sample_results() -> Vec<ExperimentResult> {
    (0..5)
        .map(|i| ExperimentResult {
            iteration: i,
            params: std::collections::HashMap::new(),
            fitness: 0.3 + i as f64 * 0.01,
            stability: 0.9,
            status: ExperimentStatus::Pass,
            cycle_time_ms: 100,
            description: format!("iter {i}"),
            chain_hash: String::new(),
        })
        .collect()
}

#[test]
fn build_prompt_includes_history() {
    let results = sample_results();
    let prompt = build_prompt(&results, &["ImpactAccuracy", "PropagationDepth"]);
    assert!(
        prompt.contains("0.3000"),
        "Prompt should contain fitness values"
    );
    assert!(
        prompt.contains("iter 0"),
        "Prompt should contain descriptions"
    );
    assert!(prompt.contains("0.9000"), "Prompt should contain stability");
}

#[test]
fn build_prompt_includes_available_actions() {
    let results = sample_results();
    let prompt = build_prompt(&results, &["ImpactAccuracy", "PropagationDepth"]);
    assert!(
        prompt.contains("SWITCH_FITNESS"),
        "Should list SWITCH_FITNESS"
    );
    assert!(prompt.contains("WIDEN"), "Should list WIDEN");
    assert!(prompt.contains("RESTART"), "Should list RESTART");
    assert!(prompt.contains("CONTINUE"), "Should list CONTINUE");
    assert!(
        prompt.contains("PropagationDepth"),
        "Should list available fitness functions"
    );
}

#[test]
fn parse_advice_switch_fitness() {
    let action = parse_advice("I recommend SWITCH_FITNESS(PropagationDepth) for better results.");
    match action {
        Action::SwitchFitness(name) => assert_eq!(name, "PropagationDepth"),
        other => panic!("Expected SwitchFitness, got {other:?}"),
    }
}

#[test]
fn parse_advice_widen() {
    let action = parse_advice("You should WIDEN the search space significantly.");
    assert!(matches!(action, Action::WidenSearch(_)));
}

#[test]
fn parse_advice_restart() {
    let action = parse_advice("RESTART with completely fresh parameters.");
    assert!(matches!(action, Action::Restart));
}

#[test]
fn parse_advice_gibberish_returns_continue() {
    let action = parse_advice("The weather is nice today, I like cats.");
    assert!(matches!(action, Action::Continue));
}

#[test]
fn parse_advice_case_insensitive() {
    let action = parse_advice("try switch_fitness(CycleTime) next");
    match action {
        Action::SwitchFitness(name) => assert_eq!(name, "CycleTime"),
        other => panic!("Expected SwitchFitness, got {other:?}"),
    }
}
