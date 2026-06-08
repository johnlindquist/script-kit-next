use std::path::{Path, PathBuf};

use serde_json::json;

use super::{
    runtime_overrides, StyleValue, ACTIONS_POPUP_KNOBS, AGENT_CHAT_KNOBS, CONFIRM_MODAL_KNOBS,
    COPY_CONTROLS, STYLE_KNOBS,
};

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
    let path = dir.join(format!("dev-style-{timestamp}.md"));
    let contents = current_settings_markdown();
    std::fs::write(&path, &contents)?;
    Ok((path, contents))
}

pub fn current_settings_markdown() -> String {
    let export = current_settings_json();
    let pretty = serde_json::to_string_pretty(&export).unwrap_or_else(|error| {
        format!("{{\"error\":\"failed to serialize style export: {error}\"}}")
    });
    format!(
        "# Script Kit Dev Style\n\nUse this export to reproduce or review dev style tool overrides.\n\n```json\n{pretty}\n```\n"
    )
}

pub fn current_settings_json() -> serde_json::Value {
    let main_window_base = crate::designs::current_main_menu_theme().base_def();
    let actions_popup_base = crate::designs::base_actions_popup_theme();
    let agent_chat_base = super::agent_chat_catalog::base_agent_chat_style();
    let confirm_modal_base = super::confirm_modal_catalog::base_confirm_modal_style();
    let main_window_style_overrides: Vec<_> = STYLE_KNOBS
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
    let main_window_style_effective: Vec<_> = STYLE_KNOBS
        .iter()
        .map(|knob| {
            let StyleValue::Number(base_value) = (knob.get)(&main_window_base);
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
    let main_window_copy_overrides: Vec<_> = COPY_CONTROLS
        .iter()
        .filter_map(|control| {
            runtime_overrides::current_copy_value(control.id).map(|value| {
                json!({
                    "id": control.id.as_str(),
                    "label": control.label,
                    "section": control.section,
                    "value": value,
                })
            })
        })
        .collect();
    let main_window_copy_effective: Vec<_> = COPY_CONTROLS
        .iter()
        .map(|control| {
            let base = (control.base)();
            let value = runtime_overrides::effective_copy_value(control.id);
            json!({
                "id": control.id.as_str(),
                "label": control.label,
                "section": control.section,
                "base": base,
                "value": value,
                "overridden": runtime_overrides::current_copy_value(control.id).is_some(),
            })
        })
        .collect();
    let actions_popup_style_overrides: Vec<_> = ACTIONS_POPUP_KNOBS
        .iter()
        .filter_map(|knob| {
            runtime_overrides::current_actions_popup_value(knob.id).map(|value| {
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
    let actions_popup_style_effective: Vec<_> = ACTIONS_POPUP_KNOBS
        .iter()
        .map(|knob| {
            let StyleValue::Number(base_value) = (knob.get)(&actions_popup_base);
            let StyleValue::Number(value) = runtime_overrides::current_actions_popup_value(knob.id)
                .unwrap_or(StyleValue::Number(base_value));
            json!({
                "id": knob.id.as_str(),
                "label": knob.label,
                "group": knob.group.label(),
                "unit": knob.unit.label(),
                "base": base_value,
                "value": value,
                "overridden": runtime_overrides::current_actions_popup_value(knob.id).is_some(),
            })
        })
        .collect();
    let agent_chat_style_overrides: Vec<_> = AGENT_CHAT_KNOBS
        .iter()
        .filter_map(|knob| {
            runtime_overrides::current_agent_chat_value(knob.id).map(|value| {
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
    let agent_chat_style_effective: Vec<_> = AGENT_CHAT_KNOBS
        .iter()
        .map(|knob| {
            let StyleValue::Number(base_value) = (knob.get)(&agent_chat_base);
            let StyleValue::Number(value) = runtime_overrides::current_agent_chat_value(knob.id)
                .unwrap_or(StyleValue::Number(base_value));
            json!({
                "id": knob.id.as_str(),
                "label": knob.label,
                "group": knob.group.label(),
                "unit": knob.unit.label(),
                "base": base_value,
                "value": value,
                "overridden": runtime_overrides::current_agent_chat_value(knob.id).is_some(),
            })
        })
        .collect();
    let confirm_modal_style_overrides: Vec<_> = CONFIRM_MODAL_KNOBS
        .iter()
        .filter_map(|knob| {
            runtime_overrides::current_confirm_modal_value(knob.id).map(|value| {
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
    let confirm_modal_style_effective: Vec<_> = CONFIRM_MODAL_KNOBS
        .iter()
        .map(|knob| {
            let StyleValue::Number(base_value) = (knob.get)(&confirm_modal_base);
            let StyleValue::Number(value) = runtime_overrides::current_confirm_modal_value(knob.id)
                .unwrap_or(StyleValue::Number(base_value));
            json!({
                "id": knob.id.as_str(),
                "label": knob.label,
                "group": knob.group.label(),
                "unit": knob.unit.label(),
                "base": base_value,
                "value": value,
                "overridden": runtime_overrides::current_confirm_modal_value(knob.id).is_some(),
            })
        })
        .collect();
    let override_count = main_window_style_overrides
        .len()
        .saturating_add(main_window_copy_overrides.len())
        .saturating_add(actions_popup_style_overrides.len())
        .saturating_add(agent_chat_style_overrides.len())
        .saturating_add(confirm_modal_style_overrides.len());

    json!({
        "schema": "script-kit-dev-style/v2",
        "generatedAt": chrono::Local::now().to_rfc3339(),
        "runtimeGeneration": runtime_overrides::generation(),
        "overrideCount": override_count,
        "controls": {
            "mainWindowStyle": STYLE_KNOBS.len(),
            "mainWindowCopy": COPY_CONTROLS.len(),
            "actionsPopupStyle": ACTIONS_POPUP_KNOBS.len(),
            "agentChatStyle": AGENT_CHAT_KNOBS.len(),
            "confirmModalStyle": CONFIRM_MODAL_KNOBS.len(),
        },
        "surfaces": {
            "mainWindow": {
                "style": {
                    "overrides": main_window_style_overrides,
                    "effective": main_window_style_effective,
                },
                "copy": {
                    "overrides": main_window_copy_overrides,
                    "effective": main_window_copy_effective,
                },
            },
            "actionsPopup": {
                "style": {
                    "overrides": actions_popup_style_overrides,
                    "effective": actions_popup_style_effective,
                },
            },
            "agentChat": {
                "style": {
                    "overrides": agent_chat_style_overrides,
                    "effective": agent_chat_style_effective,
                },
            },
            "confirmModal": {
                "style": {
                    "overrides": confirm_modal_style_overrides,
                    "effective": confirm_modal_style_effective,
                },
            },
        },
        "agentPrompt": "Apply or reason about these Script Kit GPUI dev style overrides by matching style ids to src/dev_style_tool/catalog.rs, src/dev_style_tool/actions_popup_catalog.rs, src/dev_style_tool/agent_chat_catalog.rs, and src/dev_style_tool/confirm_modal_catalog.rs, and copy ids to src/dev_style_tool/copy_catalog.rs.",
    })
}

pub fn export_summary_for_path(path: &Path) -> String {
    format!("Saved {}", path.display())
}
