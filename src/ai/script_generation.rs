use anyhow::{Context, Result};
use std::fs::{self, OpenOptions};
use std::io::{ErrorKind, Write};
use std::path::{Path, PathBuf};
use std::sync::Arc;

use super::config::ModelInfo;
use super::providers::{AiProvider, ProviderMessage, ProviderRegistry};

const AI_SCRIPT_OUTPUT_DIR: &str = "~/.kenv/scripts";
const AI_SCRIPT_DEFAULT_SLUG: &str = "ai-script";
const AI_SCRIPT_MAX_SLUG_LEN: usize = 64;

const AI_SCRIPT_GENERATION_SYSTEM_PROMPT: &str = r#"You write production-ready Script Kit scripts.

Return ONLY TypeScript source code for one Script Kit script. Do not include explanations.

Required output conventions:
1) Include metadata comments at the top:
   // Name: <clear title>
   // Description: <one-line summary>
2) Include: import "@johnlindquist/kit";
3) Use await arg() for user input when useful.
4) Use await div() for display output when useful.
5) Keep the script runnable as-is with sensible defaults and light error handling."#;

#[derive(Debug, Clone)]
pub struct GeneratedScriptOutput {
    pub path: PathBuf,
    pub slug: String,
    pub model_id: String,
    pub provider_id: String,
}

pub fn generate_script_from_prompt(
    prompt: &str,
    config: Option<&crate::config::Config>,
) -> Result<GeneratedScriptOutput> {
    let normalized_prompt = prompt.trim();
    if normalized_prompt.is_empty() {
        anyhow::bail!("AI script generation requires a non-empty prompt");
    }

    let registry = ProviderRegistry::from_environment_with_config(config);
    if !registry.has_any_provider() {
        anyhow::bail!(
            "No AI providers configured. Configure an API key first (Vercel, OpenAI, Anthropic, etc.)."
        );
    }

    let (selected_model, provider) = select_generation_model(&registry)?;
    crate::logging::log(
        "AI",
        &format!(
            "correlation_id=ai-script-generation state=provider_ready model_id={} provider_id={} prompt_len={}",
            selected_model.id,
            selected_model.provider,
            normalized_prompt.len()
        ),
    );

    let messages = vec![
        ProviderMessage::system(AI_SCRIPT_GENERATION_SYSTEM_PROMPT),
        ProviderMessage::user(format!(
            "Generate a Script Kit script for this user request:\n\n{}",
            normalized_prompt
        )),
    ];

    let raw_response = provider
        .send_message(&messages, &selected_model.id)
        .with_context(|| {
            format!(
                "AI script generation failed (attempted=send_message, model_id={}, provider_id={})",
                selected_model.id, selected_model.provider
            )
        })?;

    let extracted = extract_script_code(&raw_response);
    if extracted.trim().is_empty() {
        anyhow::bail!("AI returned an empty response for script generation (state=empty_response)");
    }

    let slug = slugify_script_name(normalized_prompt);
    let finalized = enforce_script_kit_conventions(&extracted, normalized_prompt, &slug);
    let path = write_generated_script(&slug, &finalized).with_context(|| {
        format!(
            "Failed writing AI-generated script (state=write_failed, slug={})",
            slug
        )
    })?;

    crate::logging::log(
        "AI",
        &format!(
            "correlation_id=ai-script-generation state=script_written path={} slug={}",
            path.display(),
            slug
        ),
    );

    Ok(GeneratedScriptOutput {
        path,
        slug,
        model_id: selected_model.id,
        provider_id: selected_model.provider,
    })
}

fn select_generation_model(
    registry: &ProviderRegistry,
) -> Result<(ModelInfo, Arc<dyn AiProvider>)> {
    let models = registry.get_all_models();
    let selected_model = models
        .iter()
        .find(|model| model.provider.eq_ignore_ascii_case("vercel"))
        .or_else(|| models.first())
        .cloned()
        .context("No AI models available in provider registry")?;

    let provider = registry
        .find_provider_for_model(&selected_model.id)
        .cloned()
        .with_context(|| {
            format!(
                "No provider found for selected model '{}' (state=provider_missing)",
                selected_model.id
            )
        })?;

    Ok((selected_model, provider))
}

fn generated_scripts_dir() -> PathBuf {
    PathBuf::from(shellexpand::tilde(AI_SCRIPT_OUTPUT_DIR).as_ref())
}

fn split_fence_header_and_body(fence: &str) -> (&str, &str) {
    match fence.find('\n') {
        Some(newline_index) => (&fence[..newline_index], &fence[newline_index + 1..]),
        None => ("", fence),
    }
}

fn extract_fenced_code(response: &str, preferred_languages: Option<&[&str]>) -> Option<String> {
    let mut remaining = response;

    while let Some(start) = remaining.find("```") {
        let after_start = &remaining[start + 3..];
        let Some(end) = after_start.find("```") else {
            break;
        };

        let fence_contents = &after_start[..end];
        let (header, body) = split_fence_header_and_body(fence_contents);
        let language = header
            .trim()
            .split(|c: char| c.is_whitespace() || c == '{')
            .next()
            .unwrap_or("")
            .to_ascii_lowercase();
        let code = body.trim();

        if !code.is_empty() {
            match preferred_languages {
                Some(preferred) => {
                    if preferred.iter().any(|candidate| *candidate == language) {
                        return Some(code.to_string());
                    }
                }
                None => return Some(code.to_string()),
            }
        }

        remaining = &after_start[end + 3..];
    }

    None
}

fn extract_script_code(response: &str) -> String {
    const PREFERRED_LANGUAGES: [&str; 6] = ["typescript", "ts", "javascript", "js", "tsx", "jsx"];

    extract_fenced_code(response, Some(&PREFERRED_LANGUAGES))
        .or_else(|| extract_fenced_code(response, None))
        .unwrap_or_else(|| response.trim().to_string())
}

fn slugify_script_name(prompt: &str) -> String {
    let mut slug = String::new();
    let mut last_was_hyphen = false;

    for character in prompt.to_ascii_lowercase().chars() {
        if character.is_ascii_alphanumeric() {
            slug.push(character);
            last_was_hyphen = false;
        } else if matches!(character, ' ' | '_' | '-') {
            if !slug.is_empty() && !last_was_hyphen {
                slug.push('-');
                last_was_hyphen = true;
            }
        }
    }

    while slug.ends_with('-') {
        slug.pop();
    }

    if slug.len() > AI_SCRIPT_MAX_SLUG_LEN {
        slug.truncate(AI_SCRIPT_MAX_SLUG_LEN);
        while slug.ends_with('-') {
            slug.pop();
        }
    }

    if slug.is_empty() {
        AI_SCRIPT_DEFAULT_SLUG.to_string()
    } else {
        slug
    }
}

fn slug_to_title(slug: &str) -> String {
    slug.split('-')
        .filter(|segment| !segment.is_empty())
        .map(|segment| {
            let mut chars = segment.chars();
            match chars.next() {
                Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
                None => String::new(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

fn description_from_prompt(prompt: &str) -> String {
    let normalized = prompt.split_whitespace().collect::<Vec<_>>().join(" ");
    if normalized.is_empty() {
        return "AI-generated Script Kit script".to_string();
    }

    let mut shortened = normalized;
    if shortened.chars().count() > 110 {
        shortened = format!("{}...", shortened.chars().take(107).collect::<String>());
    }
    shortened
}

fn has_kit_import(script: &str) -> bool {
    script.contains("import \"@johnlindquist/kit\";")
        || script.contains("import '@johnlindquist/kit';")
        || script.contains("import \"@scriptkit/sdk\";")
        || script.contains("import '@scriptkit/sdk';")
}

fn enforce_script_kit_conventions(script: &str, prompt: &str, slug: &str) -> String {
    let mut prefix_lines: Vec<String> = Vec::new();
    let trimmed_script = script.trim();

    if !trimmed_script
        .lines()
        .any(|line| line.trim_start().starts_with("// Name:"))
    {
        prefix_lines.push(format!("// Name: {}", slug_to_title(slug)));
    }

    if !trimmed_script
        .lines()
        .any(|line| line.trim_start().starts_with("// Description:"))
    {
        prefix_lines.push(format!(
            "// Description: {}",
            description_from_prompt(prompt)
        ));
    }

    if !has_kit_import(trimmed_script) {
        prefix_lines.push("import \"@johnlindquist/kit\";".to_string());
    }

    let mut output = String::new();
    if !prefix_lines.is_empty() {
        output.push_str(&prefix_lines.join("\n"));
        output.push_str("\n\n");
    }

    output.push_str(trimmed_script);
    if !output.ends_with('\n') {
        output.push('\n');
    }
    output
}

fn write_generated_script(slug: &str, script_content: &str) -> Result<PathBuf> {
    write_generated_script_in_dir(&generated_scripts_dir(), slug, script_content)
}

fn write_generated_script_in_dir(
    output_dir: &Path,
    slug: &str,
    script_content: &str,
) -> Result<PathBuf> {
    fs::create_dir_all(output_dir)
        .with_context(|| format!("Failed to create output dir: {}", output_dir.display()))?;

    for suffix in 0usize.. {
        let candidate = if suffix == 0 {
            slug.to_string()
        } else {
            format!("{slug}-{suffix}")
        };
        let path = output_dir.join(format!("{}.ts", candidate));
        match OpenOptions::new().write(true).create_new(true).open(&path) {
            Ok(mut file) => {
                file.write_all(script_content.as_bytes()).with_context(|| {
                    format!("Failed writing generated script to {}", path.display())
                })?;
                return Ok(path);
            }
            Err(error) if error.kind() == ErrorKind::AlreadyExists => continue,
            Err(error) => {
                return Err(error).with_context(|| {
                    format!(
                        "Failed creating generated script file {} (slug={})",
                        path.display(),
                        slug
                    )
                });
            }
        }
    }

    unreachable!("suffix loop should eventually create a unique script filename")
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_slugify_script_name_handles_spaces_and_symbols() {
        assert_eq!(
            slugify_script_name("Build: API Client!"),
            "build-api-client"
        );
        assert_eq!(slugify_script_name("  ___  "), "ai-script");
    }

    #[test]
    fn test_extract_script_code_prefers_typescript_fence_when_multiple_blocks_exist() {
        let response = r#"
Here's one idea:
```markdown
Not code
```
```typescript
await div("hello");
```
"#;
        assert_eq!(extract_script_code(response), "await div(\"hello\");");
    }

    #[test]
    fn test_extract_script_code_falls_back_to_first_fenced_block() {
        let response = r#"
```python
print("hello")
```
"#;
        assert_eq!(extract_script_code(response), "print(\"hello\")");
    }

    #[test]
    fn test_extract_script_code_returns_trimmed_response_when_no_fence_exists() {
        let response = "const answer = 42;";
        assert_eq!(extract_script_code(response), "const answer = 42;");
    }

    #[test]
    fn test_enforce_script_kit_conventions_adds_missing_metadata_and_import() {
        let script = "const name = await arg(\"Name?\");";
        let output = enforce_script_kit_conventions(script, "Ask for user name", "ask-user-name");

        assert!(output.contains("// Name: Ask User Name"));
        assert!(output.contains("// Description: Ask for user name"));
        assert!(output.contains("import \"@johnlindquist/kit\";"));
        assert!(output.contains("await arg(\"Name?\");"));
    }

    #[test]
    fn test_enforce_script_kit_conventions_keeps_existing_metadata_and_import() {
        let script = r#"// Name: Existing
// Description: Existing description
import "@johnlindquist/kit";

await div("ready");
"#;
        let output = enforce_script_kit_conventions(script, "ignored", "ignored");

        assert_eq!(
            output.matches("// Name:").count(),
            1,
            "should not duplicate existing Name metadata"
        );
        assert_eq!(
            output.matches("// Description:").count(),
            1,
            "should not duplicate existing Description metadata"
        );
        assert_eq!(
            output.matches("import \"@johnlindquist/kit\";").count(),
            1,
            "should not duplicate existing import"
        );
    }

    #[test]
    fn test_write_generated_script_in_dir_appends_numeric_suffix_for_collisions() {
        let temp_dir = tempdir().unwrap();
        let first = write_generated_script_in_dir(temp_dir.path(), "my-script", "a").unwrap();
        let second = write_generated_script_in_dir(temp_dir.path(), "my-script", "b").unwrap();

        assert_eq!(first.file_name().unwrap(), "my-script.ts");
        assert_eq!(second.file_name().unwrap(), "my-script-1.ts");
    }
}
