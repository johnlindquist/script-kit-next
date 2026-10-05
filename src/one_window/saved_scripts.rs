//! Every run that did several things is kept as an entry in one TypeScript file,
//! `~/.scriptkit/plugins/main/scripts/saved.ts`, and "Open “…” in saved.ts" shows that entry in
//! the main window under the same input.

use std::path::PathBuf;

use gpui::{div, prelude::*, px, rgb, AnyElement, Context};
use sk_one_window::contract::SAVED_SCRIPT_FILE;
use sk_one_window::EnterTarget;

use crate::intent::{plan_to_script, ScriptOptions};
use crate::list_item::ListItemColors;
use crate::ScriptListApp;

const FILE_HEADER: &str = "// Saved runs. Every sentence that did several things is kept here as \
a script you can run,\n// change or share. Script Kit adds to this file; edit any entry freely.\n\
import \"@scriptkit/sdk\";\n";
const NAME_PREFIX: &str = "// Name: ";

/// The saved entry the script view shows.
#[derive(Clone, Debug)]
pub(crate) struct Shown {
    pub(crate) name: String,
    pub(crate) source: String,
}

pub(crate) fn saved_path() -> PathBuf {
    crate::setup::get_kit_path().join("plugins/main/scripts/saved.ts")
}

/// A function name for an entry: "Tell Sarah + Post in #launcher" → `tellSarahPostInLauncher`.
pub(super) fn function_name(name: &str) -> String {
    let mut out = String::new();
    for (i, word) in name
        .split(|c: char| !c.is_ascii_alphanumeric())
        .filter(|w| !w.is_empty())
        .enumerate()
    {
        let mut chars = word.chars();
        if let Some(first) = chars.next() {
            if i == 0 {
                out.extend(first.to_lowercase());
            } else {
                out.extend(first.to_uppercase());
            }
            out.push_str(&chars.as_str().to_lowercase());
        }
    }
    if out.chars().next().is_none_or(|c| c.is_ascii_digit()) {
        out.insert_str(0, "run");
    }
    out
}

/// One exported entry: its name and sentence as comments, then the steps as a function.
pub(crate) fn entry_source(name: &str, said: &str, body: &[String]) -> String {
    let mut lines = vec![
        format!("{NAME_PREFIX}{name}"),
        format!("// Said: {said}"),
        format!("export async function {}() {{", function_name(name)),
    ];
    lines.extend(body.iter().map(|line| {
        if line.is_empty() {
            String::new()
        } else {
            format!("  {line}")
        }
    }));
    lines.push("}".to_string());
    lines.join("\n")
}

/// The file with `entry` added, replacing an entry of the same name, and `imports` hoisted.
pub(crate) fn with_entry(file: &str, name: &str, entry: &str, imports: &[String]) -> String {
    let body = if file.trim().is_empty() {
        FILE_HEADER
    } else {
        file
    };
    let mut head: Vec<String> = Vec::new();
    let mut entries: Vec<(String, String)> = Vec::new();
    for line in body.lines() {
        if let Some(entry_name) = line.strip_prefix(NAME_PREFIX) {
            entries.push((entry_name.to_string(), line.to_string()));
        } else if let Some((_, text)) = entries.last_mut() {
            text.push('\n');
            text.push_str(line);
        } else {
            head.push(line.to_string());
        }
    }
    for import in imports {
        if !head.contains(import) {
            head.push(import.clone());
        }
    }
    entries.retain(|(entry_name, _)| entry_name != name);
    entries.push((name.to_string(), entry.to_string()));
    let mut out = head.join("\n").trim_end().to_string();
    for (_, text) in entries {
        out.push_str("\n\n");
        out.push_str(text.trim_end());
    }
    out.push('\n');
    out
}

/// The entry named `name` in the file.
pub(crate) fn entry_named(file: &str, name: &str) -> Option<String> {
    let start = file.find(&format!("{NAME_PREFIX}{name}\n"))?;
    let rest = &file[start..];
    let end = rest[NAME_PREFIX.len()..]
        .find(&format!("\n{NAME_PREFIX}"))
        .map(|ix| ix + NAME_PREFIX.len())
        .unwrap_or(rest.len());
    Some(rest[..end].trim_end().to_string())
}

pub(crate) fn entry_names(file: &str) -> Vec<String> {
    file.lines()
        .filter_map(|line| line.strip_prefix(NAME_PREFIX))
        .map(str::to_string)
        .collect()
}

/// A draft script's steps without its imports and metadata, and its imports.
fn draft_body(source: &str) -> (Vec<String>, Vec<String>) {
    let mut imports = Vec::new();
    let mut body = Vec::new();
    let mut in_metadata = false;
    let mut past_metadata = false;
    for line in source.lines() {
        if !past_metadata {
            if line.starts_with("import ") {
                if line != "import \"@scriptkit/sdk\";" {
                    imports.push(line.to_string());
                }
                continue;
            }
            if line.starts_with("export const metadata") {
                in_metadata = true;
                continue;
            }
            if in_metadata {
                if line.starts_with("};") {
                    in_metadata = false;
                    past_metadata = true;
                }
                continue;
            }
            if line.trim().is_empty() {
                continue;
            }
            past_metadata = true;
        }
        body.push(line.to_string());
    }
    while body.last().is_some_and(|l| l.trim().is_empty()) {
        body.pop();
    }
    (body, imports)
}

impl ScriptListApp {
    pub(crate) fn one_window_saved_script_names(&self) -> Vec<String> {
        entry_names(&self.one_window_host.learns.saved_file)
    }

    /// The saved entry called `name`, as this session knows saved.ts.
    pub(crate) fn one_window_saved_entry(&self, name: &str) -> Option<String> {
        entry_named(&self.one_window_host.learns.saved_file, name)
    }

    /// The reducer saved a run as `name`: its entry goes into saved.ts.
    /// Saves the run as a script in `saved.ts`. False when it couldn't be written to disk.
    pub(crate) fn one_window_save_script(&mut self, name: String, said: String) -> bool {
        let plan = self
            .one_window_last_run()
            .and_then(|row| self.intent_launcher.plan(&said, &row.key));
        let (body, imports) = match plan {
            Some(plan) => draft_body(
                &plan_to_script(
                    &plan,
                    &ScriptOptions {
                        name: Some(name.clone()),
                        parameterize: Some(false),
                        ..ScriptOptions::default()
                    },
                )
                .source,
            ),
            None => (
                vec![
                    "// Jev's steps for this run weren't kept; the sentence above is what ran."
                        .to_string(),
                ],
                Vec::new(),
            ),
        };
        let next = with_entry(
            &self.one_window_host.learns.saved_file,
            &name,
            &entry_source(&name, &said, &body),
            &imports,
        );
        self.one_window_host.learns.saved_file = next.clone();
        if !self.main_services.is_production() {
            return true;
        }
        let path = saved_path();
        let written = (|| {
            if let Some(dir) = path.parent() {
                std::fs::create_dir_all(dir)?;
            }
            std::fs::write(&path, next)
        })();
        match written {
            Ok(()) => {
                tracing::info!(target: "script_kit::one_window", event = "script_saved", name = %name);
                true
            }
            Err(error) => {
                tracing::warn!(target: "script_kit::one_window", %error, "script_not_saved");
                false
            }
        }
    }

    /// Shows the saved entry `name` under the input.
    pub(crate) fn one_window_open_script(&mut self, name: String, cx: &mut Context<Self>) {
        let source = self.one_window_saved_entry(&name).unwrap_or_default();
        self.one_window_host.learns.script = Some(Shown {
            name: name.clone(),
            source,
        });
        self.one_window_open_view(
            super::learns::OwnView::Script,
            name,
            Some(EnterTarget::new("script:edit", "Edit saved.ts")),
            false,
            cx,
        );
    }

    pub(crate) fn one_window_script_enter(&mut self, _cx: &mut Context<Self>) {
        self.edit_script(&saved_path());
    }

    /// The script view's list: the file's path as the header, the entry as its one row.
    pub(crate) fn one_window_script_rows(&self) -> Option<(String, Vec<sk_one_window::Row>)> {
        let shown = self.one_window_host.learns.script.as_ref()?;
        let lines = shown.source.lines().count();
        let row = sk_one_window::Row::new(format!("saved:{}", shown.name), shown.name.clone())
            .with_sub(format!(
                "{lines} lines · one of the runs saved in this file"
            ))
            .with_icon("script")
            .with_enter_label("Edit saved.ts");
        Some((SAVED_SCRIPT_FILE.to_string(), vec![row]))
    }

    /// The saved entry's source, under the script view's row, in the mono font.
    pub(crate) fn one_window_script_element(&self, colors: ListItemColors) -> Option<AnyElement> {
        let shown = self.one_window_host.learns.script.as_ref()?;
        let fonts = self.theme.get_fonts();
        let def = self.current_main_menu_theme.def();
        let inset = def.header_info_bar.context_edge_outset_x
            + def.shell.header_padding_x
            + def.search.text_inset_x;
        Some(
            div()
                .id("one-window-script-source")
                .debug_selector(|| "one-window-script-source".to_string())
                .flex_1()
                .min_h(px(0.))
                .overflow_y_scroll()
                .px(px(inset))
                .font_family(fonts.mono_family.clone())
                .text_size(px(fonts.mono_size))
                .text_color(rgb(colors.text_primary))
                .children(shown.source.lines().map(|line| {
                    let comment = line.trim_start().starts_with("//");
                    div()
                        .whitespace_nowrap()
                        .when(comment, |d| d.text_color(rgb(colors.text_muted)))
                        .child(if line.is_empty() {
                            " ".to_string()
                        } else {
                            line.to_string()
                        })
                }))
                .into_any_element(),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn entries_are_named_functions_in_one_file() {
        let a = entry_source(
            "Tell Sarah + Post in #launcher",
            "tell sarah hi",
            &["await x()".into()],
        );
        let file = with_entry("", "Tell Sarah + Post in #launcher", &a, &[]);
        assert!(file.starts_with("// Saved runs."));
        assert!(file.contains("export async function tellSarahPostInLauncher() {\n  await x()\n}"));
        let b = entry_source("Move Standup", "move standup", &["await y()".into()]);
        let file = with_entry(
            &file,
            "Move Standup",
            &b,
            &["import { z } from \"zod\";".into()],
        );
        assert_eq!(
            entry_names(&file),
            ["Tell Sarah + Post in #launcher", "Move Standup"]
        );
        assert!(file.contains("import { z } from \"zod\";"));
        let again = entry_source("Move Standup", "move standup", &["await w()".into()]);
        let file = with_entry(&file, "Move Standup", &again, &[]);
        assert_eq!(entry_names(&file).len(), 2, "same name replaces");
        let entry = entry_named(&file, "Move Standup").unwrap();
        assert!(entry.contains("await w()") && !entry.contains("await y()"));
        assert!(entry_named(&file, "Tell Sarah + Post in #launcher")
            .unwrap()
            .contains("await x()"));
    }

    #[test]
    fn a_draft_keeps_its_steps() {
        let draft = "import \"@scriptkit/sdk\";\nimport { a } from \"b\";\n\nexport const metadata = {\n  name: \"X\",\n};\n\nawait one();\nawait two();\n";
        let (body, imports) = draft_body(draft);
        assert_eq!(body, ["await one();", "await two();"]);
        assert_eq!(imports, ["import { a } from \"b\";"]);
    }
}
