use crate::designs::ActionsPopupThemeDef;

use super::catalog::{StyleUnit, StyleValue};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ActionsPopupKnobId(&'static str);

impl ActionsPopupKnobId {
    pub const fn new(value: &'static str) -> Self {
        Self(value)
    }

    pub const fn as_str(self) -> &'static str {
        self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActionsPopupKnobGroup {
    Shell,
    Search,
    List,
    Row,
    Section,
    ContextHeader,
    Shortcut,
}

impl ActionsPopupKnobGroup {
    pub const fn label(self) -> &'static str {
        match self {
            Self::Shell => "Actions shell",
            Self::Search => "Actions search",
            Self::List => "Actions list",
            Self::Row => "Actions rows",
            Self::Section => "Actions sections",
            Self::ContextHeader => "Actions context header",
            Self::Shortcut => "Actions shortcuts",
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct ActionsPopupKnob {
    pub id: ActionsPopupKnobId,
    pub label: &'static str,
    pub group: ActionsPopupKnobGroup,
    pub unit: StyleUnit,
    pub min: f32,
    pub max: f32,
    pub step: f32,
    pub get: fn(&ActionsPopupThemeDef) -> StyleValue,
    pub apply: fn(&mut ActionsPopupThemeDef, StyleValue),
}

impl ActionsPopupKnob {
    pub fn clamp_value(self, value: StyleValue) -> StyleValue {
        match value {
            StyleValue::Number(number) => StyleValue::Number(number.clamp(self.min, self.max)),
        }
    }
}

macro_rules! f32_knob {
    ($id_const:ident, $get_fn:ident, $apply_fn:ident, $id:literal, $section:ident.$field:ident) => {
        pub const $id_const: ActionsPopupKnobId = ActionsPopupKnobId::new($id);
        fn $get_fn(def: &ActionsPopupThemeDef) -> StyleValue {
            StyleValue::Number(def.$section.$field)
        }
        fn $apply_fn(def: &mut ActionsPopupThemeDef, value: StyleValue) {
            let StyleValue::Number(value) = value;
            def.$section.$field = value;
        }
    };
}

f32_knob!(
    ACTIONS_SHELL_WIDTH_KNOB_ID,
    get_actions_shell_width,
    apply_actions_shell_width,
    "actions.shell.width",
    shell.width
);
f32_knob!(
    ACTIONS_SHELL_MAX_HEIGHT_KNOB_ID,
    get_actions_shell_max_height,
    apply_actions_shell_max_height,
    "actions.shell.maxHeight",
    shell.max_height
);
f32_knob!(
    ACTIONS_SHELL_MARGIN_X_KNOB_ID,
    get_actions_shell_margin_x,
    apply_actions_shell_margin_x,
    "actions.shell.marginX",
    shell.margin_x
);
f32_knob!(
    ACTIONS_SHELL_MARGIN_Y_KNOB_ID,
    get_actions_shell_margin_y,
    apply_actions_shell_margin_y,
    "actions.shell.marginY",
    shell.margin_y
);
f32_knob!(
    ACTIONS_SEARCH_HEIGHT_KNOB_ID,
    get_actions_search_height,
    apply_actions_search_height,
    "actions.search.height",
    search.height
);
f32_knob!(
    ACTIONS_SEARCH_INNER_HEIGHT_KNOB_ID,
    get_actions_search_inner_height,
    apply_actions_search_inner_height,
    "actions.search.innerHeight",
    search.inner_height
);
f32_knob!(
    ACTIONS_LIST_ROW_HEIGHT_KNOB_ID,
    get_actions_list_row_height,
    apply_actions_list_row_height,
    "actions.list.rowHeight",
    list.row_height
);
f32_knob!(
    ACTIONS_LIST_SECTION_HEADER_HEIGHT_KNOB_ID,
    get_actions_list_section_header_height,
    apply_actions_list_section_header_height,
    "actions.list.sectionHeaderHeight",
    list.section_header_height
);
f32_knob!(
    ACTIONS_ROW_INSET_X_KNOB_ID,
    get_actions_row_inset_x,
    apply_actions_row_inset_x,
    "actions.row.insetX",
    row.inset_x
);
f32_knob!(
    ACTIONS_ROW_INNER_Y_KNOB_ID,
    get_actions_row_inner_y,
    apply_actions_row_inner_y,
    "actions.row.innerY",
    row.inner_y
);
f32_knob!(
    ACTIONS_ROW_RADIUS_KNOB_ID,
    get_actions_row_radius,
    apply_actions_row_radius,
    "actions.row.radius",
    row.radius
);
f32_knob!(
    ACTIONS_ROW_SELECTION_OPACITY_KNOB_ID,
    get_actions_row_selection_opacity,
    apply_actions_row_selection_opacity,
    "actions.row.selectionOpacity",
    row.selection_opacity
);
f32_knob!(
    ACTIONS_ROW_HOVER_OPACITY_KNOB_ID,
    get_actions_row_hover_opacity,
    apply_actions_row_hover_opacity,
    "actions.row.hoverOpacity",
    row.hover_opacity
);
f32_knob!(
    ACTIONS_SECTION_PADDING_X_KNOB_ID,
    get_actions_section_padding_x,
    apply_actions_section_padding_x,
    "actions.section.paddingX",
    section.padding_x
);
f32_knob!(
    ACTIONS_SECTION_PADDING_TOP_KNOB_ID,
    get_actions_section_padding_top,
    apply_actions_section_padding_top,
    "actions.section.paddingTop",
    section.padding_top
);
f32_knob!(
    ACTIONS_CONTEXT_HEADER_HEIGHT_KNOB_ID,
    get_actions_context_header_height,
    apply_actions_context_header_height,
    "actions.contextHeader.height",
    context_header.height
);
f32_knob!(
    ACTIONS_SHORTCUT_KEYCAP_HEIGHT_KNOB_ID,
    get_actions_shortcut_keycap_height,
    apply_actions_shortcut_keycap_height,
    "actions.shortcut.keycapHeight",
    shortcut.keycap_height
);
f32_knob!(
    ACTIONS_SHORTCUT_KEYCAP_FONT_SIZE_KNOB_ID,
    get_actions_shortcut_keycap_font_size,
    apply_actions_shortcut_keycap_font_size,
    "actions.shortcut.keycapFontSize",
    shortcut.keycap_font_size
);

pub const ACTIONS_POPUP_KNOBS: &[ActionsPopupKnob] = &[
    ActionsPopupKnob {
        id: ACTIONS_SHELL_WIDTH_KNOB_ID,
        label: "Actions popup width",
        group: ActionsPopupKnobGroup::Shell,
        unit: StyleUnit::Px,
        min: 240.0,
        max: 720.0,
        step: 1.0,
        get: get_actions_shell_width,
        apply: apply_actions_shell_width,
    },
    ActionsPopupKnob {
        id: ACTIONS_SHELL_MAX_HEIGHT_KNOB_ID,
        label: "Actions popup max height",
        group: ActionsPopupKnobGroup::Shell,
        unit: StyleUnit::Px,
        min: 160.0,
        max: 900.0,
        step: 1.0,
        get: get_actions_shell_max_height,
        apply: apply_actions_shell_max_height,
    },
    ActionsPopupKnob {
        id: ACTIONS_SHELL_MARGIN_X_KNOB_ID,
        label: "Actions popup margin X",
        group: ActionsPopupKnobGroup::Shell,
        unit: StyleUnit::Px,
        min: 0.0,
        max: 64.0,
        step: 1.0,
        get: get_actions_shell_margin_x,
        apply: apply_actions_shell_margin_x,
    },
    ActionsPopupKnob {
        id: ACTIONS_SHELL_MARGIN_Y_KNOB_ID,
        label: "Actions popup margin Y",
        group: ActionsPopupKnobGroup::Shell,
        unit: StyleUnit::Px,
        min: 0.0,
        max: 64.0,
        step: 1.0,
        get: get_actions_shell_margin_y,
        apply: apply_actions_shell_margin_y,
    },
    ActionsPopupKnob {
        id: ACTIONS_SEARCH_HEIGHT_KNOB_ID,
        label: "Actions search height",
        group: ActionsPopupKnobGroup::Search,
        unit: StyleUnit::Px,
        min: 20.0,
        max: 88.0,
        step: 1.0,
        get: get_actions_search_height,
        apply: apply_actions_search_height,
    },
    ActionsPopupKnob {
        id: ACTIONS_SEARCH_INNER_HEIGHT_KNOB_ID,
        label: "Actions search inner height",
        group: ActionsPopupKnobGroup::Search,
        unit: StyleUnit::Px,
        min: 12.0,
        max: 64.0,
        step: 1.0,
        get: get_actions_search_inner_height,
        apply: apply_actions_search_inner_height,
    },
    ActionsPopupKnob {
        id: ACTIONS_LIST_ROW_HEIGHT_KNOB_ID,
        label: "Action row height",
        group: ActionsPopupKnobGroup::List,
        unit: StyleUnit::Px,
        min: 24.0,
        max: 80.0,
        step: 1.0,
        get: get_actions_list_row_height,
        apply: apply_actions_list_row_height,
    },
    ActionsPopupKnob {
        id: ACTIONS_LIST_SECTION_HEADER_HEIGHT_KNOB_ID,
        label: "Action section header height",
        group: ActionsPopupKnobGroup::List,
        unit: StyleUnit::Px,
        min: 8.0,
        max: 64.0,
        step: 1.0,
        get: get_actions_list_section_header_height,
        apply: apply_actions_list_section_header_height,
    },
    ActionsPopupKnob {
        id: ACTIONS_ROW_INSET_X_KNOB_ID,
        label: "Action row inset X",
        group: ActionsPopupKnobGroup::Row,
        unit: StyleUnit::Px,
        min: 0.0,
        max: 48.0,
        step: 1.0,
        get: get_actions_row_inset_x,
        apply: apply_actions_row_inset_x,
    },
    ActionsPopupKnob {
        id: ACTIONS_ROW_INNER_Y_KNOB_ID,
        label: "Action row inner Y",
        group: ActionsPopupKnobGroup::Row,
        unit: StyleUnit::Px,
        min: 0.0,
        max: 16.0,
        step: 1.0,
        get: get_actions_row_inner_y,
        apply: apply_actions_row_inner_y,
    },
    ActionsPopupKnob {
        id: ACTIONS_ROW_RADIUS_KNOB_ID,
        label: "Action row radius",
        group: ActionsPopupKnobGroup::Row,
        unit: StyleUnit::Px,
        min: 0.0,
        max: 32.0,
        step: 1.0,
        get: get_actions_row_radius,
        apply: apply_actions_row_radius,
    },
    ActionsPopupKnob {
        id: ACTIONS_ROW_SELECTION_OPACITY_KNOB_ID,
        label: "Action selected opacity",
        group: ActionsPopupKnobGroup::Row,
        unit: StyleUnit::Opacity,
        min: 0.0,
        max: 1.0,
        step: 0.01,
        get: get_actions_row_selection_opacity,
        apply: apply_actions_row_selection_opacity,
    },
    ActionsPopupKnob {
        id: ACTIONS_ROW_HOVER_OPACITY_KNOB_ID,
        label: "Action hover opacity",
        group: ActionsPopupKnobGroup::Row,
        unit: StyleUnit::Opacity,
        min: 0.0,
        max: 1.0,
        step: 0.01,
        get: get_actions_row_hover_opacity,
        apply: apply_actions_row_hover_opacity,
    },
    ActionsPopupKnob {
        id: ACTIONS_SECTION_PADDING_X_KNOB_ID,
        label: "Action section padding X",
        group: ActionsPopupKnobGroup::Section,
        unit: StyleUnit::Px,
        min: 0.0,
        max: 48.0,
        step: 1.0,
        get: get_actions_section_padding_x,
        apply: apply_actions_section_padding_x,
    },
    ActionsPopupKnob {
        id: ACTIONS_SECTION_PADDING_TOP_KNOB_ID,
        label: "Action section padding top",
        group: ActionsPopupKnobGroup::Section,
        unit: StyleUnit::Px,
        min: 0.0,
        max: 32.0,
        step: 1.0,
        get: get_actions_section_padding_top,
        apply: apply_actions_section_padding_top,
    },
    ActionsPopupKnob {
        id: ACTIONS_CONTEXT_HEADER_HEIGHT_KNOB_ID,
        label: "Action context header height",
        group: ActionsPopupKnobGroup::ContextHeader,
        unit: StyleUnit::Px,
        min: 0.0,
        max: 80.0,
        step: 1.0,
        get: get_actions_context_header_height,
        apply: apply_actions_context_header_height,
    },
    ActionsPopupKnob {
        id: ACTIONS_SHORTCUT_KEYCAP_HEIGHT_KNOB_ID,
        label: "Action shortcut keycap height",
        group: ActionsPopupKnobGroup::Shortcut,
        unit: StyleUnit::Px,
        min: 12.0,
        max: 40.0,
        step: 1.0,
        get: get_actions_shortcut_keycap_height,
        apply: apply_actions_shortcut_keycap_height,
    },
    ActionsPopupKnob {
        id: ACTIONS_SHORTCUT_KEYCAP_FONT_SIZE_KNOB_ID,
        label: "Action shortcut font size",
        group: ActionsPopupKnobGroup::Shortcut,
        unit: StyleUnit::Px,
        min: 8.0,
        max: 24.0,
        step: 1.0,
        get: get_actions_shortcut_keycap_font_size,
        apply: apply_actions_shortcut_keycap_font_size,
    },
];

pub fn actions_popup_knob_by_id(id: ActionsPopupKnobId) -> Option<&'static ActionsPopupKnob> {
    ACTIONS_POPUP_KNOBS.iter().find(|knob| knob.id == id)
}

pub fn actions_popup_knob_id_from_str(value: &str) -> Option<ActionsPopupKnobId> {
    let normalized = value
        .strip_prefix("control:dev-style-tool-actions:")
        .unwrap_or(value)
        .strip_prefix("slider:dev-style-tool-actions:")
        .unwrap_or_else(|| {
            value
                .strip_prefix("input:dev-style-tool-actions:")
                .unwrap_or(value)
        })
        .strip_prefix("button:dev-style-tool-actions-reset:")
        .unwrap_or(value);
    ACTIONS_POPUP_KNOBS
        .iter()
        .find(|knob| knob.id.as_str() == normalized)
        .map(|knob| knob.id)
}
