use gpui::FontWeight;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ActionsPopupThemeDef {
    pub shell: ActionsPopupShellTokens,
    pub search: ActionsPopupSearchTokens,
    pub list: ActionsPopupListTokens,
    pub row: ActionsPopupRowTokens,
    pub section: ActionsPopupSectionTokens,
    pub context_header: ActionsPopupContextHeaderTokens,
    pub shortcut: ActionsPopupShortcutTokens,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ActionsPopupShellTokens {
    pub width: f32,
    pub max_height: f32,
    pub notes_recent_max_height: f32,
    pub margin_x: f32,
    pub margin_y: f32,
    pub titlebar_offset_y: f32,
    pub border_height: f32,
    pub radius: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ActionsPopupSearchTokens {
    pub height: f32,
    pub inner_height: f32,
    pub padding_x: f32,
    pub padding_y_extra: f32,
    pub font_size: f32,
    pub cursor_width: f32,
    pub cursor_height: f32,
    pub prefix_gap: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ActionsPopupListTokens {
    pub row_height: f32,
    pub empty_row_height: f32,
    pub section_header_height: f32,
    pub padding_top: f32,
    pub padding_bottom: f32,
    pub overdraw_px: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ActionsPopupRowTokens {
    pub inset_x: f32,
    pub inner_y: f32,
    pub radius: f32,
    pub selection_opacity: f32,
    pub hover_opacity: f32,
    pub title_font_size: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ActionsPopupSectionTokens {
    pub padding_x: f32,
    pub padding_top: f32,
    pub padding_bottom: f32,
    pub font_size: f32,
    pub font_weight: FontWeight,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ActionsPopupContextHeaderTokens {
    pub height: f32,
    pub padding_x: f32,
    pub padding_top: f32,
    pub padding_bottom: f32,
    pub font_size: f32,
    pub font_weight: FontWeight,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ActionsPopupShortcutTokens {
    pub keycap_height: f32,
    pub keycap_padding_x: f32,
    pub keycap_padding_y: f32,
    pub keycap_font_size: f32,
    pub keycap_radius: f32,
}

pub fn base_actions_popup_theme() -> ActionsPopupThemeDef {
    ActionsPopupThemeDef {
        shell: ActionsPopupShellTokens {
            width: crate::actions::constants::POPUP_WIDTH,
            max_height: crate::actions::constants::POPUP_MAX_HEIGHT,
            notes_recent_max_height: crate::actions::constants::NOTES_RECENT_POPUP_MAX_HEIGHT,
            margin_x: 8.0,
            margin_y: 8.0,
            titlebar_offset_y: 36.0,
            border_height: 2.0,
            radius: crate::actions::constants::ACTIONS_POPUP_RADIUS,
        },
        search: ActionsPopupSearchTokens {
            height: 40.0,
            inner_height: 28.0,
            padding_x: crate::actions::constants::ACTION_PADDING_X,
            padding_y_extra: 2.0,
            font_size: 16.0,
            cursor_width: 2.0,
            cursor_height: 16.0,
            prefix_gap: 6.0,
        },
        list: ActionsPopupListTokens {
            row_height: 36.0,
            empty_row_height: 36.0,
            section_header_height: 24.0,
            padding_top: 0.0,
            padding_bottom: 0.0,
            overdraw_px: 100.0,
        },
        row: ActionsPopupRowTokens {
            inset_x: crate::actions::constants::ACTION_ROW_INSET,
            inner_y: 2.0,
            radius: crate::actions::constants::ACTIONS_ROW_RADIUS,
            selection_opacity: 0.72,
            hover_opacity: 0.56,
            title_font_size: 14.0,
        },
        section: ActionsPopupSectionTokens {
            padding_x: crate::actions::constants::ACTION_PADDING_X,
            padding_top: crate::actions::constants::ACTION_PADDING_TOP,
            padding_bottom: 4.0,
            font_size: 12.0,
            font_weight: FontWeight::SEMIBOLD,
        },
        context_header: ActionsPopupContextHeaderTokens {
            height: crate::actions::constants::HEADER_HEIGHT,
            padding_x: crate::actions::constants::ACTION_PADDING_X,
            padding_top: crate::actions::constants::ACTION_PADDING_TOP,
            padding_bottom: 4.0,
            font_size: 12.0,
            font_weight: FontWeight::SEMIBOLD,
        },
        shortcut: ActionsPopupShortcutTokens {
            keycap_height: crate::components::footer_chrome::FOOTER_KEYCAP_HEIGHT_PX,
            keycap_padding_x: crate::components::footer_chrome::FOOTER_KEYCAP_PADDING_X_PX,
            keycap_padding_y: 0.0,
            keycap_font_size: crate::components::footer_chrome::FOOTER_HINT_FONT_SIZE_PX,
            keycap_radius: crate::components::footer_chrome::FOOTER_KEYCAP_RADIUS_PX,
        },
    }
}

pub fn current_actions_popup_theme() -> ActionsPopupThemeDef {
    let mut def = crate::dev_style_tool::runtime_overrides::apply_to_actions_popup_def(
        base_actions_popup_theme(),
    );
    def.search.cursor_height = def.search.font_size;
    def
}
