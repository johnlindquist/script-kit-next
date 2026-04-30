use crate::scripts::{Script, Scriptlet};

pub fn command_slug(input: &str) -> String {
    let mut slug = String::new();
    let mut last_was_dash = false;

    for ch in input.chars() {
        if ch.is_ascii_alphanumeric() {
            slug.push(ch.to_ascii_lowercase());
            last_was_dash = false;
        } else if !last_was_dash && !slug.is_empty() {
            slug.push('-');
            last_was_dash = true;
        }
    }

    while slug.ends_with('-') {
        slug.pop();
    }

    slug
}

pub fn script_command_head(script: &Script) -> String {
    script
        .alias
        .as_deref()
        .map(command_slug)
        .filter(|slug| !slug.is_empty())
        .or_else(|| {
            script
                .path
                .file_stem()
                .map(|stem| command_slug(&stem.to_string_lossy()))
                .filter(|slug| !slug.is_empty())
        })
        .unwrap_or_else(|| command_slug(&script.name))
}

pub fn scriptlet_command_head(scriptlet: &Scriptlet) -> String {
    scriptlet
        .command
        .as_deref()
        .map(command_slug)
        .filter(|slug| !slug.is_empty())
        .or_else(|| {
            scriptlet
                .alias
                .as_deref()
                .map(command_slug)
                .filter(|slug| !slug.is_empty())
        })
        .unwrap_or_else(|| command_slug(&scriptlet.name))
}

pub fn command_head_matches(input: &str, candidate: &str) -> bool {
    command_slug(input) == command_slug(candidate)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::scripts::{Script, Scriptlet};
    use std::path::PathBuf;

    #[test]
    fn command_slug_normalizes_names_without_shell_semantics() {
        assert_eq!(command_slug("Deploy Prod"), "deploy-prod");
        assert_eq!(command_slug("test-menu_syntax.ts"), "test-menu-syntax-ts");
        assert_eq!(command_slug("  Open PR!  "), "open-pr");
    }

    #[test]
    fn script_command_head_prefers_alias_then_file_stem() {
        let mut script = Script {
            name: "Deploy Prod".to_string(),
            path: PathBuf::from("/tmp/deploy-prod.ts"),
            extension: "ts".to_string(),
            ..Default::default()
        };
        assert_eq!(script_command_head(&script), "deploy-prod");
        script.alias = Some("dp".to_string());
        assert_eq!(script_command_head(&script), "dp");
    }

    #[test]
    fn scriptlet_command_head_prefers_declared_command() {
        let scriptlet = Scriptlet {
            name: "Open PR".to_string(),
            description: None,
            code: String::new(),
            tool: "ts".to_string(),
            shortcut: None,
            keyword: None,
            group: None,
            plugin_id: "main".to_string(),
            plugin_title: None,
            file_path: None,
            command: Some("open-pr".to_string()),
            alias: Some("pr".to_string()),
        };
        assert_eq!(scriptlet_command_head(&scriptlet), "open-pr");
    }
}
