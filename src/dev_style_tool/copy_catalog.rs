#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct CopyControlId(&'static str);

impl CopyControlId {
    pub const fn new(value: &'static str) -> Self {
        Self(value)
    }

    pub const fn as_str(self) -> &'static str {
        self.0
    }
}

#[derive(Debug, Clone, Copy)]
pub struct CopyControl {
    pub id: CopyControlId,
    pub label: &'static str,
    pub section: &'static str,
    pub base: fn() -> String,
}

pub const MAIN_INPUT_PLACEHOLDER_COPY_ID: CopyControlId =
    CopyControlId::new("main.input.placeholder");

fn base_main_input_placeholder() -> String {
    crate::ROOT_LAUNCHER_PLACEHOLDER.to_string()
}

pub const COPY_CONTROLS: &[CopyControl] = &[CopyControl {
    id: MAIN_INPUT_PLACEHOLDER_COPY_ID,
    label: "Main input placeholder",
    section: "Main window",
    base: base_main_input_placeholder,
}];

pub fn copy_control_by_id(id: CopyControlId) -> Option<&'static CopyControl> {
    COPY_CONTROLS.iter().find(|control| control.id == id)
}

pub fn copy_control_id_from_str(value: &str) -> Option<CopyControlId> {
    let normalized = value
        .strip_prefix("control:dev-style-tool-copy:")
        .unwrap_or(value)
        .strip_prefix("input:dev-style-tool-copy:")
        .unwrap_or_else(|| {
            value
                .strip_prefix("button:dev-style-tool-copy-reset:")
                .unwrap_or(value)
        });
    COPY_CONTROLS
        .iter()
        .find(|control| control.id.as_str() == normalized)
        .map(|control| control.id)
}
