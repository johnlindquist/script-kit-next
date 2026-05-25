use std::path::PathBuf;

use script_kit_gpui::ai::agent_chat::pi::launch_spec::PiLaunchSpec;
use script_kit_gpui::ai::agent_chat::warm_key::{normalized_material, pi_warm_key};

fn spec() -> PiLaunchSpec {
    PiLaunchSpec {
        pi_binary: PathBuf::from("pi"),
        profile_id: Some("script-kit".to_string()),
        profile_name: Some("Script Kit".to_string()),
        cwd: Some(PathBuf::from("/Users/test/.scriptkit")),
        provider: Some("openai-codex".to_string()),
        model: Some("gpt-5.4".to_string()),
        thinking: None,
        system_prompt: None,
        append_system_prompt: Some("base prompt".to_string()),
        tools: Some(vec!["read".to_string(), "write".to_string()]),
        path_policy_json: None,
        blocked_action_message: None,
        disable_extensions: true,
        extension_paths: Vec::new(),
        extension_policy: None,
        disable_skills: true,
        skill_paths: Vec::new(),
        disable_prompt_templates: true,
        prompt_template_paths: Vec::new(),
        hide_cwd_in_prompt: false,
        session_dir: None,
        no_session: false,
        session_durability: None,
    }
}

#[test]
fn warm_key_is_derived_from_normalized_launch_spec_not_profile_name() {
    let first = spec();
    let second = spec();
    assert_eq!(pi_warm_key(&first), pi_warm_key(&second));
    assert_eq!(normalized_material(&first), normalized_material(&second));
}

#[test]
fn warm_key_changes_when_cwd_changes() {
    let first = spec();
    let mut second = spec();
    second.cwd = Some(PathBuf::from("/Users/test/.scriptkit/agent-chat/general"));
    assert_ne!(pi_warm_key(&first), pi_warm_key(&second));
}

#[test]
fn warm_key_normalizes_tool_order_whitespace_and_duplicates() {
    let mut first = spec();
    first.tools = Some(vec![
        " write ".to_string(),
        "READ".to_string(),
        "read".to_string(),
    ]);

    let mut second = spec();
    second.tools = Some(vec!["read".to_string(), "write".to_string()]);

    assert_eq!(pi_warm_key(&first), pi_warm_key(&second));
    assert!(normalized_material(&first).contains("tools=some:read,write"));
}

#[test]
fn warm_key_distinguishes_no_tools_from_pi_default_tools() {
    let mut no_tools = spec();
    no_tools.tools = Some(Vec::new());

    let mut pi_default_tools = spec();
    pi_default_tools.tools = None;

    assert_ne!(pi_warm_key(&no_tools), pi_warm_key(&pi_default_tools));
    assert!(normalized_material(&no_tools).contains("tools=some:"));
    assert!(normalized_material(&pi_default_tools).contains("tools=none"));
}

#[test]
fn warm_key_changes_when_prompt_or_resource_policy_changes() {
    let first = spec();

    let mut prompt_changed = spec();
    prompt_changed.append_system_prompt = Some("different prompt".to_string());
    assert_ne!(pi_warm_key(&first), pi_warm_key(&prompt_changed));

    let mut policy_changed = spec();
    policy_changed.disable_extensions = false;
    policy_changed.extension_paths = vec!["/tmp/ext".to_string()];
    policy_changed.extension_policy = Some("Allow".to_string());
    assert_ne!(pi_warm_key(&first), pi_warm_key(&policy_changed));
}
