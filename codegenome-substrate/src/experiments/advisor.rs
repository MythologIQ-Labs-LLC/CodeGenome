use crate::experiments::log::ExperimentResult;
use crate::experiments::review::Action;

/// Build a prompt from experiment history for the LLM advisor.
/// Pure function: history in, prompt string out.
pub fn build_prompt(history: &[ExperimentResult], available_fitness: &[&str]) -> String {
    let mut prompt = String::from(
        "You are analyzing experiment results for a code graph system.\n\
         Given the history below, recommend ONE action.\n\n\
         HISTORY (iteration, fitness, stability, cycle_ms, description):\n",
    );
    for r in history.iter().rev().take(20) {
        prompt.push_str(&format!(
            "{}\t{:.4}\t{:.4}\t{}\t{}\n",
            r.iteration, r.fitness, r.stability, r.cycle_time_ms, r.description,
        ));
    }
    prompt.push_str("\nAVAILABLE ACTIONS:\n");
    for name in available_fitness {
        prompt.push_str(&format!("- SWITCH_FITNESS({name})\n"));
    }
    prompt.push_str("- WIDEN (increase search range)\n");
    prompt.push_str("- RESTART (random parameters)\n");
    prompt.push_str("- CONTINUE (no change)\n");
    prompt.push_str("\nRespond with exactly ONE action.\n");
    prompt
}

/// Query a local LLM via mistralrs. Blocks on a scoped tokio runtime.
/// Returns the raw response text or an error.
pub fn query_model(prompt: &str, model_id: &str) -> Result<String, String> {
    let rt = tokio::runtime::Runtime::new().map_err(|e| format!("tokio init failed: {e}"))?;
    rt.block_on(query_model_async(prompt, model_id))
}

async fn query_model_async(prompt: &str, model_id: &str) -> Result<String, String> {
    use mistralrs::{IsqBits, ModelBuilder};

    let model = ModelBuilder::new(model_id)
        .with_auto_isq(IsqBits::Four)
        .build()
        .await
        .map_err(|e| format!("model load failed: {e}"))?;

    let response = model
        .chat(prompt)
        .await
        .map_err(|e| format!("inference failed: {e}"))?;

    Ok(response.to_string())
}

/// Parse a model response into an Action via keyword matching.
/// Pure function: response string in, Action out.
pub fn parse_advice(response: &str) -> Action {
    let lower = response.to_lowercase();
    if let Some(name) = extract_switch_fitness(&lower, response) {
        return Action::SwitchFitness(name);
    }
    if lower.contains("widen") {
        return Action::WidenSearch(0.4);
    }
    if lower.contains("restart") {
        return Action::Restart;
    }
    Action::Continue
}

/// Top-level advisor: build prompt, query model, parse response.
/// Gracefully degrades to Continue on any failure.
pub fn advise(history: &[ExperimentResult], model_id: &str, available_fitness: &[&str]) -> Action {
    let prompt = build_prompt(history, available_fitness);
    match query_model(&prompt, model_id) {
        Ok(response) => {
            eprintln!("[ADVISOR] response: {response}");
            parse_advice(&response)
        }
        Err(e) => {
            eprintln!("[ADVISOR] failed: {e}");
            Action::Continue
        }
    }
}

fn extract_switch_fitness(lower: &str, original: &str) -> Option<String> {
    let marker = "switch_fitness(";
    let start = lower.find(marker)? + marker.len();
    let end = original[start..].find(')')? + start;
    let name = original[start..end].trim().to_string();
    if name.is_empty() {
        None
    } else {
        Some(name)
    }
}
