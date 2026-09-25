//! Identity of a warm goose session.
//!
//! Two launches may share a warm session only when every input that shapes
//! the goose process and session is identical: binary, working directory,
//! recipe (instructions, tools, provider, model), reasoning level, and
//! session visibility.

use crate::ai::agent_chat::goose::GooseLaunchSpec;

pub(crate) fn launch_warm_key(spec: &GooseLaunchSpec) -> String {
    format!(
        "goose-warm-v1:{:016x}",
        fnv1a64(normalized_material(spec).as_bytes())
    )
}

pub(crate) fn normalized_material(spec: &GooseLaunchSpec) -> String {
    [
        format!(
            "binary={}",
            spec.command.to_string_lossy().trim().replace('\\', "/")
        ),
        format!("args={}", spec.args.join(" ")),
        format!(
            "cwd={}",
            spec.cwd.to_string_lossy().trim().replace('\\', "/")
        ),
        // Recipes are built by one function in a fixed order, so equal
        // recipes serialize identically.
        format!("recipe={}", spec.recipe),
        format!(
            "thinking={}",
            spec.thinking.as_deref().unwrap_or_default().trim()
        ),
        format!("hidden={}", spec.hidden_session),
        format!(
            "goose_root={}",
            spec.env
                .get("GOOSE_PATH_ROOT")
                .map(String::as_str)
                .unwrap_or_default()
        ),
    ]
    .join("\n")
}

fn fnv1a64(bytes: &[u8]) -> u64 {
    let mut hash = 0xcbf29ce484222325;
    for byte in bytes {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x100000001b3);
    }
    hash
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn warm_key_changes_with_cwd_or_recipe_but_not_with_credentials() {
        let mut spec = GooseLaunchSpec::new("/tmp/goose", "/tmp/a");
        spec.recipe = json!({"instructions": "one"});
        let base = launch_warm_key(&spec);

        let mut moved = spec.clone();
        moved.cwd = "/tmp/b".into();
        assert_ne!(launch_warm_key(&moved), base);

        let mut rewritten = spec.clone();
        rewritten.recipe = json!({"instructions": "two"});
        assert_ne!(launch_warm_key(&rewritten), base);

        let mut keyed = spec.clone();
        keyed.env.insert("OPENAI_API_KEY".into(), "secret".into());
        assert_eq!(launch_warm_key(&keyed), base);
        assert!(!normalized_material(&keyed).contains("secret"));
    }
}
