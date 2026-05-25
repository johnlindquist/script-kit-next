use std::path::Path;

use crate::ai::agent_chat::pi::launch_spec::PiLaunchSpec;

pub fn pi_warm_key(spec: &PiLaunchSpec) -> String {
    format!(
        "pi-warm-v1:{:016x}",
        fnv1a64(normalized_material(spec).as_bytes())
    )
}

pub fn normalized_material(spec: &PiLaunchSpec) -> String {
    let mut lines = Vec::new();
    lines.push(format!("pi_binary={}", normalize_path(&spec.pi_binary)));
    lines.push(format!(
        "profile_id={}",
        normalize_text_opt(spec.profile_id.as_deref())
    ));
    lines.push(format!(
        "profile_name={}",
        normalize_text_opt(spec.profile_name.as_deref())
    ));
    lines.push(format!("cwd={}", normalize_opt_path(spec.cwd.as_deref())));
    lines.push(format!(
        "provider={}",
        normalize_casefold_opt(spec.provider.as_deref())
    ));
    lines.push(format!(
        "model={}",
        normalize_casefold_opt(spec.model.as_deref())
    ));
    lines.push(format!(
        "thinking={}",
        normalize_casefold_opt(spec.thinking.as_deref())
    ));
    lines.push(format!(
        "system_prompt={}",
        normalize_text_opt(spec.system_prompt.as_deref())
    ));
    lines.push(format!(
        "append_system_prompt={}",
        normalize_text_opt(spec.append_system_prompt.as_deref())
    ));
    lines.push(format!(
        "tools={}",
        normalize_optional_list(spec.tools.as_deref(), true)
    ));
    lines.push(format!(
        "path_policy_json={}",
        normalize_text_opt(spec.path_policy_json.as_deref())
    ));
    lines.push(format!(
        "blocked_action_message={}",
        normalize_text_opt(spec.blocked_action_message.as_deref())
    ));
    lines.push(format!("disable_extensions={}", spec.disable_extensions));
    lines.push(format!(
        "extension_paths={}",
        normalize_list(&spec.extension_paths, false)
    ));
    lines.push(format!(
        "extension_policy={}",
        normalize_casefold_opt(spec.extension_policy.as_deref())
    ));
    lines.push(format!("disable_skills={}", spec.disable_skills));
    lines.push(format!(
        "skill_paths={}",
        normalize_list(&spec.skill_paths, false)
    ));
    lines.push(format!(
        "disable_prompt_templates={}",
        spec.disable_prompt_templates
    ));
    lines.push(format!(
        "prompt_template_paths={}",
        normalize_list(&spec.prompt_template_paths, false)
    ));
    lines.push(format!("hide_cwd_in_prompt={}", spec.hide_cwd_in_prompt));
    lines.push(format!(
        "session_dir={}",
        normalize_text_opt(spec.session_dir.as_deref())
    ));
    lines.push(format!("no_session={}", spec.no_session));
    lines.push(format!(
        "session_durability={}",
        normalize_casefold_opt(spec.session_durability.as_deref())
    ));
    lines.join("\n")
}

fn normalize_opt_path(path: Option<&Path>) -> String {
    path.map(normalize_path)
        .unwrap_or_else(|| "<none>".to_string())
}

fn normalize_path(path: &Path) -> String {
    normalize_separators(path.to_string_lossy().trim())
}

fn normalize_text_opt(value: Option<&str>) -> String {
    value
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .unwrap_or("<none>")
        .to_string()
}

fn normalize_casefold_opt(value: Option<&str>) -> String {
    normalize_text_opt(value).to_ascii_lowercase()
}

fn normalize_optional_list(values: Option<&[String]>, casefold: bool) -> String {
    match values {
        Some(values) => format!("some:{}", normalize_list(values, casefold)),
        None => "none".to_string(),
    }
}

fn normalize_list(values: &[String], casefold: bool) -> String {
    let mut normalized = values
        .iter()
        .filter_map(|value| {
            let trimmed = value.trim();
            if trimmed.is_empty() {
                None
            } else if casefold {
                Some(trimmed.to_ascii_lowercase())
            } else {
                Some(normalize_separators(trimmed))
            }
        })
        .collect::<Vec<_>>();
    normalized.sort();
    normalized.dedup();
    normalized.join(",")
}

fn normalize_separators(value: &str) -> String {
    value.replace('\\', "/")
}

fn fnv1a64(bytes: &[u8]) -> u64 {
    let mut hash = 0xcbf29ce484222325;
    for byte in bytes {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x100000001b3);
    }
    hash
}
