use super::{Failure, Window};
use serde_json::Value;

pub(super) struct Field {
    pub reference: String,
    pub label: String,
}

/// The caller took a --surface sheet snapshot, pinned to this exact window.
/// Never search the underlying document or pick an arbitrary writable text box.
pub(super) fn filename_field(s: &Value, w: &Window) -> Result<Field, Failure> {
    if s["app"].as_str() != Some(&w.app)
        || s["window"]["id"].as_str() != Some(&w.window_id)
        || s["complete"] != true
    {
        return Err("unsafe_surface");
    }
    let snap = s["snapshot_id"]
        .as_str()
        .filter(|s| !s.is_empty() && s.len() <= 64 && s.bytes().all(|b| b.is_ascii_alphanumeric()))
        .ok_or("unsafe_surface")?;
    let mut fields = Vec::new();
    let mut save = false;
    fn walk(
        n: &Value,
        snap: &str,
        fields: &mut Vec<Field>,
        save: &mut bool,
    ) -> Result<(), Failure> {
        if n["subtree_truncated"] == true {
            return Err("unsafe_surface");
        }
        let states = n["states"].as_array();
        let has = |x: &str| states.is_some_and(|a| a.iter().any(|v| v == x));
        if has("offscreen") || has("hidden") || has("secure") {
            return Ok(());
        }
        let name = n["name"].as_str().unwrap_or_default();
        let role = n["role"].as_str();
        if role == Some("button") && matches!(name, "Save" | "Export") {
            *save = true;
        }
        let is_filename = matches!(
            name.trim()
                .trim_end_matches(':')
                .to_ascii_lowercase()
                .as_str(),
            "save as" | "export as" | "file name" | "filename" | "name"
        ) || n["native_id"]["value"].as_str() == Some("saveAsNameTextField");
        if role == Some("textfield")
            && is_filename
            && !has("disabled")
            && !has("readonly")
            && n["available_actions"]
                .as_array()
                .is_some_and(|a| a.iter().any(|v| v == "SetValue"))
        {
            let r = n["ref_id"].as_str().ok_or("unsafe_surface")?;
            let num = r
                .strip_prefix(&format!("@{snap}:e"))
                .ok_or("unsafe_surface")?;
            if num.is_empty() || !num.bytes().all(|b| b.is_ascii_digit()) {
                return Err("unsafe_surface");
            }
            fields.push(Field {
                reference: r.into(),
                label: if name.is_empty() {
                    "Filename".into()
                } else {
                    name.into()
                },
            });
        }
        if let Some(children) = n["children"].as_array() {
            for c in children {
                walk(c, snap, fields, save)?;
            }
        }
        Ok(())
    }
    walk(&s["tree"], snap, &mut fields, &mut save)?;
    if !save {
        return Err("unsafe_surface");
    };
    match fields.len() {
        1 => Ok(fields.remove(0)),
        0 => Err("missing_field"),
        _ => Err("ambiguous_field"),
    }
}
