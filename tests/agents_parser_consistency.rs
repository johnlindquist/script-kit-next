use std::path::Path;

use script_kit_gpui::agents::parse_agent;

#[test]
fn test_parse_agent_returns_none_when_frontmatter_yaml_is_malformed() {
    let path = Path::new("/path/to/bad-frontmatter.claude.md");
    let content = r#"---
_sk_name: "Broken
model: sonnet
---
Prompt"#;

    assert!(parse_agent(path, content).is_none());
}
