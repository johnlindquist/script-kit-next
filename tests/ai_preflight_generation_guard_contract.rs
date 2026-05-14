use std::fs;

const CONTEXT_PREFLIGHT: &str = "src/ai/window/context_preflight.rs";
const RENDER_MAIN_PANEL: &str = "src/ai/window/render_main_panel.rs";

#[test]
fn loading_generation_does_not_retain_prior_receipt_or_recommendations() {
    let source = fs::read_to_string(CONTEXT_PREFLIGHT).expect("read context preflight source");
    let schedule = function_body(&source, "schedule_context_preflight");

    assert!(
        schedule.contains("self.context_preflight = ContextPreflightState"),
        "schedule_context_preflight must assign a full ContextPreflightState for Loading"
    );
    assert!(
        schedule.contains("generation,")
            && schedule.contains("status: ContextPreflightStatus::Loading")
            && schedule.contains("..Default::default()"),
        "Loading assignment must carry generation/status and clear derived receipt state with Default"
    );
    assert!(
        !schedule.contains("self.context_preflight.status = ContextPreflightStatus::Loading"),
        "Loading must not mutate status in place because that retains stale receipt/recommendation data"
    );
}

#[test]
fn recommendation_apply_requires_current_generation_and_surfaced_action_id() {
    let source = fs::read_to_string(CONTEXT_PREFLIGHT).expect("read context preflight source");
    let apply = function_body(&source, "apply_context_recommendation");

    assert!(
        source.contains("fn apply_context_recommendation(\n        &mut self,\n        generation: u64,\n        action_id: &str,"),
        "apply_context_recommendation must accept rendered generation and action id"
    );
    assert!(
        apply.contains("self.context_preflight.generation != generation")
            && apply.contains("ai_context_recommendation_stale_dropped"),
        "apply path must drop stale-generation recommendation actions"
    );
    assert!(
        apply.contains("recommendation_resolution")
            && apply.contains(".surfaced")
            && apply.contains("item.action_id == action_id")
            && apply.contains("ai_context_recommendation_not_surfaced_dropped"),
        "apply path must require the action id to still be surfaced"
    );
    assert!(
        apply.find("still_surfaced").unwrap() < apply.find("self.add_context_part").unwrap(),
        "surfaced-action guard must precede add_context_part"
    );
}

#[test]
fn recommendation_render_passes_snapshot_generation_and_action_id() {
    let source = fs::read_to_string(RENDER_MAIN_PANEL).expect("read main panel source");
    let render = function_body(&source, "render_context_recommendations");

    assert!(
        render.contains("let generation = self.context_preflight.generation;")
            && render.contains("let action_id = recommendation.action_id().to_string();"),
        "render must capture the generation and rendered action id with the rail snapshot"
    );
    assert!(
        render.contains("this.apply_context_recommendation(generation, &action_id, kind, cx)"),
        "recommendation click handler must pass generation and action id to the guard"
    );
}

fn function_body<'a>(source: &'a str, name: &str) -> &'a str {
    let needle = format!("fn {name}");
    let start = source
        .find(&needle)
        .unwrap_or_else(|| panic!("missing {name}"));
    let after = &source[start..];
    let brace = after.find('{').expect("function has body");
    let body_start = start + brace;
    let mut depth = 0usize;
    for (offset, ch) in source[body_start..].char_indices() {
        match ch {
            '{' => depth += 1,
            '}' => {
                depth -= 1;
                if depth == 0 {
                    return &source[body_start..body_start + offset + 1];
                }
            }
            _ => {}
        }
    }
    panic!("unterminated function {name}");
}
