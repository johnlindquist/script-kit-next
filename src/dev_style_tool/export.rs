use std::path::PathBuf;

use serde_json::json;

use super::{runtime_overrides, StyleValue, STYLE_KNOBS};

pub fn save_current_settings_markdown() -> anyhow::Result<PathBuf> {
    save_current_settings_markdown_with_contents().map(|(path, _contents)| path)
}

pub fn save_current_settings_markdown_with_contents() -> anyhow::Result<(PathBuf, String)> {
    let dir = dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join(".scriptkit")
        .join("dev-style-tool");
    std::fs::create_dir_all(&dir)?;
    let timestamp = chrono::Local::now().format("%Y%m%d-%H%M%S");
    let path = dir.join(format!("main-window-style-{timestamp}.md"));
    let contents = current_settings_markdown();
    std::fs::write(&path, &contents)?;
    Ok((path, contents))
}

pub fn current_settings_markdown() -> String {
    let export = current_settings_json();
    let pretty = serde_json::to_string_pretty(&export).expect("style export json must serialize");
    format!(
        "# Script Kit Main Window Style\n\nUse this export to reproduce or review dev style tool overrides.\n\n```json\n{pretty}\n```\n"
    )
}

pub fn current_settings_json() -> serde_json::Value {
    let base = crate::designs::current_main_menu_theme().base_def();
    let overrides: Vec<_> = STYLE_KNOBS
        .iter()
        .filter_map(|knob| {
            runtime_overrides::current_value(knob.id).map(|value| {
                let StyleValue::Number(value) = value;
                json!({
                    "id": knob.id.as_str(),
                    "label": knob.label,
                    "group": knob.group.label(),
                    "unit": knob.unit.label(),
                    "value": value,
                })
            })
        })
        .collect();
    let effective: Vec<_> = STYLE_KNOBS
        .iter()
        .map(|knob| {
            let StyleValue::Number(base_value) = (knob.get)(&base);
            let StyleValue::Number(value) =
                runtime_overrides::current_value(knob.id).unwrap_or(StyleValue::Number(base_value));
            json!({
                "id": knob.id.as_str(),
                "label": knob.label,
                "group": knob.group.label(),
                "unit": knob.unit.label(),
                "base": base_value,
                "value": value,
                "overridden": runtime_overrides::current_value(knob.id).is_some(),
            })
        })
        .collect();

    json!({
        "schema": "script-kit-main-window-style/v1",
        "generatedAt": chrono::Local::now().to_rfc3339(),
        "runtimeGeneration": runtime_overrides::generation(),
        "overrideCount": overrides.len(),
        "controls": STYLE_KNOBS.len(),
        "overrides": overrides,
        "effective": effective,
        "agentPrompt": "Apply or reason about these Script Kit GPUI main-window style overrides by matching each id to src/dev_style_tool/catalog.rs.",
    })
}

pub fn export_summary_for_path(path: &PathBuf) -> String {
    format!("Saved {}", path.display())
}
