//! Capture-handler scaffolding templates.
//!
//! Oracle iter 007 commit 6: produce a `.ts` scaffold that a user can drop
//! under `~/.scriptkit/plugins/main/scripts/` so `;<target>` gets a new
//! handler without copy-pasting a shipped example. The output mirrors the
//! shape of the shipped examples in `scripts/examples/menu-syntax/`:
//!
//! - Metadata header with `menuSyntax: [{ family: "capture.v1", targets: [...] }]`
//! - `KIT_MENU_SYNTAX_PAYLOAD_PATH` env reader that parses the JSON payload
//! - `SK_PATH` fallback to `~/.scriptkit` for local-first artifacts
//!
//! This module is pure: it does not touch the filesystem, open an editor,
//! or reach into GPUI. The authoring flow (writing the file, opening it,
//! wiring the Cmd+N picker action) is a separate concern that ships in a
//! later tick with live interactive testing.

use super::payload::is_known_capture_target;

/// Render a TypeScript capture-handler scaffold for `target` with filename
/// hint `slug`. The returned string is pure `.ts` source and can be written
/// directly to disk by the caller. Unknown targets still render a valid
/// scaffold — the `menuSyntax` block simply pins the target as-typed, and
/// the parser will ignore it until the target is registered.
///
/// `slug` is echoed in the handler's human-readable name so two handlers for
/// the same target (`capture-todo-inbox.ts`, `capture-todo-jira.ts`) stay
/// distinguishable in the picker.
pub fn render_capture_handler_template(target: &str, slug: &str) -> String {
    let target_slug = slug_or_target(target, slug);
    let handler_name = display_name_from_slug(target, &target_slug);
    let artifact_hint = artifact_hint_for(target);
    let accepts = accepts_hint_for(target);

    format!(
        r##"// capture-{target_slug}.ts
// Auto-scaffolded by Script Kit — menu-syntax capture handler.
//
// This handler fires whenever the launcher sees a `;{target}` or `{target}:`
// menu-syntax invocation. Script Kit writes the parsed payload to a JSON
// tempfile and passes its path through the `KIT_MENU_SYNTAX_PAYLOAD_PATH`
// env var. See lat.md/menu-syntax.md#Execution Payload for the contract.
//
// Edit this file to decide what to do with the captured payload. The body
// is intentionally a local-first example: it appends a JSONL line under
// `$SK_PATH/menu-syntax/` (defaulting to `~/.scriptkit/menu-syntax/`).

import {{ mkdir, appendFile, readFile }} from "node:fs/promises";
import {{ join }} from "node:path";

export const metadata = {{
  name: "{handler_name}",
  description: "Handle ;{target} menu-syntax captures ({target_slug}).",
  menuSyntax: [
    {{
      family: "capture.v1",
      targets: ["{target}"],
      accepts: {accepts},
      label: "{handler_name}",
      payloadSchema: "kit://schema/menu-syntax/payload-v1",
      // Set defaultHandler to true to make this the preferred handler for
      // ;{target}. Only set it on ONE handler per target — the ranker uses
      // it to place a row at the very top of the capture picker.
      defaultHandler: false,
    }},
  ],
}};

const payloadPath = process.env.KIT_MENU_SYNTAX_PAYLOAD_PATH;
if (!payloadPath) {{
  throw new Error(
    "KIT_MENU_SYNTAX_PAYLOAD_PATH is required — did Script Kit launch this script?",
  );
}}

const payload = JSON.parse(await readFile(payloadPath, "utf8"));
const skPath = process.env.SK_PATH || join(process.env.HOME || ".", ".scriptkit");

const dir = join(skPath, "menu-syntax");
await mkdir(dir, {{ recursive: true }});

await appendFile(
  join(dir, "{artifact_hint}"),
  JSON.stringify({{
    target: "{target}",
    body: payload.body,
    tags: payload.tags,
    priority: payload.priority,
    url: payload.url ?? null,
    duration: payload.duration ?? null,
    dates: payload.dates ?? [],
    raw: payload.raw,
    createdAt: new Date().toISOString(),
  }}) + "\n",
);
"##
    )
}

/// If the caller passes an empty or whitespace-only slug, use the target as
/// a fallback slug so the scaffold stays parseable. Otherwise normalize the
/// slug by lowercasing and replacing non-alphanumeric runs with dashes.
fn slug_or_target(target: &str, slug: &str) -> String {
    let trimmed = slug.trim();
    let source = if trimmed.is_empty() { target } else { trimmed };
    let lower = source.to_ascii_lowercase();
    let mut out = String::with_capacity(lower.len());
    let mut prev_dash = false;
    for ch in lower.chars() {
        if ch.is_ascii_alphanumeric() {
            out.push(ch);
            prev_dash = false;
        } else if !prev_dash {
            out.push('-');
            prev_dash = true;
        }
    }
    let trimmed = out.trim_matches('-').to_string();
    if trimmed.is_empty() {
        target.to_ascii_lowercase()
    } else {
        trimmed
    }
}

fn display_name_from_slug(target: &str, slug: &str) -> String {
    let mut out = String::new();
    for (idx, part) in slug.split('-').enumerate() {
        if part.is_empty() {
            continue;
        }
        if idx > 0 && !out.is_empty() {
            out.push(' ');
        }
        let mut chars = part.chars();
        if let Some(first) = chars.next() {
            out.extend(first.to_uppercase());
            out.push_str(chars.as_str());
        }
    }
    if out.is_empty() {
        format!("Capture {target}")
    } else {
        format!("Capture {out}")
    }
}

fn artifact_hint_for(target: &str) -> &'static str {
    if !is_known_capture_target(target) {
        return "entries.jsonl";
    }
    match target.to_ascii_lowercase().as_str() {
        "todo" => "todos.jsonl",
        "cal" => "events.jsonl",
        "note" => "notes.jsonl",
        "social" => "drafts.jsonl",
        "link" => "bookmarks.jsonl",
        _ => "entries.jsonl",
    }
}

fn accepts_hint_for(target: &str) -> &'static str {
    match target.to_ascii_lowercase().as_str() {
        "todo" => r#"["tags", "date", "priority", "url", "kv"]"#,
        "cal" => r#"["date", "duration", "tags", "kv"]"#,
        "note" => r#"["tags", "date", "kv"]"#,
        "social" => r#"["tags", "url", "kv"]"#,
        "link" => r#"["url", "tags", "kv"]"#,
        _ => r#"["tags", "date", "url", "kv"]"#,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn template_contains_menu_syntax_metadata_block() {
        let out = render_capture_handler_template("todo", "custom");
        assert!(out.contains("menuSyntax"), "should declare menuSyntax");
        assert!(
            out.contains("capture.v1"),
            "should pin the capture.v1 family"
        );
        assert!(
            out.contains(r#"targets: ["todo"]"#),
            "should carry the target list"
        );
    }

    #[test]
    fn generated_capture_handler_template_executes_with_payload_env_and_writes_jsonl() {
        use serde_json::json;
        use std::process::Command;

        let tmp = tempfile::TempDir::new().expect("tempdir");
        let handler_path = tmp.path().join("capture-note-handler.mjs");
        let runner_path = tmp.path().join("run-generated-handler.mjs");
        let payload_path = tmp.path().join("payload.json");

        std::fs::write(
            &handler_path,
            render_capture_handler_template("note", "daily"),
        )
        .expect("write generated handler");
        std::fs::write(
            &runner_path,
            r#"import { pathToFileURL } from "node:url";
const mod = await import(pathToFileURL(process.argv[2]).href);
console.log(JSON.stringify(mod.metadata));
"#,
        )
        .expect("write runner");
        std::fs::write(
            &payload_path,
            serde_json::to_string(&json!({
                "body": "Daily note",
                "tags": ["journal"],
                "priority": 2,
                "url": "https://example.test/note",
                "duration": "15m",
                "dates": [{ "role": "due", "source": "tomorrow" }],
                "raw": ";note Daily note #journal p2"
            }))
            .expect("serialize payload"),
        )
        .expect("write payload");

        let runtime =
            std::env::var("MENU_SYNTAX_TEST_JS_RUNTIME").unwrap_or_else(|_| "node".to_string());
        let output = Command::new(runtime)
            .arg(&runner_path)
            .arg(&handler_path)
            .env("KIT_MENU_SYNTAX_PAYLOAD_PATH", &payload_path)
            .env("SK_PATH", tmp.path())
            .output()
            .expect("run generated handler");
        assert!(
            output.status.success(),
            "generated handler failed\nstdout:\n{}\nstderr:\n{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );

        let metadata: serde_json::Value =
            serde_json::from_slice(&output.stdout).expect("metadata stdout must be JSON");
        assert_eq!(metadata["menuSyntax"][0]["family"], "capture.v1");
        assert_eq!(metadata["menuSyntax"][0]["targets"], json!(["note"]));
        assert_eq!(
            metadata["menuSyntax"][0]["accepts"],
            json!(["tags", "date", "kv"])
        );
        assert_eq!(metadata["menuSyntax"][0]["defaultHandler"], false);

        let artifact_path = tmp.path().join("menu-syntax").join("notes.jsonl");
        let artifact = std::fs::read_to_string(&artifact_path).expect("read notes.jsonl");
        let lines: Vec<_> = artifact.lines().collect();
        assert_eq!(lines.len(), 1, "expected one JSONL row: {artifact}");
        let row: serde_json::Value =
            serde_json::from_str(lines[0]).expect("artifact row must be JSON");
        assert_eq!(row["target"], "note");
        assert_eq!(row["body"], "Daily note");
        assert_eq!(row["tags"], json!(["journal"]));
        assert_eq!(row["priority"], 2);
        assert_eq!(row["url"], "https://example.test/note");
        assert_eq!(row["duration"], "15m");
        assert_eq!(
            row["dates"],
            json!([{ "role": "due", "source": "tomorrow" }])
        );
        assert_eq!(row["raw"], ";note Daily note #journal p2");
        assert!(row["createdAt"]
            .as_str()
            .is_some_and(|created_at| !created_at.is_empty()));
    }

    #[test]
    fn template_explains_when_the_handler_fires() {
        let out = render_capture_handler_template("todo", "inbox");
        assert!(
            out.contains("fires whenever the launcher sees"),
            "should document activation"
        );
    }

    #[test]
    fn template_defaults_default_handler_to_false_with_guidance() {
        // Prevents every scaffolded handler from fighting for the top slot.
        let out = render_capture_handler_template("todo", "inbox");
        assert!(
            out.contains("defaultHandler: false"),
            "scaffolds must not auto-claim the default slot"
        );
        assert!(
            out.contains("Only set it on ONE handler per target"),
            "scaffolds must teach the author why"
        );
    }

    #[test]
    fn template_renders_for_every_known_target() {
        for target in ["todo", "cal", "note", "social", "link"] {
            let out = render_capture_handler_template(target, "custom");
            assert!(
                out.contains(&format!(r#"targets: ["{target}"]"#)),
                "target `{target}` must be pinned in its scaffold"
            );
        }
    }

    #[test]
    fn template_core_target_artifact_hints_are_taxonomy_pins() {
        for (target, slug, filename) in [
            ("todo", "inbox", "todos.jsonl"),
            ("cal", "events", "events.jsonl"),
            ("note", "daily", "notes.jsonl"),
            ("social", "draft", "drafts.jsonl"),
            ("link", "bookmarks", "bookmarks.jsonl"),
        ] {
            assert!(
                render_capture_handler_template(target, slug).contains(filename),
                "`{target}` should scaffold to `{filename}`"
            );
        }
    }

    #[test]
    fn template_non_core_dynamic_target_uses_generic_artifact_until_registered() {
        let out = render_capture_handler_template("gcal", "x");
        assert!(
            out.contains("entries.jsonl"),
            "non-core dynamic target should fall back to a generic artifact filename"
        );
        assert!(
            out.contains(r#"targets: ["gcal"]"#),
            "dynamic target must still render verbatim"
        );
    }

    #[test]
    fn template_unknown_custom_target_uses_generic_artifact() {
        let out = render_capture_handler_template("custom-target", "x");
        assert!(
            out.contains("entries.jsonl"),
            "unknown target should fall back to a generic artifact filename"
        );
        assert!(
            out.contains(r#"targets: ["custom-target"]"#),
            "unknown target must still render verbatim"
        );
    }

    #[test]
    fn template_core_target_accepts_hints_are_taxonomy_pins() {
        for (target, accepts) in [
            (
                "todo",
                r#"accepts: ["tags", "date", "priority", "url", "kv"]"#,
            ),
            ("cal", r#"accepts: ["date", "duration", "tags", "kv"]"#),
            ("note", r#"accepts: ["tags", "date", "kv"]"#),
            ("social", r#"accepts: ["tags", "url", "kv"]"#),
            ("link", r#"accepts: ["url", "tags", "kv"]"#),
        ] {
            assert!(
                render_capture_handler_template(target, "x").contains(accepts),
                "`{target}` should scaffold accepts hint `{accepts}`"
            );
        }
    }

    #[test]
    fn slug_or_target_falls_back_to_target_when_empty() {
        assert_eq!(slug_or_target("todo", ""), "todo");
        assert_eq!(slug_or_target("todo", "   "), "todo");
        assert_eq!(slug_or_target("todo", "!@#"), "todo");
    }

    #[test]
    fn slug_or_target_normalizes_user_input() {
        assert_eq!(slug_or_target("todo", "My Custom Inbox"), "my-custom-inbox");
        assert_eq!(slug_or_target("todo", "Already-Kebab"), "already-kebab");
        assert_eq!(slug_or_target("todo", "___under__score"), "under-score");
    }

    #[test]
    fn template_name_derives_from_slug() {
        let out = render_capture_handler_template("todo", "jira sync");
        assert!(
            out.contains("Capture Jira Sync"),
            "handler name should be title-cased from slug: {out}"
        );
    }

    #[test]
    fn template_filename_header_uses_normalized_slug() {
        let out = render_capture_handler_template("todo", "My Custom Inbox");
        assert!(
            out.starts_with("// capture-my-custom-inbox.ts"),
            "first line should echo the normalized filename hint, got {}",
            &out[..60]
        );
    }

    #[test]
    fn template_payload_path_error_message_is_actionable() {
        let out = render_capture_handler_template("todo", "x");
        assert!(
            out.contains("did Script Kit launch this script?"),
            "runtime error should hint at the cause"
        );
    }
}
