use crate::components::{
    confirm_modal_shell::{
        CONFIRM_MODAL_HEADER_ACCENT_HEIGHT, CONFIRM_MODAL_HEADER_ACCENT_WIDTH,
        CONFIRM_MODAL_HEADER_GAP, CONFIRM_MODAL_RADIUS,
    },
    footer_chrome::{
        current_main_menu_footer_height, current_main_menu_footer_metrics,
        footer_action_slot_width, footer_button_height, FooterActionSlot,
    },
};

use super::catalog::{StyleUnit, StyleValue};

pub(crate) const CONFIRM_MODAL_DEFAULT_PADDING_X: f32 = 16.0;
pub(crate) const CONFIRM_MODAL_DEFAULT_PADDING_Y: f32 = 16.0;
pub(crate) const CONFIRM_MODAL_DEFAULT_SECTION_GAP: f32 = 10.0;
pub(crate) const CONFIRM_MODAL_DEFAULT_BODY_LINE_HEIGHT: f32 = 16.0;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ConfirmModalStyleDef {
    pub shell: ConfirmModalShellStyle,
    pub header: ConfirmModalHeaderStyle,
    pub anatomy: ConfirmModalAnatomyStyle,
    pub actions: ConfirmModalActionsStyle,
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

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ConfirmModalAnatomyStyle {
    pub header_body_gap: f32,
    pub body_actions_gap: f32,
    pub body_line_height: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ConfirmModalActionsStyle {
    pub gap: f32,
    pub button_height: f32,
    pub cancel_slot_width: f32,
    pub confirm_slot_width: f32,
    pub button_radius: f32,
    pub padding_x: f32,
    pub edge_padding_x: f32,
    pub padding_y: f32,
    pub content_gap: f32,
}

pub fn base_confirm_modal_style() -> ConfirmModalStyleDef {
    let footer_metrics = current_main_menu_footer_metrics();
    ConfirmModalStyleDef {
        shell: ConfirmModalShellStyle {
            padding_x: CONFIRM_MODAL_DEFAULT_PADDING_X,
            padding_y: CONFIRM_MODAL_DEFAULT_PADDING_Y,
            gap: CONFIRM_MODAL_DEFAULT_SECTION_GAP,
            radius: CONFIRM_MODAL_RADIUS,
        },
        header: ConfirmModalHeaderStyle {
            accent_width: CONFIRM_MODAL_HEADER_ACCENT_WIDTH,
            accent_height: CONFIRM_MODAL_HEADER_ACCENT_HEIGHT,
            gap: CONFIRM_MODAL_HEADER_GAP,
        },
        anatomy: ConfirmModalAnatomyStyle {
            header_body_gap: CONFIRM_MODAL_DEFAULT_SECTION_GAP,
            body_actions_gap: CONFIRM_MODAL_DEFAULT_SECTION_GAP,
            body_line_height: CONFIRM_MODAL_DEFAULT_BODY_LINE_HEIGHT,
        },
        actions: ConfirmModalActionsStyle {
            gap: footer_metrics.item_gap_px,
            button_height: footer_button_height(current_main_menu_footer_height()),
            cancel_slot_width: footer_action_slot_width(FooterActionSlot::Close),
            confirm_slot_width: footer_action_slot_width(FooterActionSlot::Run),
            button_radius: footer_metrics.button_radius,
            padding_x: footer_metrics.button_padding_x,
            edge_padding_x: footer_metrics.button_padding_x,
            padding_y: footer_metrics.button_padding_y,
            content_gap: footer_metrics.content_gap,
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
    Anatomy,
    Actions,
}

impl ConfirmModalKnobGroup {
    pub const fn label(self) -> &'static str {
        match self {
            Self::Shell => "Confirm modal shell",
            Self::Header => "Confirm modal header",
            Self::Anatomy => "Confirm modal anatomy",
            Self::Actions => "Confirm modal actions",
        }
    }

    /// One-line anatomy hint tying the group to the element it styles in the
    /// confirm modal, so tool users can tell which controls map where.
    pub const fn description(self) -> &'static str {
        match self {
            Self::Shell => "Modal surface: width, padding, corner radius",
            Self::Header => "Title and message block at the top",
            Self::Anatomy => "Spacing between header, body, and actions",
            Self::Actions => "Confirm/cancel button row at the bottom",
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
f32_knob!(
    CONFIRM_MODAL_ANATOMY_HEADER_BODY_GAP_KNOB_ID,
    get_anatomy_header_body_gap,
    apply_anatomy_header_body_gap,
    "confirmModal.anatomy.headerBodyGap",
    anatomy.header_body_gap
);
f32_knob!(
    CONFIRM_MODAL_ANATOMY_BODY_ACTIONS_GAP_KNOB_ID,
    get_anatomy_body_actions_gap,
    apply_anatomy_body_actions_gap,
    "confirmModal.anatomy.bodyActionsGap",
    anatomy.body_actions_gap
);
f32_knob!(
    CONFIRM_MODAL_ANATOMY_BODY_LINE_HEIGHT_KNOB_ID,
    get_anatomy_body_line_height,
    apply_anatomy_body_line_height,
    "confirmModal.anatomy.bodyLineHeight",
    anatomy.body_line_height
);
f32_knob!(
    CONFIRM_MODAL_ACTIONS_GAP_KNOB_ID,
    get_actions_gap,
    apply_actions_gap,
    "confirmModal.actions.gap",
    actions.gap
);
f32_knob!(
    CONFIRM_MODAL_ACTIONS_BUTTON_HEIGHT_KNOB_ID,
    get_actions_button_height,
    apply_actions_button_height,
    "confirmModal.actions.buttonHeight",
    actions.button_height
);
f32_knob!(
    CONFIRM_MODAL_ACTIONS_CANCEL_SLOT_WIDTH_KNOB_ID,
    get_actions_cancel_slot_width,
    apply_actions_cancel_slot_width,
    "confirmModal.actions.cancelSlotWidth",
    actions.cancel_slot_width
);
f32_knob!(
    CONFIRM_MODAL_ACTIONS_CONFIRM_SLOT_WIDTH_KNOB_ID,
    get_actions_confirm_slot_width,
    apply_actions_confirm_slot_width,
    "confirmModal.actions.confirmSlotWidth",
    actions.confirm_slot_width
);
f32_knob!(
    CONFIRM_MODAL_ACTIONS_BUTTON_RADIUS_KNOB_ID,
    get_actions_button_radius,
    apply_actions_button_radius,
    "confirmModal.actions.buttonRadius",
    actions.button_radius
);
f32_knob!(
    CONFIRM_MODAL_ACTIONS_PADDING_X_KNOB_ID,
    get_actions_padding_x,
    apply_actions_padding_x,
    "confirmModal.actions.paddingX",
    actions.padding_x
);
f32_knob!(
    CONFIRM_MODAL_ACTIONS_EDGE_PADDING_X_KNOB_ID,
    get_actions_edge_padding_x,
    apply_actions_edge_padding_x,
    "confirmModal.actions.edgePaddingX",
    actions.edge_padding_x
);
f32_knob!(
    CONFIRM_MODAL_ACTIONS_PADDING_Y_KNOB_ID,
    get_actions_padding_y,
    apply_actions_padding_y,
    "confirmModal.actions.paddingY",
    actions.padding_y
);
f32_knob!(
    CONFIRM_MODAL_ACTIONS_CONTENT_GAP_KNOB_ID,
    get_actions_content_gap,
    apply_actions_content_gap,
    "confirmModal.actions.contentGap",
    actions.content_gap
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
    ConfirmModalKnob {
        id: CONFIRM_MODAL_ANATOMY_HEADER_BODY_GAP_KNOB_ID,
        label: "Header/body gap",
        group: ConfirmModalKnobGroup::Anatomy,
        unit: StyleUnit::Px,
        min: 0.0,
        max: 24.0,
        step: 1.0,
        get: get_anatomy_header_body_gap,
        apply: apply_anatomy_header_body_gap,
    },
    ConfirmModalKnob {
        id: CONFIRM_MODAL_ANATOMY_BODY_ACTIONS_GAP_KNOB_ID,
        label: "Body/actions gap",
        group: ConfirmModalKnobGroup::Anatomy,
        unit: StyleUnit::Px,
        min: 0.0,
        max: 24.0,
        step: 1.0,
        get: get_anatomy_body_actions_gap,
        apply: apply_anatomy_body_actions_gap,
    },
    ConfirmModalKnob {
        id: CONFIRM_MODAL_ANATOMY_BODY_LINE_HEIGHT_KNOB_ID,
        label: "Body line height",
        group: ConfirmModalKnobGroup::Anatomy,
        unit: StyleUnit::Px,
        min: 12.0,
        max: 24.0,
        step: 1.0,
        get: get_anatomy_body_line_height,
        apply: apply_anatomy_body_line_height,
    },
    ConfirmModalKnob {
        id: CONFIRM_MODAL_ACTIONS_GAP_KNOB_ID,
        label: "Action button gap",
        group: ConfirmModalKnobGroup::Actions,
        unit: StyleUnit::Px,
        min: 0.0,
        max: 24.0,
        step: 1.0,
        get: get_actions_gap,
        apply: apply_actions_gap,
    },
    ConfirmModalKnob {
        id: CONFIRM_MODAL_ACTIONS_BUTTON_HEIGHT_KNOB_ID,
        label: "Action button height",
        group: ConfirmModalKnobGroup::Actions,
        unit: StyleUnit::Px,
        min: 20.0,
        max: 44.0,
        step: 1.0,
        get: get_actions_button_height,
        apply: apply_actions_button_height,
    },
    ConfirmModalKnob {
        id: CONFIRM_MODAL_ACTIONS_CANCEL_SLOT_WIDTH_KNOB_ID,
        label: "Cancel slot width",
        group: ConfirmModalKnobGroup::Actions,
        unit: StyleUnit::Px,
        min: 48.0,
        max: 180.0,
        step: 1.0,
        get: get_actions_cancel_slot_width,
        apply: apply_actions_cancel_slot_width,
    },
    ConfirmModalKnob {
        id: CONFIRM_MODAL_ACTIONS_CONFIRM_SLOT_WIDTH_KNOB_ID,
        label: "Confirm slot width",
        group: ConfirmModalKnobGroup::Actions,
        unit: StyleUnit::Px,
        min: 48.0,
        max: 180.0,
        step: 1.0,
        get: get_actions_confirm_slot_width,
        apply: apply_actions_confirm_slot_width,
    },
    ConfirmModalKnob {
        id: CONFIRM_MODAL_ACTIONS_BUTTON_RADIUS_KNOB_ID,
        label: "Action radius",
        group: ConfirmModalKnobGroup::Actions,
        unit: StyleUnit::Px,
        min: 0.0,
        max: 18.0,
        step: 1.0,
        get: get_actions_button_radius,
        apply: apply_actions_button_radius,
    },
    ConfirmModalKnob {
        id: CONFIRM_MODAL_ACTIONS_PADDING_X_KNOB_ID,
        label: "Action padding X",
        group: ConfirmModalKnobGroup::Actions,
        unit: StyleUnit::Px,
        min: 0.0,
        max: 16.0,
        step: 1.0,
        get: get_actions_padding_x,
        apply: apply_actions_padding_x,
    },
    ConfirmModalKnob {
        id: CONFIRM_MODAL_ACTIONS_EDGE_PADDING_X_KNOB_ID,
        label: "Action edge padding X",
        group: ConfirmModalKnobGroup::Actions,
        unit: StyleUnit::Px,
        min: 0.0,
        max: 16.0,
        step: 1.0,
        get: get_actions_edge_padding_x,
        apply: apply_actions_edge_padding_x,
    },
    ConfirmModalKnob {
        id: CONFIRM_MODAL_ACTIONS_PADDING_Y_KNOB_ID,
        label: "Action padding Y",
        group: ConfirmModalKnobGroup::Actions,
        unit: StyleUnit::Px,
        min: 0.0,
        max: 16.0,
        step: 1.0,
        get: get_actions_padding_y,
        apply: apply_actions_padding_y,
    },
    ConfirmModalKnob {
        id: CONFIRM_MODAL_ACTIONS_CONTENT_GAP_KNOB_ID,
        label: "Label/keycap gap",
        group: ConfirmModalKnobGroup::Actions,
        unit: StyleUnit::Px,
        min: 0.0,
        max: 16.0,
        step: 1.0,
        get: get_actions_content_gap,
        apply: apply_actions_content_gap,
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
