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
const SCRIPT_KIT_SDK_IMPORT_MODULE: &str = "@scriptkit/sdk";
const SCRIPT_KIT_SDK_IMPORT_STATEMENT: &str = "import \"@scriptkit/sdk\";";

pub(crate) const AI_SCRIPT_GENERATION_SYSTEM_PROMPT: &str = r#"You write production-ready Script Kit TypeScript scripts.

RETURN ONLY ONE THING: TypeScript source code for a single Script Kit script.

* No markdown fences. No explanations. No multiple options.

NON-NEGOTIABLE OUTPUT FORMAT

1. The first lines are:
   // Name: <short, clear, user-facing title>
   // Description: <one-line summary>
2. Near the top include EXACTLY ONE import (and no others):
   import "@scriptkit/sdk";
3. Use top-level await (no main(), no async IIFE, no servers).
4. Prefer Script Kit prompts + UI over console.log.

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

    let (slug, finalized) = prepare_script_from_ai_response(normalized_prompt, &raw_response)?;
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

    let slug = slugify_script_name(normalized_prompt);
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
    fn test_write_generated_script_in_dir_appends_numeric_suffix_for_collisions() {
        let temp_dir = tempdir().unwrap();
        let first = write_generated_script_in_dir(temp_dir.path(), "my-script", "a").unwrap();
        let second = write_generated_script_in_dir(temp_dir.path(), "my-script", "b").unwrap();

        assert_eq!(first.file_name().unwrap(), "my-script.ts");
        assert_eq!(second.file_name().unwrap(), "my-script-1.ts");
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
}
