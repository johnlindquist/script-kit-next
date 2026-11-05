//! The launcher's transport to the same version-checked, revision-guarded helper as MCP.
use super::run_config_cli;
use serde_json::Value;
use sk_one_window::font_preference::{change, FontPreference};

fn run(args: &[&str]) -> Result<Value, String> {
    let (_, reply) = run_config_cli(
        "font_preference",
        args.iter().map(|s| (*s).to_owned()).collect(),
    )
    .map_err(|(_, e)| {
        match e.message.as_str() {
            "external_process_forbidden" => "This session cannot run the settings helper",
            s if s.contains("Stale config") => "Settings changed; review the current value",
            _ => "Couldn't access the verified settings helper",
        }
        .to_owned()
    })?;
    reply
        .get("data")
        .cloned()
        .ok_or_else(|| "Invalid settings reply".into())
}

pub struct FontUndo(change::FontUndo);

pub fn apply(choice: FontPreference, displayed: f32) -> anyhow::Result<Option<FontUndo>> {
    change::apply(choice, displayed, run)
        .map(|undo| undo.map(FontUndo))
        .map_err(anyhow::Error::msg)
}

impl FontUndo {
    pub fn restore(self) -> anyhow::Result<Option<f32>> {
        self.0.restore(run).map_err(anyhow::Error::msg)
    }
}
