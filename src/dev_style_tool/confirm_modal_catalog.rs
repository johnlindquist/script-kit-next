use super::catalog::{StyleUnit, StyleValue};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ConfirmModalStyleDef {
    pub shell: ConfirmModalShellStyle,
    pub header: ConfirmModalHeaderStyle,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ConfirmModalShellStyle {
    pub padding_x: f32,
    pub padding_y: f32,
    pub gap: f32,
    pub radius: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ConfirmModalHeaderStyle {
    pub accent_width: f32,
    pub accent_height: f32,
    pub gap: f32,
}

pub fn base_confirm_modal_style() -> ConfirmModalStyleDef {
    ConfirmModalStyleDef {
        shell: ConfirmModalShellStyle {
            padding_x: 16.0,
            padding_y: 16.0,
            gap: 10.0,
            radius: 8.0,
        },
        header: ConfirmModalHeaderStyle {
            accent_width: 2.0,
            accent_height: 14.0,
            gap: 8.0,
        },
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ConfirmModalKnobId(&'static str);

impl ConfirmModalKnobId {
    pub const fn new(value: &'static str) -> Self {
        Self(value)
    }

    pub const fn as_str(self) -> &'static str {
        self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConfirmModalKnobGroup {
    Shell,
    Header,
}

impl ConfirmModalKnobGroup {
    pub const fn label(self) -> &'static str {
        match self {
            Self::Shell => "Confirm modal shell",
            Self::Header => "Confirm modal header",
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct ConfirmModalKnob {
    pub id: ConfirmModalKnobId,
    pub label: &'static str,
    pub group: ConfirmModalKnobGroup,
    pub unit: StyleUnit,
    pub min: f32,
    pub max: f32,
    pub step: f32,
    pub get: fn(&ConfirmModalStyleDef) -> StyleValue,
    pub apply: fn(&mut ConfirmModalStyleDef, StyleValue),
}

impl ConfirmModalKnob {
    pub fn clamp_value(self, value: StyleValue) -> StyleValue {
        match value {
            StyleValue::Number(number) => StyleValue::Number(number.clamp(self.min, self.max)),
        }
    }
}

macro_rules! f32_knob {
    ($id_const:ident, $get_fn:ident, $apply_fn:ident, $id:literal, $section:ident.$field:ident) => {
        pub const $id_const: ConfirmModalKnobId = ConfirmModalKnobId::new($id);
        fn $get_fn(def: &ConfirmModalStyleDef) -> StyleValue {
            StyleValue::Number(def.$section.$field)
        }
        fn $apply_fn(def: &mut ConfirmModalStyleDef, value: StyleValue) {
            let StyleValue::Number(value) = value;
            def.$section.$field = value;
        }
    };
}

f32_knob!(
    CONFIRM_MODAL_PADDING_X_KNOB_ID,
    get_shell_padding_x,
    apply_shell_padding_x,
    "confirmModal.shell.paddingX",
    shell.padding_x
);
f32_knob!(
    CONFIRM_MODAL_PADDING_Y_KNOB_ID,
    get_shell_padding_y,
    apply_shell_padding_y,
    "confirmModal.shell.paddingY",
    shell.padding_y
);
f32_knob!(
    CONFIRM_MODAL_GAP_KNOB_ID,
    get_shell_gap,
    apply_shell_gap,
    "confirmModal.shell.gap",
    shell.gap
);
f32_knob!(
    CONFIRM_MODAL_RADIUS_KNOB_ID,
    get_shell_radius,
    apply_shell_radius,
    "confirmModal.shell.radius",
    shell.radius
);
f32_knob!(
    CONFIRM_MODAL_HEADER_ACCENT_WIDTH_KNOB_ID,
    get_header_accent_width,
    apply_header_accent_width,
    "confirmModal.header.accentWidth",
    header.accent_width
);
f32_knob!(
    CONFIRM_MODAL_HEADER_ACCENT_HEIGHT_KNOB_ID,
    get_header_accent_height,
    apply_header_accent_height,
    "confirmModal.header.accentHeight",
    header.accent_height
);
f32_knob!(
    CONFIRM_MODAL_HEADER_GAP_KNOB_ID,
    get_header_gap,
    apply_header_gap,
    "confirmModal.header.gap",
    header.gap
);

pub const CONFIRM_MODAL_KNOBS: &[ConfirmModalKnob] = &[
    ConfirmModalKnob {
        id: CONFIRM_MODAL_PADDING_X_KNOB_ID,
        label: "Shell padding X",
        group: ConfirmModalKnobGroup::Shell,
        unit: StyleUnit::Px,
        min: 8.0,
        max: 32.0,
        step: 1.0,
        get: get_shell_padding_x,
        apply: apply_shell_padding_x,
    },
    ConfirmModalKnob {
        id: CONFIRM_MODAL_PADDING_Y_KNOB_ID,
        label: "Shell padding Y",
        group: ConfirmModalKnobGroup::Shell,
        unit: StyleUnit::Px,
        min: 8.0,
        max: 32.0,
        step: 1.0,
        get: get_shell_padding_y,
        apply: apply_shell_padding_y,
    },
    ConfirmModalKnob {
        id: CONFIRM_MODAL_GAP_KNOB_ID,
        label: "Section gap",
        group: ConfirmModalKnobGroup::Shell,
        unit: StyleUnit::Px,
        min: 4.0,
        max: 20.0,
        step: 1.0,
        get: get_shell_gap,
        apply: apply_shell_gap,
    },
    ConfirmModalKnob {
        id: CONFIRM_MODAL_RADIUS_KNOB_ID,
        label: "Shell radius",
        group: ConfirmModalKnobGroup::Shell,
        unit: StyleUnit::Px,
        min: 0.0,
        max: 18.0,
        step: 1.0,
        get: get_shell_radius,
        apply: apply_shell_radius,
    },
    ConfirmModalKnob {
        id: CONFIRM_MODAL_HEADER_ACCENT_WIDTH_KNOB_ID,
        label: "Accent width",
        group: ConfirmModalKnobGroup::Header,
        unit: StyleUnit::Px,
        min: 1.0,
        max: 6.0,
        step: 0.5,
        get: get_header_accent_width,
        apply: apply_header_accent_width,
    },
    ConfirmModalKnob {
        id: CONFIRM_MODAL_HEADER_ACCENT_HEIGHT_KNOB_ID,
        label: "Accent height",
        group: ConfirmModalKnobGroup::Header,
        unit: StyleUnit::Px,
        min: 8.0,
        max: 24.0,
        step: 1.0,
        get: get_header_accent_height,
        apply: apply_header_accent_height,
    },
    ConfirmModalKnob {
        id: CONFIRM_MODAL_HEADER_GAP_KNOB_ID,
        label: "Header gap",
        group: ConfirmModalKnobGroup::Header,
        unit: StyleUnit::Px,
        min: 4.0,
        max: 16.0,
        step: 1.0,
        get: get_header_gap,
        apply: apply_header_gap,
    },
];

pub fn confirm_modal_knob_by_id(id: ConfirmModalKnobId) -> Option<&'static ConfirmModalKnob> {
    CONFIRM_MODAL_KNOBS.iter().find(|knob| knob.id == id)
}

pub fn confirm_modal_knob_id_from_str(value: &str) -> Option<ConfirmModalKnobId> {
    let normalized = value
        .strip_prefix("control:dev-style-tool-confirm-modal:")
        .or_else(|| value.strip_prefix("slider:dev-style-tool-confirm-modal:"))
        .or_else(|| value.strip_prefix("input:dev-style-tool-confirm-modal:"))
        .or_else(|| value.strip_prefix("button:dev-style-tool-confirm-modal-reset:"))
        .unwrap_or(value);
    CONFIRM_MODAL_KNOBS
        .iter()
        .find(|knob| knob.id.as_str() == normalized)
        .map(|knob| knob.id)
}
