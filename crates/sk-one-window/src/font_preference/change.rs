//! A text-size transaction over the installed settings CLI contract. The host supplies the
//! transport; these rules never spawn a process, access a file, or depend on GPUI.
use super::{FontPreference, FontSurface};
use serde::Deserialize;
use serde_json::{Number, Value};

type Result<T> = std::result::Result<T, String>;

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct SizeState {
    path: String,
    key: String,
    revision: String,
    value: Number,
    is_default: bool,
}
impl SizeState {
    fn read(key: &str, run: &mut impl FnMut(&[&str]) -> Result<Value>) -> Result<Self> {
        let s: Self =
            serde_json::from_value(run(&["get", key])?).map_err(|_| "Invalid settings reply")?;
        if s.key != key
            || !std::path::Path::new(&s.path).is_absolute()
            || !(s.revision == "missing"
                || (s.revision.len() == 64 && s.revision.bytes().all(|b| b.is_ascii_hexdigit())))
        {
            return Err("Wrong settings target or revision".into());
        }
        Ok(s)
    }
    fn pixels(&self) -> Result<f32> {
        self.value
            .as_f64()
            .map(|v| v as f32)
            .filter(|n| n.is_finite() && *n > 0.)
            .ok_or_else(|| "Invalid stored text size".into())
    }
}

pub struct FontUndo {
    surface: FontSurface,
    before: SizeState,
    saved_revision: String,
}

/// Compare against the value the person saw. The CLI additionally checks the file revision
/// inside its write lock; an edit that races with the read must not be overwritten.
pub fn apply(
    choice: FontPreference,
    displayed: f32,
    mut run: impl FnMut(&[&str]) -> Result<Value>,
) -> Result<Option<FontUndo>> {
    let key = choice.surface.key();
    let before = SizeState::read(key, &mut run)?;
    if before.pixels()? != displayed {
        return Err("Settings changed; review the current text size".into());
    }
    if displayed == f32::from(choice.pixels) {
        return Ok(None);
    }
    let saved = run(&[
        "set-json",
        key,
        &choice.pixels.to_string(),
        "--if-revision",
        &before.revision,
    ])?;
    let after = SizeState::read(key, &mut run)?;
    if after.path != before.path
        || Some(after.revision.as_str()) != saved["revision"].as_str()
        || after.pixels()? != f32::from(choice.pixels)
    {
        return Err("Settings changed before readback".into());
    }
    Ok(Some(FontUndo {
        surface: choice.surface,
        before,
        saved_revision: after.revision,
    }))
}
impl FontUndo {
    pub fn restore(self, mut run: impl FnMut(&[&str]) -> Result<Value>) -> Result<Option<f32>> {
        let key = self.surface.key();
        let current = SizeState::read(key, &mut run)?;
        if current.path != self.before.path || current.revision != self.saved_revision {
            return Err("Settings changed; undo did not overwrite them".into());
        }
        let written = if self.before.is_default {
            run(&["reset", key, "--if-revision", &self.saved_revision])?
        } else {
            run(&[
                "set-json",
                key,
                &self.before.value.to_string(),
                "--if-revision",
                &self.saved_revision,
            ])?
        };
        let after = SizeState::read(key, &mut run)?;
        if after.path != self.before.path
            || after.value != self.before.value
            || after.is_default != self.before.is_default
            || Some(after.revision.as_str()) != written["revision"].as_str()
        {
            return Err("Settings changed before readback".into());
        }
        Ok(if after.is_default {
            None
        } else {
            Some(after.pixels()?)
        })
    }
}
