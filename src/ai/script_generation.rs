use anyhow::{Context, Result};
use itertools::Itertools;
use std::fs;
use std::path::PathBuf;
use std::sync::Arc;

use super::config::ModelInfo;
use super::providers::{AiProvider, ProviderMessage, ProviderRegistry};

const AI_SCRIPT_DEFAULT_SLUG: &str = "ai-script";
const AI_SCRIPT_MAX_SLUG_LEN: usize = 64;
const SCRIPT_KIT_SDK_IMPORT_MODULE: &str = "@scriptkit/sdk";
const SCRIPT_KIT_SDK_IMPORT_STATEMENT: &str = "import \"@scriptkit/sdk\";";
const AI_SCRIPT_USER_REQUEST_START_DELIMITER: &str = "---USER_REQUEST---";
const AI_SCRIPT_USER_REQUEST_END_DELIMITER: &str = "---END_REQUEST---";
const AI_SCRIPT_SHELL_EXECUTION_PATTERNS: [(&str, &str); 5] = [
    ("child_process", "child_process"),
    ("exec", "exec"),
    ("execSync", "execsync"),
    ("spawn", "spawn"),
    ("spawnSync", "spawnsync"),
];

pub(crate) const AI_SCRIPT_GENERATION_SYSTEM_PROMPT: &str = r#"You write production-ready Script Kit TypeScript scripts.

CRITICAL: Your ENTIRE response must be valid TypeScript. No prose, no markdown, no explanations, no preamble, no postamble. The very first character of your response must be "/" (the start of "// Name:"). If you include ANY text that is not valid TypeScript, the script will fail to parse and crash.

WRONG (will crash):
**Assumed:** You want a script that...
// Name: My Script

WRONG (will crash):
Here's a script that does X:
```typescript
// Name: My Script
```

RIGHT (the ONLY acceptable format):
// Name: My Script
// Description: Does something useful
import "@scriptkit/sdk";

NON-NEGOTIABLE OUTPUT FORMAT

1. The FIRST line must be: // Name: <short, clear, user-facing title>
2. The SECOND line must be: // Description: <one-line summary>
3. Near the top include EXACTLY ONE import (and no others):
   import "@scriptkit/sdk";
4. Use top-level await (no main(), no async IIFE, no servers).
5. Prefer Script Kit prompts + UI over console.log.
6. NO markdown fences. NO explanations. NO commentary. ONLY TypeScript code.

RUNTIME ASSUMPTIONS

* Script Kit provides globals for prompts/UI, filesystem, HTTP, clipboard, automation, and AI.
* Do not import node:* modules. Use global path.* utilities plus prompts like path()/find().
* Write scripts that feel like native tools: interactive, fast, keyboard-friendly.

UX QUALITY BAR

* Ask for missing inputs (arg/fields/path/find/drop/editor). Don't hardcode what you can prompt for.
* Prefer interactive UI over logs: show results with div(md(...)) or editor(...).
* Lists should be rich choice objects: { name, value, description?, preview? }.
  * preview is HTML (often md(...)) or a function returning HTML.
* Add actions for common operations (Copy/Open/Save/Reveal/Retry) via Action[] on arg/div/editor or setActions().
* Use sensible defaults and remember preferences when helpful (env/db/store).

ERROR HANDLING

* Treat Esc/cancel as normal: catch and exit quietly (or toast/notify if useful).
* Validate input early; for strict validation use arg({ onSubmit }) + setHint/setEnter + preventSubmit.
* For exec(), capture/show stderr/stdout on failure (editor/div) and suggest next steps.
* For long tasks: show progress (setStatus/setLoading/setProgress) or a "working" div with onInit + submit.

SCRIPT KIT IDIOMS (PREFERRED)

* await arg()/select()/grid()/fields()/editor()/div(md(...))
* home()/kenvPath()/tmpPath() + path.join/extname/basename (no imports)
* clipboard.* + getClipboardHistory() for clipboard tools
* await hide() before disruptive automation (keyboard/mouse/exec) when you don't need the prompt visible
* div({ html, onInit }) + submit(value) for "working…" screens
* Use isMac/isWin/isLinux for platform-specific behavior when unavoidable

TEACH BY EXAMPLE (REFERENCE ONLY — ADAPT PATTERNS, DO NOT COPY VERBATIM)

Example 1 — Simple input → output (arg + file write)
// Name: Save Note
// Description: Prompt for a note and save it as a text file
import "@scriptkit/sdk";
const note = await arg("Note text");
const outDir = home("Documents", "Notes");
await ensureDir(outDir);
const filePath = path.join(outDir, `note-${new Date().toISOString().slice(0, 10)}.txt`);
await writeFile(filePath, note, "utf8");
await div(md(`✅ Saved to: \`${filePath}\``));

Example 2 — List with choices + preview
// Name: Clipboard Picker
// Description: Search clipboard history, preview items, copy the selection
import "@scriptkit/sdk";
const items = (await getClipboardHistory()).slice(0, 100);
const value = await arg("Pick a clipboard item", items.map((i) => ({
  name: i.name || i.value.slice(0, 80),
  description: i.description || formatDateToNow(new Date(i.timestamp)),
  value: i.value,
  preview: md(`## Preview\n\n${i.value.slice(0, 2000)}`),
})));
await clipboard.writeText(value);
toast("Copied");

Example 3 — Multi-step workflow + rich HTML output (div(md()))
// Name: Markdown Card Builder
// Description: Collect fields, edit markdown, then render a styled preview
import "@scriptkit/sdk";
const [title, tags] = await fields(["Title", "Tags (comma-separated)"]);
const initial = `# ${title}\n\nTags: ${tags}\n\nWrite your content here...\n`;
const markdown = await editor(initial);
await div({ html: md(markdown), containerClasses: "p-6 prose dark:prose-invert" }, [
  { name: "Copy", shortcut: `${cmd}+c`, onAction: () => clipboard.writeText(markdown) },
  { name: "Save", shortcut: `${cmd}+s`, onAction: () => writeFile(home("Desktop", `${title}.md`), markdown, "utf8") },
]);

Example 4 — AI-powered helper (ai())
// Name: AI Rewrite
// Description: Rewrite text in a chosen tone using ai()
import "@scriptkit/sdk";
const tone = await arg("Tone", ["Concise", "Friendly", "Professional"]);
const input = await editor("Paste text to rewrite...");
const rewrite = ai(`Rewrite the text in a ${tone} tone. Return only the rewritten text.`);
const output = await rewrite(input);
await editor(output);

Example 5 — System automation (exec + readFile/writeFile)
// Name: Quick Replace In File
// Description: Replace text in a file, save, then open it
import "@scriptkit/sdk";
const filePath = await path({ hint: "Select a text file to edit" });
const findText = await arg("Find");
const replaceText = await arg("Replace with");
const before = await readFile(filePath, "utf8");
await writeFile(filePath, before.split(findText).join(replaceText), "utf8");
const openCmd = isMac ? "open" : isWin ? "start" : "xdg-open";
await exec(`${openCmd} "${filePath}"`);
toast("Updated");

COMPACT API REFERENCE (ONE LINE PER FUNCTION, GROUPED)

Prompts & Rendering
* arg(...) — text input or searchable choices (supports actions + preview)
* select(...) — multi-select list
* grid(...) — multi-select grid
* fields(...) — quick form, returns string[]
* editor(...) — edit/copy large text
* div(...) — render HTML (pair with md())
* form(...) — HTML form, returns object
* textarea(...) — simple textarea
* drop(...) — drag/drop files or text
* find(...) — file search prompt
* path(...) — file/folder picker (also provides path.join/etc)
* onTab(name, fn) — multi-tab prompt flows

UI Helpers
* md(markdown) — markdown to HTML
* toast(message, options?) — in-window toast
* notify(bodyOrOptions) — system notification
* setActions(actions, options?) — action palette w/ shortcuts
* openActions() — open actions menu
* setHint(text) — hint under input
* setEnter(text) — enter button label
* setFooter(text) — footer content
* setPreview(html, classes?) — preview panel
* setPanel(html, classes?) — panel content
* setLoading(boolean) — spinner/loading state
* setProgress(number) — progress bar 0..1
* setStatus({ status, message }) — tray status + message
* show() — show prompt
* hide(options?) — hide prompt
* blur() — focus previous app
* submit(value) — force submit
* preventSubmit — block submit from onSubmit

Config, State, Time
* env(key, promptOrFn?) — read/prompt and persist env var
* db(dataOrKeyOrPath?, data?, fromCache?) — lightweight JSON DB
* store(key, initial?) — persistent key-value store
* wait(ms, submitValue?) — delay (optionally submit)

Files & Paths
* home(...) — home-relative path
* kenvPath(...) — ~/.kenv-relative path
* tmpPath(...) — temp path
* ensureDir(path) — ensure dir exists
* ensureFile(path) — ensure file exists
* readFile(path, enc?) — read file
* writeFile(path, data, enc?) — write file
* readdir(path) — list dir
* pathExists(path) — exists?
* globby(patterns) — glob files
* replace({ files, from, to }) — replace text in files

Web & Data
* get(url, config?) — HTTP GET
* post(url, data?, config?) — HTTP POST
* put(url, data?, config?) — HTTP PUT
* patch(url, data?, config?) — HTTP PATCH
* del(url, config?) — HTTP DELETE
* download(url, destination) — download a file
* inspect(data, extension?) — dump data to a file and open it

Automation
* exec(command, options?) — run a shell command
* browse(url) — open in browser
* edit(filePath) — open in external editor
* clipboard.readText() — read clipboard text
* clipboard.writeText(text) — write clipboard text
* clipboard.readImage() — read image buffer
* clipboard.writeImage(buffer) — write image buffer
* getClipboardHistory() — clipboard history items
* removeClipboardItem(id) — remove one history item
* clearClipboardHistory() — clear history
* keyboard.type(...textOrKeys) — type (use with caution)
* mouse.move(points) — move mouse (use with caution)

AI
* ai(systemPrompt, options?) — returns (input) => text
* ai.object(promptOrMessages, schema, options?) — structured output via zod
* assistant(systemPrompt, options?) — multi-turn AI w/ tool calling
* generate(promptOrMessages, schema, options?) — structured generation
* mcp(options?) — MCP client

Handy Globals
* isMac — OS boolean
* isWin — OS boolean
* isLinux — OS boolean
* cmd — "cmd" on macOS, "ctrl" elsewhere
* args — CLI args array
* flag — parsed CLI flags

FINAL CHECKLIST
* Only TypeScript source code.
* Includes // Name: and // Description: at top.
* Exactly one import: import "@scriptkit/sdk";
* Top-level await + Script Kit globals.
* Interactive UX (prompts, previews, actions) instead of console output.
* Practical errors + safe cancellation."#;

#[derive(Debug, Clone)]
pub struct GeneratedScriptOutput {
    pub path: PathBuf,
    pub slug: String,
    pub model_id: String,
    pub provider_id: String,
    pub shell_execution_warning: bool,
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
    tracing::info!(
        target: "ai",
        correlation_id = "ai-script-generation",
        state = "provider_ready",
        model_id = %selected_model.id,
        provider_id = %selected_model.provider,
        prompt_len = normalized_prompt.len(),
        "Script generation provider ready"
    );

    let messages = build_script_generation_messages(normalized_prompt);

    let raw_response = provider
        .send_message(&messages, &selected_model.id)
        .with_context(|| {
            format!(
                "AI script generation failed (attempted=send_message, model_id={}, provider_id={})",
                selected_model.id, selected_model.provider
            )
        })?;

    let (slug, finalized) = prepare_script_from_ai_response(normalized_prompt, &raw_response)?;
    let suspicious_shell_patterns =
        detect_unexpected_shell_execution_patterns(normalized_prompt, &finalized);
    let shell_execution_warning = !suspicious_shell_patterns.is_empty();
    if shell_execution_warning {
        tracing::warn!(
            correlation_id = "ai-script-generation",
            state = "suspicious_shell_pattern_detected",
            patterns = ?suspicious_shell_patterns,
            model_id = %selected_model.id,
            provider_id = %selected_model.provider,
            "AI-generated script includes shell execution patterns without explicit shell intent"
        );
    }

    let path = crate::script_creation::create_new_script(&slug).with_context(|| {
        format!(
            "Failed creating AI-generated script (state=create_failed, slug={})",
            slug
        )
    })?;

    fs::write(&path, &finalized).with_context(|| {
        format!(
            "Failed writing AI-generated script content (state=write_failed, path={})",
            path.display()
        )
    })?;

    tracing::info!(
        target: "ai",
        correlation_id = "ai-script-generation",
        state = "script_written",
        path = %path.display(),
        slug = %slug,
        "AI-generated script written"
    );

    Ok(GeneratedScriptOutput {
        path,
        slug,
        model_id: selected_model.id,
        provider_id: selected_model.provider,
        shell_execution_warning,
    })
}

pub(crate) fn prepare_script_from_ai_response(
    prompt: &str,
    raw_response: &str,
) -> Result<(String, String)> {
    let normalized_prompt = prompt.trim();
    if normalized_prompt.is_empty() {
        anyhow::bail!("AI script generation requires a non-empty prompt");
    }

    let extracted = extract_script_code(raw_response);
    if extracted.trim().is_empty() {
        anyhow::bail!("AI returned an empty response for script generation (state=empty_response)");
    }

    // Prefer the AI's // Name: as the slug source — it's concise and descriptive.
    // Fall back to the prompt only if the AI didn't include one.
    let slug_source =
        extract_name_comment(&extracted).unwrap_or_else(|| normalized_prompt.to_string());
    let slug = slugify_script_name(&slug_source);
    let finalized = enforce_script_kit_conventions(&extracted, normalized_prompt, &slug);
    Ok((slug, finalized))
}

pub(crate) fn save_generated_script_from_response(
    prompt: &str,
    raw_response: &str,
) -> Result<PathBuf> {
    let (slug, script_source) = prepare_script_from_ai_response(prompt, raw_response)?;
    let script_path = crate::script_creation::create_new_script(&slug).with_context(|| {
        format!(
            "Failed to create script for AI response (state=create_failed, slug={})",
            slug
        )
    })?;

    fs::write(&script_path, script_source).with_context(|| {
        format!(
            "Failed writing script for AI response (state=write_failed, path={})",
            script_path.display()
        )
    })?;

    Ok(script_path)
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

fn build_script_generation_messages(normalized_prompt: &str) -> Vec<ProviderMessage> {
    vec![
        ProviderMessage::system(AI_SCRIPT_GENERATION_SYSTEM_PROMPT),
        ProviderMessage::user(format!(
            "Generate a Script Kit script for this user request:\n\n{}\n{}\n{}",
            AI_SCRIPT_USER_REQUEST_START_DELIMITER,
            normalized_prompt,
            AI_SCRIPT_USER_REQUEST_END_DELIMITER
        )),
    ]
}

fn prompt_allows_shell_execution(prompt: &str) -> bool {
    prompt
        .split(|character: char| !character.is_ascii_alphanumeric() && character != '_')
        .filter(|token| !token.is_empty())
        .any(|token| {
            let normalized_token = token.to_ascii_lowercase();
            normalized_token.starts_with("shell")
                || normalized_token.starts_with("exec")
                || normalized_token.starts_with("command")
                || normalized_token.starts_with("terminal")
                || normalized_token.starts_with("process")
        })
}

fn detect_shell_execution_patterns(script_source: &str) -> Vec<&'static str> {
    let normalized_tokens = script_source
        .split(|character: char| !character.is_ascii_alphanumeric() && character != '_')
        .filter(|token| !token.is_empty())
        .map(|token| token.to_ascii_lowercase())
        .collect::<Vec<_>>();

    AI_SCRIPT_SHELL_EXECUTION_PATTERNS
        .iter()
        .filter_map(|(pattern_name, normalized_pattern)| {
            normalized_tokens
                .iter()
                .any(|token| token == normalized_pattern)
                .then_some(*pattern_name)
        })
        .collect()
}

fn detect_unexpected_shell_execution_patterns(
    prompt: &str,
    script_source: &str,
) -> Vec<&'static str> {
    if prompt_allows_shell_execution(prompt) {
        return Vec::new();
    }

    detect_shell_execution_patterns(script_source)
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
        .unwrap_or_else(|| strip_leading_prose(response.trim()))
}

/// Strip leading non-TypeScript prose from an AI response that wasn't fenced.
/// Looks for the first line that starts with a valid TS/JS construct and drops everything before it.
fn strip_leading_prose(response: &str) -> String {
    let lines: Vec<&str> = response.lines().collect();

    // Find the first line that looks like TypeScript/JavaScript code
    let code_start = lines.iter().position(|line| {
        let trimmed = line.trim();
        trimmed.starts_with("//")
            || trimmed.starts_with("import ")
            || trimmed.starts_with("import{")
            || trimmed.starts_with("import\"")
            || trimmed.starts_with("import'")
            || trimmed.starts_with("export ")
            || trimmed.starts_with("const ")
            || trimmed.starts_with("let ")
            || trimmed.starts_with("var ")
            || trimmed.starts_with("async ")
            || trimmed.starts_with("await ")
            || trimmed.starts_with("function ")
            || trimmed.starts_with("type ")
            || trimmed.starts_with("interface ")
            || trimmed.starts_with("class ")
            || trimmed.starts_with("enum ")
    });

    match code_start {
        Some(0) => response.to_string(),
        Some(idx) => {
            let stripped = lines[idx..].join("\n");
            tracing::warn!(
                category = "AI",
                stripped_lines = idx,
                first_stripped_line = lines[0],
                "Stripped leading prose from AI script response"
            );
            stripped
        }
        None => response.to_string(),
    }
}

/// Extract the value from a `// Name: <value>` comment line in the script source.
fn extract_name_comment(script: &str) -> Option<String> {
    script
        .lines()
        .find_map(|line| line.trim().strip_prefix("// Name:"))
        .map(|name| name.trim().to_string())
        .filter(|name| !name.is_empty())
}

fn slugify_script_name(prompt: &str) -> String {
    let mut slug = String::new();
    let mut last_was_hyphen = false;

    for character in prompt.to_ascii_lowercase().chars() {
        if character.is_ascii_alphanumeric() {
            slug.push(character);
            last_was_hyphen = false;
        } else if matches!(character, ' ' | '_' | '-') && !slug.is_empty() && !last_was_hyphen {
            slug.push('-');
            last_was_hyphen = true;
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
        .join(" ")
}

fn description_from_prompt(prompt: &str) -> String {
    let normalized = prompt.split_whitespace().join(" ");
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
    script.lines().any(|line| {
        let trimmed = line.trim();
        trimmed.starts_with("import")
            && trimmed.contains(SCRIPT_KIT_SDK_IMPORT_MODULE)
            && (trimmed.contains('\"') || trimmed.contains('\''))
    })
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
        prefix_lines.push(SCRIPT_KIT_SDK_IMPORT_STATEMENT.to_string());
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
    fn test_build_script_generation_messages_wraps_prompt_with_request_delimiters() {
        let messages = build_script_generation_messages("show today's weather");
        assert_eq!(messages.len(), 2);
        assert_eq!(messages[1].role, "user");
        assert!(messages[1]
            .content
            .contains(AI_SCRIPT_USER_REQUEST_START_DELIMITER));
        assert!(messages[1]
            .content
            .contains(AI_SCRIPT_USER_REQUEST_END_DELIMITER));
        assert!(messages[1]
            .content
            .contains("---USER_REQUEST---\nshow today's weather\n---END_REQUEST---"));
    }

    #[test]
    fn test_detect_unexpected_shell_execution_patterns_returns_patterns_when_prompt_disallows_shell(
    ) {
        let prompt = "Show CPU usage in a rich UI";
        let script_source = r#"
import { execSync } from "child_process";
await div(execSync("top -l 1").toString());
"#;

        let patterns = detect_unexpected_shell_execution_patterns(prompt, script_source);
        assert_eq!(patterns, vec!["child_process", "execSync"]);
    }

    #[test]
    fn test_detect_unexpected_shell_execution_patterns_returns_empty_when_prompt_allows_shell() {
        let prompt = "Run a shell command in the terminal and show output";
        let script_source = r#"
import { execSync } from "child_process";
await div(execSync("pwd").toString());
"#;

        let patterns = detect_unexpected_shell_execution_patterns(prompt, script_source);
        assert!(patterns.is_empty());
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
        assert!(output.contains("import \"@scriptkit/sdk\";"));
        assert!(output.contains("await arg(\"Name?\");"));
    }

    #[test]
    fn test_enforce_script_kit_conventions_keeps_existing_metadata_and_import() {
        let script = r#"// Name: Existing
// Description: Existing description
import "@scriptkit/sdk";

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
            output.matches("import \"@scriptkit/sdk\";").count(),
            1,
            "should not duplicate existing import"
        );
    }

    #[test]
    fn test_has_kit_import_accepts_scriptkit_sdk_and_rejects_legacy_kit_import() {
        assert!(has_kit_import("import \"@scriptkit/sdk\";"));
        assert!(has_kit_import("import '@scriptkit/sdk'"));
        assert!(!has_kit_import("import \"@johnlindquist/kit\";"));
    }

    #[test]
    fn test_ai_script_generation_system_prompt_uses_modern_sdk_conventions() {
        assert!(AI_SCRIPT_GENERATION_SYSTEM_PROMPT.contains("import \"@scriptkit/sdk\";"));
        assert!(!AI_SCRIPT_GENERATION_SYSTEM_PROMPT.contains("@johnlindquist/kit"));
        assert!(AI_SCRIPT_GENERATION_SYSTEM_PROMPT.contains("arg("));
        assert!(AI_SCRIPT_GENERATION_SYSTEM_PROMPT.contains("div("));
        assert!(AI_SCRIPT_GENERATION_SYSTEM_PROMPT.contains("editor("));
        assert!(AI_SCRIPT_GENERATION_SYSTEM_PROMPT.contains("notify("));
        // New prompt includes examples and comprehensive API reference
        assert!(AI_SCRIPT_GENERATION_SYSTEM_PROMPT.contains("TEACH BY EXAMPLE"));
        assert!(AI_SCRIPT_GENERATION_SYSTEM_PROMPT.contains("COMPACT API REFERENCE"));
        assert!(AI_SCRIPT_GENERATION_SYSTEM_PROMPT.contains("ai("));
        assert!(AI_SCRIPT_GENERATION_SYSTEM_PROMPT.contains("clipboard"));
        assert!(AI_SCRIPT_GENERATION_SYSTEM_PROMPT.contains("home("));
    }

    #[test]
    fn test_prepare_script_from_ai_response_adds_conventions_when_ai_omits_them() {
        let prompt = "Create a weather checker";
        let response = "await div(\"Sunny\");";

        let (slug, source) = prepare_script_from_ai_response(prompt, response).unwrap();
        assert_eq!(slug, "create-a-weather-checker");
        assert!(source.contains("// Name: Create A Weather Checker"));
        assert!(source.contains("// Description: Create a weather checker"));
        assert!(source.contains("import \"@scriptkit/sdk\";"));
        assert!(source.contains("await div(\"Sunny\");"));
    }

    #[test]
    fn test_prepare_script_from_ai_response_uses_name_comment_for_slug() {
        let prompt =
            "Generate a Script Kit script that automates what I am doing in the current app";
        let response = r#"// Name: cmux Quick Actions
// Description: Quick action palette for cmux
import "@scriptkit/sdk";

await arg("Pick an action");
"#;
        let (slug, source) = prepare_script_from_ai_response(prompt, response).unwrap();
        assert_eq!(
            slug, "cmux-quick-actions",
            "slug should come from // Name:, not the prompt"
        );
        assert!(source.contains("// Name: cmux Quick Actions"));
    }

    #[test]
    fn test_extract_name_comment_finds_name_line() {
        assert_eq!(
            extract_name_comment("// Name: My Cool Script\nimport \"@scriptkit/sdk\";"),
            Some("My Cool Script".to_string())
        );
        assert_eq!(
            extract_name_comment("import \"@scriptkit/sdk\";\nawait arg(\"hi\");"),
            None
        );
        assert_eq!(
            extract_name_comment("// Name: \nimport \"@scriptkit/sdk\";"),
            None,
            "empty name should return None"
        );
    }

    #[test]
    fn test_prepare_script_from_ai_response_extracts_typescript_fence_when_present() {
        let prompt = "Build script";
        let response = r#"
```typescript
await arg("Name?");
```
"#;

        let (_slug, source) = prepare_script_from_ai_response(prompt, response).unwrap();
        assert!(source.contains("await arg(\"Name?\");"));
        assert!(!source.contains("```"));
    }

    #[test]
    fn test_strip_leading_prose_removes_markdown_preamble() {
        let response = r#"**Assumed:** You're using cmux and want a quick-action palette.

**Required permissions:** Accessibility access for Script Kit.

// Name: cmux Quick Actions
// Description: Quick action palette for cmux
import "@scriptkit/sdk";

await arg("Pick an action");
"#;
        let stripped = strip_leading_prose(response.trim());
        assert!(
            stripped.starts_with("// Name:"),
            "Should start with // Name:, got: {}",
            &stripped[..stripped.len().min(50)]
        );
        assert!(!stripped.contains("**Assumed:**"));
        assert!(stripped.contains("await arg"));
    }

    #[test]
    fn test_strip_leading_prose_preserves_clean_response() {
        let response = r#"// Name: My Script
// Description: Does something
import "@scriptkit/sdk";

await div("hello");
"#;
        let stripped = strip_leading_prose(response.trim());
        assert!(stripped.starts_with("// Name:"));
        assert!(stripped.contains("await div"));
    }

    #[test]
    fn test_extract_script_code_strips_prose_when_no_fence() {
        let response = r#"Here's a script for you:

// Name: Test
import "@scriptkit/sdk";

await arg("hello");
"#;
        let extracted = extract_script_code(response);
        assert!(
            extracted.starts_with("// Name:"),
            "Should start with code, got: {}",
            &extracted[..extracted.len().min(50)]
        );
        assert!(!extracted.contains("Here's a script"));
    }
}
