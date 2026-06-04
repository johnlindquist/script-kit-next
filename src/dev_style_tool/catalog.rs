use crate::designs::MainMenuThemeDef;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct StyleKnobId(&'static str);

impl StyleKnobId {
    pub const fn new(value: &'static str) -> Self {
        Self(value)
    }

    pub const fn as_str(self) -> &'static str {
        self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum StyleValue {
    Number(f32),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StyleKnobGroup {
    Shell,
    Search,
    List,
    Row,
    Icon,
    Metadata,
    Typography,
    Footer,
    HeaderInfoBar,
}

impl StyleKnobGroup {
    pub const fn label(self) -> &'static str {
        match self {
            StyleKnobGroup::Shell => "Shell",
            StyleKnobGroup::Search => "Search input",
            StyleKnobGroup::List => "List",
            StyleKnobGroup::Row => "Rows",
            StyleKnobGroup::Icon => "Icons",
            StyleKnobGroup::Metadata => "Metadata",
            StyleKnobGroup::Typography => "Typography",
            StyleKnobGroup::Footer => "Footer",
            StyleKnobGroup::HeaderInfoBar => "Header info bar",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StyleUnit {
    Px,
    Alpha,
    Opacity,
}

impl StyleUnit {
    pub const fn label(self) -> &'static str {
        match self {
            StyleUnit::Px => "px",
            StyleUnit::Alpha => "alpha",
            StyleUnit::Opacity => "opacity",
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct StyleKnob {
    pub id: StyleKnobId,
    pub label: &'static str,
    pub group: StyleKnobGroup,
    pub unit: StyleUnit,
    pub min: f32,
    pub max: f32,
    pub step: f32,
    pub get: fn(&MainMenuThemeDef) -> StyleValue,
    pub apply: fn(&mut MainMenuThemeDef, StyleValue),
}

impl StyleKnob {
    pub fn clamp_value(self, value: StyleValue) -> StyleValue {
        match value {
            StyleValue::Number(number) => StyleValue::Number(number.clamp(self.min, self.max)),
        }
    }
}

macro_rules! f32_knob {
    ($id_const:ident, $get_fn:ident, $apply_fn:ident, $id:literal, $section:ident.$field:ident) => {
        pub const $id_const: StyleKnobId = StyleKnobId::new($id);
        fn $get_fn(def: &MainMenuThemeDef) -> StyleValue {
            StyleValue::Number(def.$section.$field)
        }
        fn $apply_fn(def: &mut MainMenuThemeDef, value: StyleValue) {
            let StyleValue::Number(value) = value;
            def.$section.$field = value;
        }
    };
}

macro_rules! f32_nested_knob {
    ($id_const:ident, $get_fn:ident, $apply_fn:ident, $id:literal, $section:ident.$nested:ident.$field:ident) => {
        pub const $id_const: StyleKnobId = StyleKnobId::new($id);
        fn $get_fn(def: &MainMenuThemeDef) -> StyleValue {
            StyleValue::Number(def.$section.$nested.$field)
        }
        fn $apply_fn(def: &mut MainMenuThemeDef, value: StyleValue) {
            let StyleValue::Number(value) = value;
            def.$section.$nested.$field = value;
        }
    };
}

macro_rules! alpha_knob {
    ($id_const:ident, $get_fn:ident, $apply_fn:ident, $id:literal, $section:ident.$field:ident) => {
        pub const $id_const: StyleKnobId = StyleKnobId::new($id);
        fn $get_fn(def: &MainMenuThemeDef) -> StyleValue {
            StyleValue::Number(def.$section.$field as f32)
        }
        fn $apply_fn(def: &mut MainMenuThemeDef, value: StyleValue) {
            let StyleValue::Number(value) = value;
            def.$section.$field = value.round().clamp(0.0, 255.0) as u32;
        }
    };
}

macro_rules! alpha_nested_knob {
    ($id_const:ident, $get_fn:ident, $apply_fn:ident, $id:literal, $section:ident.$nested:ident.$field:ident) => {
        pub const $id_const: StyleKnobId = StyleKnobId::new($id);
        fn $get_fn(def: &MainMenuThemeDef) -> StyleValue {
            StyleValue::Number(def.$section.$nested.$field as f32)
        }
        fn $apply_fn(def: &mut MainMenuThemeDef, value: StyleValue) {
            let StyleValue::Number(value) = value;
            def.$section.$nested.$field = value.round().clamp(0.0, 255.0) as u32;
        }
    };
}

f32_knob!(
    SHELL_CONTENT_INSET_X_KNOB_ID,
    get_shell_content_inset_x,
    apply_shell_content_inset_x,
    "shell.contentInsetX",
    shell.content_inset_x
);
f32_knob!(
    SHELL_CONTENT_INSET_BOTTOM_KNOB_ID,
    get_shell_content_inset_bottom,
    apply_shell_content_inset_bottom,
    "shell.contentInsetBottom",
    shell.content_inset_bottom
);
f32_knob!(
    SHELL_HEADER_PADDING_X_KNOB_ID,
    get_shell_header_padding_x,
    apply_shell_header_padding_x,
    "shell.headerPaddingX",
    shell.header_padding_x
);
f32_knob!(
    SHELL_HEADER_PADDING_Y_KNOB_ID,
    get_shell_header_padding_y,
    apply_shell_header_padding_y,
    "shell.headerPaddingY",
    shell.header_padding_y
);
f32_knob!(
    SHELL_HEADER_GAP_KNOB_ID,
    get_shell_header_gap,
    apply_shell_header_gap,
    "shell.headerGap",
    shell.header_gap
);
f32_knob!(
    SHELL_DIVIDER_MARGIN_X_KNOB_ID,
    get_shell_divider_margin_x,
    apply_shell_divider_margin_x,
    "shell.dividerMarginX",
    shell.divider_margin_x
);
f32_knob!(
    SHELL_DIVIDER_HEIGHT_KNOB_ID,
    get_shell_divider_height,
    apply_shell_divider_height,
    "shell.dividerHeight",
    shell.divider_height
);
alpha_knob!(
    SHELL_DIVIDER_ALPHA_KNOB_ID,
    get_shell_divider_alpha,
    apply_shell_divider_alpha,
    "shell.dividerAlpha",
    shell.divider_alpha
);

f32_knob!(
    SEARCH_HEIGHT_KNOB_ID,
    get_search_height,
    apply_search_height,
    "search.height",
    search.height
);
f32_knob!(
    SEARCH_RADIUS_KNOB_ID,
    get_search_radius,
    apply_search_radius,
    "search.radius",
    search.radius
);
f32_knob!(
    SEARCH_TEXT_INSET_X_KNOB_ID,
    get_search_text_inset_x,
    apply_search_text_inset_x,
    "search.textInsetX",
    search.text_inset_x
);
f32_knob!(
    SEARCH_TEXT_INSET_Y_KNOB_ID,
    get_search_text_inset_y,
    apply_search_text_inset_y,
    "search.textInsetY",
    search.text_inset_y
);
f32_knob!(
    SEARCH_FONT_SIZE_KNOB_ID,
    get_search_font_size,
    apply_search_font_size,
    "search.fontSize",
    search.font_size
);
alpha_knob!(
    SEARCH_SURFACE_ALPHA_KNOB_ID,
    get_search_surface_alpha,
    apply_search_surface_alpha,
    "search.surfaceAlpha",
    search.surface_alpha
);
alpha_knob!(
    SEARCH_BORDER_ALPHA_KNOB_ID,
    get_search_border_alpha,
    apply_search_border_alpha,
    "search.borderAlpha",
    search.border_alpha
);

f32_knob!(
    LIST_ITEM_HEIGHT_KNOB_ID,
    get_list_item_height,
    apply_list_item_height,
    "list.itemHeight",
    list.item_height
);
f32_knob!(
    LIST_SECTION_HEADER_HEIGHT_KNOB_ID,
    get_list_section_header_height,
    apply_list_section_header_height,
    "list.sectionHeaderHeight",
    list.section_header_height
);
f32_knob!(
    LIST_FIRST_SECTION_HEADER_HEIGHT_KNOB_ID,
    get_list_first_section_header_height,
    apply_list_first_section_header_height,
    "list.firstSectionHeaderHeight",
    list.first_section_header_height
);
f32_knob!(
    LIST_SOURCE_STATUS_ROW_HEIGHT_KNOB_ID,
    get_list_source_status_row_height,
    apply_list_source_status_row_height,
    "list.sourceStatusRowHeight",
    list.source_status_row_height
);
f32_knob!(
    LIST_AVERAGE_SCROLL_HEIGHT_KNOB_ID,
    get_list_average_scroll_height,
    apply_list_average_scroll_height,
    "list.averageScrollHeight",
    list.average_scroll_height
);
f32_knob!(
    LIST_SECTION_PADDING_X_KNOB_ID,
    get_list_section_padding_x,
    apply_list_section_padding_x,
    "list.sectionPaddingX",
    list.section_padding_x
);
f32_knob!(
    LIST_SECTION_PADDING_TOP_KNOB_ID,
    get_list_section_padding_top,
    apply_list_section_padding_top,
    "list.sectionPaddingTop",
    list.section_padding_top
);
f32_knob!(
    LIST_SECTION_PADDING_BOTTOM_KNOB_ID,
    get_list_section_padding_bottom,
    apply_list_section_padding_bottom,
    "list.sectionPaddingBottom",
    list.section_padding_bottom
);
f32_knob!(
    LIST_SECTION_GAP_KNOB_ID,
    get_list_section_gap,
    apply_list_section_gap,
    "list.sectionGap",
    list.section_gap
);
f32_knob!(
    LIST_SECTION_ICON_SIZE_KNOB_ID,
    get_list_section_icon_size,
    apply_list_section_icon_size,
    "list.sectionIconSize",
    list.section_icon_size
);

f32_knob!(
    ROW_OUTER_PADDING_X_KNOB_ID,
    get_row_outer_padding_x,
    apply_row_outer_padding_x,
    "row.outerPaddingX",
    row.outer_padding_x
);
f32_knob!(
    ROW_OUTER_PADDING_Y_KNOB_ID,
    get_row_outer_padding_y,
    apply_row_outer_padding_y,
    "row.outerPaddingY",
    row.outer_padding_y
);
f32_knob!(
    ROW_INNER_PADDING_X_KNOB_ID,
    get_row_inner_padding_x,
    apply_row_inner_padding_x,
    "row.innerPaddingX",
    row.inner_padding_x
);
f32_knob!(
    ROW_INNER_PADDING_Y_KNOB_ID,
    get_row_inner_padding_y,
    apply_row_inner_padding_y,
    "row.innerPaddingY",
    row.inner_padding_y
);
f32_knob!(
    ROW_RADIUS_KNOB_ID,
    get_row_radius,
    apply_row_radius,
    "row.radius",
    row.radius
);
f32_knob!(
    ROW_NAME_DESC_GAP_KNOB_ID,
    get_row_name_desc_gap,
    apply_row_name_desc_gap,
    "row.nameDescGap",
    row.name_desc_gap
);
f32_knob!(
    ROW_ICON_TEXT_GAP_KNOB_ID,
    get_row_icon_text_gap,
    apply_row_icon_text_gap,
    "row.iconTextGap",
    row.icon_text_gap
);
f32_knob!(
    ROW_ACCESSORY_GAP_KNOB_ID,
    get_row_accessory_gap,
    apply_row_accessory_gap,
    "row.accessoryGap",
    row.accessory_gap
);
f32_knob!(
    ROW_SELECTED_BORDER_WIDTH_KNOB_ID,
    get_row_selected_border_width,
    apply_row_selected_border_width,
    "row.selectedBorderWidth",
    row.selected_border_width
);
alpha_knob!(
    ROW_SELECTED_BORDER_ALPHA_KNOB_ID,
    get_row_selected_border_alpha,
    apply_row_selected_border_alpha,
    "row.selectedBorderAlpha",
    row.selected_border_alpha
);
alpha_knob!(
    ROW_SELECTED_FILL_ALPHA_KNOB_ID,
    get_row_selected_fill_alpha,
    apply_row_selected_fill_alpha,
    "row.selectedFillAlpha",
    row.selected_fill_alpha
);
alpha_knob!(
    ROW_HOVER_FILL_ALPHA_KNOB_ID,
    get_row_hover_fill_alpha,
    apply_row_hover_fill_alpha,
    "row.hoverFillAlpha",
    row.hover_fill_alpha
);

f32_knob!(
    ICON_CONTAINER_SIZE_KNOB_ID,
    get_icon_container_size,
    apply_icon_container_size,
    "icon.containerSize",
    icon.container_size
);
f32_knob!(
    ICON_SVG_SIZE_KNOB_ID,
    get_icon_svg_size,
    apply_icon_svg_size,
    "icon.svgSize",
    icon.svg_size
);
f32_knob!(
    ICON_TILE_SIZE_KNOB_ID,
    get_icon_tile_size,
    apply_icon_tile_size,
    "icon.tileSize",
    icon.tile_size
);
f32_knob!(
    ICON_TILE_RADIUS_KNOB_ID,
    get_icon_tile_radius,
    apply_icon_tile_radius,
    "icon.tileRadius",
    icon.tile_radius
);
alpha_knob!(
    ICON_TILE_FILL_ALPHA_KNOB_ID,
    get_icon_tile_fill_alpha,
    apply_icon_tile_fill_alpha,
    "icon.tileFillAlpha",
    icon.tile_fill_alpha
);
alpha_knob!(
    ICON_TILE_BORDER_ALPHA_KNOB_ID,
    get_icon_tile_border_alpha,
    apply_icon_tile_border_alpha,
    "icon.tileBorderAlpha",
    icon.tile_border_alpha
);

alpha_knob!(
    METADATA_ALPHA_KNOB_ID,
    get_metadata_alpha,
    apply_metadata_alpha,
    "metadata.alpha",
    metadata.metadata_alpha
);
f32_knob!(
    METADATA_TYPE_ACCESSORY_SIZE_KNOB_ID,
    get_metadata_type_accessory_size,
    apply_metadata_type_accessory_size,
    "metadata.typeAccessorySize",
    metadata.type_accessory_size
);
f32_knob!(
    METADATA_SOURCE_FONT_SIZE_KNOB_ID,
    get_metadata_source_font_size,
    apply_metadata_source_font_size,
    "metadata.sourceFontSize",
    metadata.source_font_size
);
f32_knob!(
    METADATA_BADGE_FONT_SIZE_KNOB_ID,
    get_metadata_badge_font_size,
    apply_metadata_badge_font_size,
    "metadata.badgeFontSize",
    metadata.badge_font_size
);
f32_knob!(
    METADATA_BADGE_PADDING_X_KNOB_ID,
    get_metadata_badge_padding_x,
    apply_metadata_badge_padding_x,
    "metadata.badgePaddingX",
    metadata.badge_padding_x
);
f32_knob!(
    METADATA_BADGE_PADDING_Y_KNOB_ID,
    get_metadata_badge_padding_y,
    apply_metadata_badge_padding_y,
    "metadata.badgePaddingY",
    metadata.badge_padding_y
);
f32_knob!(
    METADATA_BADGE_RADIUS_KNOB_ID,
    get_metadata_badge_radius,
    apply_metadata_badge_radius,
    "metadata.badgeRadius",
    metadata.badge_radius
);
f32_knob!(
    METADATA_KEYCAP_FONT_SIZE_KNOB_ID,
    get_metadata_keycap_font_size,
    apply_metadata_keycap_font_size,
    "metadata.keycapFontSize",
    metadata.keycap_font_size
);

f32_knob!(
    TYPOGRAPHY_NAME_FONT_SIZE_KNOB_ID,
    get_typography_name_font_size,
    apply_typography_name_font_size,
    "typography.nameFontSize",
    typography.name_font_size
);
f32_knob!(
    TYPOGRAPHY_NAME_LINE_HEIGHT_KNOB_ID,
    get_typography_name_line_height,
    apply_typography_name_line_height,
    "typography.nameLineHeight",
    typography.name_line_height
);
f32_knob!(
    TYPOGRAPHY_DESC_FONT_SIZE_KNOB_ID,
    get_typography_desc_font_size,
    apply_typography_desc_font_size,
    "typography.descFontSize",
    typography.desc_font_size
);
f32_knob!(
    TYPOGRAPHY_DESC_LINE_HEIGHT_KNOB_ID,
    get_typography_desc_line_height,
    apply_typography_desc_line_height,
    "typography.descLineHeight",
    typography.desc_line_height
);
f32_knob!(
    TYPOGRAPHY_SECTION_FONT_SIZE_KNOB_ID,
    get_typography_section_font_size,
    apply_typography_section_font_size,
    "typography.sectionFontSize",
    typography.section_font_size
);
f32_knob!(
    TYPOGRAPHY_SECTION_LINE_HEIGHT_KNOB_ID,
    get_typography_section_line_height,
    apply_typography_section_line_height,
    "typography.sectionLineHeight",
    typography.section_line_height
);

f32_nested_knob!(
    FOOTER_SIDE_INSET_KNOB_ID,
    get_footer_side_inset,
    apply_footer_side_inset,
    "footer.sideInset",
    footer.metrics.side_inset_px
);
f32_nested_knob!(
    FOOTER_ITEM_GAP_KNOB_ID,
    get_footer_item_gap,
    apply_footer_item_gap,
    "footer.itemGap",
    footer.metrics.item_gap_px
);
f32_nested_knob!(
    FOOTER_CONTENT_GAP_KNOB_ID,
    get_footer_content_gap,
    apply_footer_content_gap,
    "footer.contentGap",
    footer.metrics.content_gap
);
f32_nested_knob!(
    FOOTER_BUTTON_PADDING_X_KNOB_ID,
    get_footer_button_padding_x,
    apply_footer_button_padding_x,
    "footer.buttonPaddingX",
    footer.metrics.button_padding_x
);
f32_nested_knob!(
    FOOTER_BUTTON_PADDING_Y_KNOB_ID,
    get_footer_button_padding_y,
    apply_footer_button_padding_y,
    "footer.buttonPaddingY",
    footer.metrics.button_padding_y
);
f32_nested_knob!(
    FOOTER_RUN_BUTTON_PADDING_X_KNOB_ID,
    get_footer_run_button_padding_x,
    apply_footer_run_button_padding_x,
    "footer.runButtonPaddingX",
    footer.metrics.run_button_padding_x
);
f32_nested_knob!(
    FOOTER_BUTTON_RADIUS_KNOB_ID,
    get_footer_button_radius,
    apply_footer_button_radius,
    "footer.buttonRadius",
    footer.metrics.button_radius
);
f32_nested_knob!(
    FOOTER_KEYCAP_PADDING_X_KNOB_ID,
    get_footer_keycap_padding_x,
    apply_footer_keycap_padding_x,
    "footer.keycapPaddingX",
    footer.metrics.keycap_padding_x
);
f32_nested_knob!(
    FOOTER_KEYCAP_PADDING_Y_KNOB_ID,
    get_footer_keycap_padding_y,
    apply_footer_keycap_padding_y,
    "footer.keycapPaddingY",
    footer.metrics.keycap_padding_y
);
f32_nested_knob!(
    FOOTER_KEYCAP_RADIUS_KNOB_ID,
    get_footer_keycap_radius,
    apply_footer_keycap_radius,
    "footer.keycapRadius",
    footer.metrics.keycap_radius
);
f32_nested_knob!(
    FOOTER_LABEL_FONT_SIZE_KNOB_ID,
    get_footer_label_font_size,
    apply_footer_label_font_size,
    "footer.labelFontSize",
    footer.metrics.label_font_size
);
f32_nested_knob!(
    FOOTER_KEYCAP_FONT_SIZE_KNOB_ID,
    get_footer_keycap_font_size,
    apply_footer_keycap_font_size,
    "footer.keycapFontSize",
    footer.metrics.keycap_font_size
);
alpha_knob!(
    FOOTER_DIVIDER_ALPHA_KNOB_ID,
    get_footer_divider_alpha,
    apply_footer_divider_alpha,
    "footer.dividerAlpha",
    footer.divider_alpha
);
alpha_nested_knob!(
    FOOTER_BUTTON_BORDER_ALPHA_KNOB_ID,
    get_footer_button_border_alpha,
    apply_footer_button_border_alpha,
    "footer.buttonBorderAlpha",
    footer.button.border_alpha
);

f32_knob!(
    HEADER_INFO_FONT_SIZE_KNOB_ID,
    get_header_info_font_size,
    apply_header_info_font_size,
    "headerInfo.fontSize",
    header_info_bar.font_size
);
f32_knob!(
    HEADER_INFO_HEIGHT_KNOB_ID,
    get_header_info_height,
    apply_header_info_height,
    "headerInfo.height",
    header_info_bar.height_px
);
f32_knob!(
    HEADER_INFO_GAP_KNOB_ID,
    get_header_info_gap,
    apply_header_info_gap,
    "headerInfo.gap",
    header_info_bar.gap_px
);
f32_knob!(
    HEADER_INFO_PILL_PADDING_X_KNOB_ID,
    get_header_info_pill_padding_x,
    apply_header_info_pill_padding_x,
    "headerInfo.pillPaddingX",
    header_info_bar.pill_padding_x
);
f32_knob!(
    HEADER_INFO_PILL_PADDING_Y_KNOB_ID,
    get_header_info_pill_padding_y,
    apply_header_info_pill_padding_y,
    "headerInfo.pillPaddingY",
    header_info_bar.pill_padding_y
);
f32_knob!(
    HEADER_INFO_PILL_RADIUS_KNOB_ID,
    get_header_info_pill_radius,
    apply_header_info_pill_radius,
    "headerInfo.pillRadius",
    header_info_bar.pill_radius
);
f32_knob!(
    HEADER_INFO_OPACITY_KNOB_ID,
    get_header_info_opacity,
    apply_header_info_opacity,
    "headerInfo.opacity",
    header_info_bar.opacity
);
f32_knob!(
    HEADER_INFO_KEY_OPACITY_KNOB_ID,
    get_header_info_key_opacity,
    apply_header_info_key_opacity,
    "headerInfo.keyOpacity",
    header_info_bar.key_opacity
);
alpha_knob!(
    HEADER_INFO_PILL_BG_ALPHA_KNOB_ID,
    get_header_info_pill_bg_alpha,
    apply_header_info_pill_bg_alpha,
    "headerInfo.pillBgAlpha",
    header_info_bar.pill_bg_alpha
);
alpha_knob!(
    HEADER_INFO_PILL_BORDER_ALPHA_KNOB_ID,
    get_header_info_pill_border_alpha,
    apply_header_info_pill_border_alpha,
    "headerInfo.pillBorderAlpha",
    header_info_bar.pill_border_alpha
);
f32_knob!(
    HEADER_INFO_CONTEXT_EDGE_OUTSET_X_KNOB_ID,
    get_header_info_context_edge_outset_x,
    apply_header_info_context_edge_outset_x,
    "headerInfo.contextEdgeOutsetX",
    header_info_bar.context_edge_outset_x
);
f32_knob!(
    HEADER_INFO_VARIATION_BADGE_WIDTH_KNOB_ID,
    get_header_info_variation_badge_width,
    apply_header_info_variation_badge_width,
    "headerInfo.variationBadgeWidth",
    header_info_bar.variation_badge_width_px
);

pub const STYLE_KNOBS: &[StyleKnob] = &[
    StyleKnob {
        id: SHELL_CONTENT_INSET_X_KNOB_ID,
        label: "Content inset X",
        group: StyleKnobGroup::Shell,
        unit: StyleUnit::Px,
        min: 0.0,
        max: 48.0,
        step: 1.0,
        get: get_shell_content_inset_x,
        apply: apply_shell_content_inset_x,
    },
    StyleKnob {
        id: SHELL_CONTENT_INSET_BOTTOM_KNOB_ID,
        label: "Content inset bottom",
        group: StyleKnobGroup::Shell,
        unit: StyleUnit::Px,
        min: 0.0,
        max: 48.0,
        step: 1.0,
        get: get_shell_content_inset_bottom,
        apply: apply_shell_content_inset_bottom,
    },
    StyleKnob {
        id: SHELL_HEADER_PADDING_X_KNOB_ID,
        label: "Header padding X",
        group: StyleKnobGroup::Shell,
        unit: StyleUnit::Px,
        min: 0.0,
        max: 48.0,
        step: 1.0,
        get: get_shell_header_padding_x,
        apply: apply_shell_header_padding_x,
    },
    StyleKnob {
        id: SHELL_HEADER_PADDING_Y_KNOB_ID,
        label: "Header padding Y",
        group: StyleKnobGroup::Shell,
        unit: StyleUnit::Px,
        min: 0.0,
        max: 32.0,
        step: 1.0,
        get: get_shell_header_padding_y,
        apply: apply_shell_header_padding_y,
    },
    StyleKnob {
        id: SHELL_HEADER_GAP_KNOB_ID,
        label: "Header gap",
        group: StyleKnobGroup::Shell,
        unit: StyleUnit::Px,
        min: 0.0,
        max: 32.0,
        step: 1.0,
        get: get_shell_header_gap,
        apply: apply_shell_header_gap,
    },
    StyleKnob {
        id: SHELL_DIVIDER_MARGIN_X_KNOB_ID,
        label: "Divider margin X",
        group: StyleKnobGroup::Shell,
        unit: StyleUnit::Px,
        min: 0.0,
        max: 64.0,
        step: 1.0,
        get: get_shell_divider_margin_x,
        apply: apply_shell_divider_margin_x,
    },
    StyleKnob {
        id: SHELL_DIVIDER_HEIGHT_KNOB_ID,
        label: "Divider height",
        group: StyleKnobGroup::Shell,
        unit: StyleUnit::Px,
        min: 0.0,
        max: 8.0,
        step: 0.5,
        get: get_shell_divider_height,
        apply: apply_shell_divider_height,
    },
    StyleKnob {
        id: SHELL_DIVIDER_ALPHA_KNOB_ID,
        label: "Divider alpha",
        group: StyleKnobGroup::Shell,
        unit: StyleUnit::Alpha,
        min: 0.0,
        max: 255.0,
        step: 1.0,
        get: get_shell_divider_alpha,
        apply: apply_shell_divider_alpha,
    },
    StyleKnob {
        id: SEARCH_HEIGHT_KNOB_ID,
        label: "Main input height",
        group: StyleKnobGroup::Search,
        unit: StyleUnit::Px,
        min: 20.0,
        max: 96.0,
        step: 1.0,
        get: get_search_height,
        apply: apply_search_height,
    },
    StyleKnob {
        id: SEARCH_RADIUS_KNOB_ID,
        label: "Input radius",
        group: StyleKnobGroup::Search,
        unit: StyleUnit::Px,
        min: 0.0,
        max: 32.0,
        step: 1.0,
        get: get_search_radius,
        apply: apply_search_radius,
    },
    StyleKnob {
        id: SEARCH_TEXT_INSET_X_KNOB_ID,
        label: "Input text inset X",
        group: StyleKnobGroup::Search,
        unit: StyleUnit::Px,
        min: 0.0,
        max: 48.0,
        step: 1.0,
        get: get_search_text_inset_x,
        apply: apply_search_text_inset_x,
    },
    StyleKnob {
        id: SEARCH_TEXT_INSET_Y_KNOB_ID,
        label: "Input text inset Y",
        group: StyleKnobGroup::Search,
        unit: StyleUnit::Px,
        min: 0.0,
        max: 24.0,
        step: 1.0,
        get: get_search_text_inset_y,
        apply: apply_search_text_inset_y,
    },
    StyleKnob {
        id: SEARCH_FONT_SIZE_KNOB_ID,
        label: "Input font size",
        group: StyleKnobGroup::Search,
        unit: StyleUnit::Px,
        min: 8.0,
        max: 36.0,
        step: 1.0,
        get: get_search_font_size,
        apply: apply_search_font_size,
    },
    StyleKnob {
        id: SEARCH_SURFACE_ALPHA_KNOB_ID,
        label: "Input surface alpha",
        group: StyleKnobGroup::Search,
        unit: StyleUnit::Alpha,
        min: 0.0,
        max: 255.0,
        step: 1.0,
        get: get_search_surface_alpha,
        apply: apply_search_surface_alpha,
    },
    StyleKnob {
        id: SEARCH_BORDER_ALPHA_KNOB_ID,
        label: "Input border alpha",
        group: StyleKnobGroup::Search,
        unit: StyleUnit::Alpha,
        min: 0.0,
        max: 255.0,
        step: 1.0,
        get: get_search_border_alpha,
        apply: apply_search_border_alpha,
    },
    StyleKnob {
        id: LIST_ITEM_HEIGHT_KNOB_ID,
        label: "Item height",
        group: StyleKnobGroup::List,
        unit: StyleUnit::Px,
        min: 24.0,
        max: 96.0,
        step: 1.0,
        get: get_list_item_height,
        apply: apply_list_item_height,
    },
    StyleKnob {
        id: LIST_SECTION_HEADER_HEIGHT_KNOB_ID,
        label: "Section header height",
        group: StyleKnobGroup::List,
        unit: StyleUnit::Px,
        min: 0.0,
        max: 72.0,
        step: 1.0,
        get: get_list_section_header_height,
        apply: apply_list_section_header_height,
    },
    StyleKnob {
        id: LIST_FIRST_SECTION_HEADER_HEIGHT_KNOB_ID,
        label: "First section header height",
        group: StyleKnobGroup::List,
        unit: StyleUnit::Px,
        min: 0.0,
        max: 72.0,
        step: 1.0,
        get: get_list_first_section_header_height,
        apply: apply_list_first_section_header_height,
    },
    StyleKnob {
        id: LIST_SOURCE_STATUS_ROW_HEIGHT_KNOB_ID,
        label: "Source status row height",
        group: StyleKnobGroup::List,
        unit: StyleUnit::Px,
        min: 12.0,
        max: 72.0,
        step: 1.0,
        get: get_list_source_status_row_height,
        apply: apply_list_source_status_row_height,
    },
    StyleKnob {
        id: LIST_AVERAGE_SCROLL_HEIGHT_KNOB_ID,
        label: "Average scroll height",
        group: StyleKnobGroup::List,
        unit: StyleUnit::Px,
        min: 8.0,
        max: 160.0,
        step: 1.0,
        get: get_list_average_scroll_height,
        apply: apply_list_average_scroll_height,
    },
    StyleKnob {
        id: LIST_SECTION_PADDING_X_KNOB_ID,
        label: "Section padding X",
        group: StyleKnobGroup::List,
        unit: StyleUnit::Px,
        min: 0.0,
        max: 48.0,
        step: 1.0,
        get: get_list_section_padding_x,
        apply: apply_list_section_padding_x,
    },
    StyleKnob {
        id: LIST_SECTION_PADDING_TOP_KNOB_ID,
        label: "Section padding top",
        group: StyleKnobGroup::List,
        unit: StyleUnit::Px,
        min: 0.0,
        max: 48.0,
        step: 1.0,
        get: get_list_section_padding_top,
        apply: apply_list_section_padding_top,
    },
    StyleKnob {
        id: LIST_SECTION_PADDING_BOTTOM_KNOB_ID,
        label: "Section padding bottom",
        group: StyleKnobGroup::List,
        unit: StyleUnit::Px,
        min: 0.0,
        max: 24.0,
        step: 1.0,
        get: get_list_section_padding_bottom,
        apply: apply_list_section_padding_bottom,
    },
    StyleKnob {
        id: LIST_SECTION_GAP_KNOB_ID,
        label: "Section content gap",
        group: StyleKnobGroup::List,
        unit: StyleUnit::Px,
        min: 0.0,
        max: 24.0,
        step: 1.0,
        get: get_list_section_gap,
        apply: apply_list_section_gap,
    },
    StyleKnob {
        id: LIST_SECTION_ICON_SIZE_KNOB_ID,
        label: "Section icon size",
        group: StyleKnobGroup::List,
        unit: StyleUnit::Px,
        min: 0.0,
        max: 32.0,
        step: 1.0,
        get: get_list_section_icon_size,
        apply: apply_list_section_icon_size,
    },
    StyleKnob {
        id: ROW_OUTER_PADDING_X_KNOB_ID,
        label: "Item outer padding X",
        group: StyleKnobGroup::Row,
        unit: StyleUnit::Px,
        min: 0.0,
        max: 32.0,
        step: 1.0,
        get: get_row_outer_padding_x,
        apply: apply_row_outer_padding_x,
    },
    StyleKnob {
        id: ROW_OUTER_PADDING_Y_KNOB_ID,
        label: "Item outer padding Y",
        group: StyleKnobGroup::Row,
        unit: StyleUnit::Px,
        min: 0.0,
        max: 24.0,
        step: 1.0,
        get: get_row_outer_padding_y,
        apply: apply_row_outer_padding_y,
    },
    StyleKnob {
        id: ROW_INNER_PADDING_X_KNOB_ID,
        label: "Item inner padding X",
        group: StyleKnobGroup::Row,
        unit: StyleUnit::Px,
        min: 0.0,
        max: 48.0,
        step: 1.0,
        get: get_row_inner_padding_x,
        apply: apply_row_inner_padding_x,
    },
    StyleKnob {
        id: ROW_INNER_PADDING_Y_KNOB_ID,
        label: "Item inner padding Y",
        group: StyleKnobGroup::Row,
        unit: StyleUnit::Px,
        min: 0.0,
        max: 24.0,
        step: 1.0,
        get: get_row_inner_padding_y,
        apply: apply_row_inner_padding_y,
    },
    StyleKnob {
        id: ROW_RADIUS_KNOB_ID,
        label: "Item radius",
        group: StyleKnobGroup::Row,
        unit: StyleUnit::Px,
        min: 0.0,
        max: 32.0,
        step: 1.0,
        get: get_row_radius,
        apply: apply_row_radius,
    },
    StyleKnob {
        id: ROW_NAME_DESC_GAP_KNOB_ID,
        label: "Name/description gap",
        group: StyleKnobGroup::Row,
        unit: StyleUnit::Px,
        min: 0.0,
        max: 16.0,
        step: 0.5,
        get: get_row_name_desc_gap,
        apply: apply_row_name_desc_gap,
    },
    StyleKnob {
        id: ROW_ICON_TEXT_GAP_KNOB_ID,
        label: "Icon/text gap",
        group: StyleKnobGroup::Row,
        unit: StyleUnit::Px,
        min: 0.0,
        max: 24.0,
        step: 1.0,
        get: get_row_icon_text_gap,
        apply: apply_row_icon_text_gap,
    },
    StyleKnob {
        id: ROW_ACCESSORY_GAP_KNOB_ID,
        label: "Accessory gap",
        group: StyleKnobGroup::Row,
        unit: StyleUnit::Px,
        min: 0.0,
        max: 24.0,
        step: 1.0,
        get: get_row_accessory_gap,
        apply: apply_row_accessory_gap,
    },
    StyleKnob {
        id: ROW_SELECTED_BORDER_WIDTH_KNOB_ID,
        label: "Selected border width",
        group: StyleKnobGroup::Row,
        unit: StyleUnit::Px,
        min: 0.0,
        max: 6.0,
        step: 0.5,
        get: get_row_selected_border_width,
        apply: apply_row_selected_border_width,
    },
    StyleKnob {
        id: ROW_SELECTED_BORDER_ALPHA_KNOB_ID,
        label: "Selected border alpha",
        group: StyleKnobGroup::Row,
        unit: StyleUnit::Alpha,
        min: 0.0,
        max: 255.0,
        step: 1.0,
        get: get_row_selected_border_alpha,
        apply: apply_row_selected_border_alpha,
    },
    StyleKnob {
        id: ROW_SELECTED_FILL_ALPHA_KNOB_ID,
        label: "Selected fill alpha",
        group: StyleKnobGroup::Row,
        unit: StyleUnit::Alpha,
        min: 0.0,
        max: 255.0,
        step: 1.0,
        get: get_row_selected_fill_alpha,
        apply: apply_row_selected_fill_alpha,
    },
    StyleKnob {
        id: ROW_HOVER_FILL_ALPHA_KNOB_ID,
        label: "Hover fill alpha",
        group: StyleKnobGroup::Row,
        unit: StyleUnit::Alpha,
        min: 0.0,
        max: 255.0,
        step: 1.0,
        get: get_row_hover_fill_alpha,
        apply: apply_row_hover_fill_alpha,
    },
    StyleKnob {
        id: ICON_CONTAINER_SIZE_KNOB_ID,
        label: "Icon container size",
        group: StyleKnobGroup::Icon,
        unit: StyleUnit::Px,
        min: 0.0,
        max: 48.0,
        step: 1.0,
        get: get_icon_container_size,
        apply: apply_icon_container_size,
    },
    StyleKnob {
        id: ICON_SVG_SIZE_KNOB_ID,
        label: "Icon SVG size",
        group: StyleKnobGroup::Icon,
        unit: StyleUnit::Px,
        min: 0.0,
        max: 48.0,
        step: 1.0,
        get: get_icon_svg_size,
        apply: apply_icon_svg_size,
    },
    StyleKnob {
        id: ICON_TILE_SIZE_KNOB_ID,
        label: "Icon tile size",
        group: StyleKnobGroup::Icon,
        unit: StyleUnit::Px,
        min: 0.0,
        max: 48.0,
        step: 1.0,
        get: get_icon_tile_size,
        apply: apply_icon_tile_size,
    },
    StyleKnob {
        id: ICON_TILE_RADIUS_KNOB_ID,
        label: "Icon tile radius",
        group: StyleKnobGroup::Icon,
        unit: StyleUnit::Px,
        min: 0.0,
        max: 24.0,
        step: 1.0,
        get: get_icon_tile_radius,
        apply: apply_icon_tile_radius,
    },
    StyleKnob {
        id: ICON_TILE_FILL_ALPHA_KNOB_ID,
        label: "Icon tile fill alpha",
        group: StyleKnobGroup::Icon,
        unit: StyleUnit::Alpha,
        min: 0.0,
        max: 255.0,
        step: 1.0,
        get: get_icon_tile_fill_alpha,
        apply: apply_icon_tile_fill_alpha,
    },
    StyleKnob {
        id: ICON_TILE_BORDER_ALPHA_KNOB_ID,
        label: "Icon tile border alpha",
        group: StyleKnobGroup::Icon,
        unit: StyleUnit::Alpha,
        min: 0.0,
        max: 255.0,
        step: 1.0,
        get: get_icon_tile_border_alpha,
        apply: apply_icon_tile_border_alpha,
    },
    StyleKnob {
        id: METADATA_ALPHA_KNOB_ID,
        label: "Metadata alpha",
        group: StyleKnobGroup::Metadata,
        unit: StyleUnit::Alpha,
        min: 0.0,
        max: 255.0,
        step: 1.0,
        get: get_metadata_alpha,
        apply: apply_metadata_alpha,
    },
    StyleKnob {
        id: METADATA_TYPE_ACCESSORY_SIZE_KNOB_ID,
        label: "Type accessory size",
        group: StyleKnobGroup::Metadata,
        unit: StyleUnit::Px,
        min: 0.0,
        max: 48.0,
        step: 1.0,
        get: get_metadata_type_accessory_size,
        apply: apply_metadata_type_accessory_size,
    },
    StyleKnob {
        id: METADATA_SOURCE_FONT_SIZE_KNOB_ID,
        label: "Source font size",
        group: StyleKnobGroup::Metadata,
        unit: StyleUnit::Px,
        min: 8.0,
        max: 24.0,
        step: 1.0,
        get: get_metadata_source_font_size,
        apply: apply_metadata_source_font_size,
    },
    StyleKnob {
        id: METADATA_BADGE_FONT_SIZE_KNOB_ID,
        label: "Badge font size",
        group: StyleKnobGroup::Metadata,
        unit: StyleUnit::Px,
        min: 8.0,
        max: 24.0,
        step: 1.0,
        get: get_metadata_badge_font_size,
        apply: apply_metadata_badge_font_size,
    },
    StyleKnob {
        id: METADATA_BADGE_PADDING_X_KNOB_ID,
        label: "Badge padding X",
        group: StyleKnobGroup::Metadata,
        unit: StyleUnit::Px,
        min: 0.0,
        max: 24.0,
        step: 0.5,
        get: get_metadata_badge_padding_x,
        apply: apply_metadata_badge_padding_x,
    },
    StyleKnob {
        id: METADATA_BADGE_PADDING_Y_KNOB_ID,
        label: "Badge padding Y",
        group: StyleKnobGroup::Metadata,
        unit: StyleUnit::Px,
        min: 0.0,
        max: 16.0,
        step: 0.5,
        get: get_metadata_badge_padding_y,
        apply: apply_metadata_badge_padding_y,
    },
    StyleKnob {
        id: METADATA_BADGE_RADIUS_KNOB_ID,
        label: "Badge radius",
        group: StyleKnobGroup::Metadata,
        unit: StyleUnit::Px,
        min: 0.0,
        max: 24.0,
        step: 0.5,
        get: get_metadata_badge_radius,
        apply: apply_metadata_badge_radius,
    },
    StyleKnob {
        id: METADATA_KEYCAP_FONT_SIZE_KNOB_ID,
        label: "Metadata keycap font size",
        group: StyleKnobGroup::Metadata,
        unit: StyleUnit::Px,
        min: 8.0,
        max: 24.0,
        step: 1.0,
        get: get_metadata_keycap_font_size,
        apply: apply_metadata_keycap_font_size,
    },
    StyleKnob {
        id: TYPOGRAPHY_NAME_FONT_SIZE_KNOB_ID,
        label: "Name font size",
        group: StyleKnobGroup::Typography,
        unit: StyleUnit::Px,
        min: 8.0,
        max: 28.0,
        step: 1.0,
        get: get_typography_name_font_size,
        apply: apply_typography_name_font_size,
    },
    StyleKnob {
        id: TYPOGRAPHY_NAME_LINE_HEIGHT_KNOB_ID,
        label: "Name line height",
        group: StyleKnobGroup::Typography,
        unit: StyleUnit::Px,
        min: 8.0,
        max: 36.0,
        step: 1.0,
        get: get_typography_name_line_height,
        apply: apply_typography_name_line_height,
    },
    StyleKnob {
        id: TYPOGRAPHY_DESC_FONT_SIZE_KNOB_ID,
        label: "Description font size",
        group: StyleKnobGroup::Typography,
        unit: StyleUnit::Px,
        min: 8.0,
        max: 24.0,
        step: 1.0,
        get: get_typography_desc_font_size,
        apply: apply_typography_desc_font_size,
    },
    StyleKnob {
        id: TYPOGRAPHY_DESC_LINE_HEIGHT_KNOB_ID,
        label: "Description line height",
        group: StyleKnobGroup::Typography,
        unit: StyleUnit::Px,
        min: 8.0,
        max: 32.0,
        step: 1.0,
        get: get_typography_desc_line_height,
        apply: apply_typography_desc_line_height,
    },
    StyleKnob {
        id: TYPOGRAPHY_SECTION_FONT_SIZE_KNOB_ID,
        label: "Section font size",
        group: StyleKnobGroup::Typography,
        unit: StyleUnit::Px,
        min: 8.0,
        max: 24.0,
        step: 1.0,
        get: get_typography_section_font_size,
        apply: apply_typography_section_font_size,
    },
    StyleKnob {
        id: TYPOGRAPHY_SECTION_LINE_HEIGHT_KNOB_ID,
        label: "Section line height",
        group: StyleKnobGroup::Typography,
        unit: StyleUnit::Px,
        min: 8.0,
        max: 32.0,
        step: 1.0,
        get: get_typography_section_line_height,
        apply: apply_typography_section_line_height,
    },
    StyleKnob {
        id: FOOTER_SIDE_INSET_KNOB_ID,
        label: "Footer side inset",
        group: StyleKnobGroup::Footer,
        unit: StyleUnit::Px,
        min: 0.0,
        max: 64.0,
        step: 1.0,
        get: get_footer_side_inset,
        apply: apply_footer_side_inset,
    },
    StyleKnob {
        id: FOOTER_ITEM_GAP_KNOB_ID,
        label: "Footer item gap",
        group: StyleKnobGroup::Footer,
        unit: StyleUnit::Px,
        min: 0.0,
        max: 32.0,
        step: 1.0,
        get: get_footer_item_gap,
        apply: apply_footer_item_gap,
    },
    StyleKnob {
        id: FOOTER_CONTENT_GAP_KNOB_ID,
        label: "Footer content gap",
        group: StyleKnobGroup::Footer,
        unit: StyleUnit::Px,
        min: 0.0,
        max: 24.0,
        step: 1.0,
        get: get_footer_content_gap,
        apply: apply_footer_content_gap,
    },
    StyleKnob {
        id: FOOTER_BUTTON_PADDING_X_KNOB_ID,
        label: "Footer button padding X",
        group: StyleKnobGroup::Footer,
        unit: StyleUnit::Px,
        min: 0.0,
        max: 32.0,
        step: 1.0,
        get: get_footer_button_padding_x,
        apply: apply_footer_button_padding_x,
    },
    StyleKnob {
        id: FOOTER_BUTTON_PADDING_Y_KNOB_ID,
        label: "Footer button padding Y",
        group: StyleKnobGroup::Footer,
        unit: StyleUnit::Px,
        min: 0.0,
        max: 24.0,
        step: 1.0,
        get: get_footer_button_padding_y,
        apply: apply_footer_button_padding_y,
    },
    StyleKnob {
        id: FOOTER_RUN_BUTTON_PADDING_X_KNOB_ID,
        label: "Run button padding X",
        group: StyleKnobGroup::Footer,
        unit: StyleUnit::Px,
        min: 0.0,
        max: 40.0,
        step: 1.0,
        get: get_footer_run_button_padding_x,
        apply: apply_footer_run_button_padding_x,
    },
    StyleKnob {
        id: FOOTER_BUTTON_RADIUS_KNOB_ID,
        label: "Footer button radius",
        group: StyleKnobGroup::Footer,
        unit: StyleUnit::Px,
        min: 0.0,
        max: 24.0,
        step: 1.0,
        get: get_footer_button_radius,
        apply: apply_footer_button_radius,
    },
    StyleKnob {
        id: FOOTER_KEYCAP_PADDING_X_KNOB_ID,
        label: "Footer keycap padding X",
        group: StyleKnobGroup::Footer,
        unit: StyleUnit::Px,
        min: 0.0,
        max: 24.0,
        step: 1.0,
        get: get_footer_keycap_padding_x,
        apply: apply_footer_keycap_padding_x,
    },
    StyleKnob {
        id: FOOTER_KEYCAP_PADDING_Y_KNOB_ID,
        label: "Footer keycap padding Y",
        group: StyleKnobGroup::Footer,
        unit: StyleUnit::Px,
        min: 0.0,
        max: 16.0,
        step: 1.0,
        get: get_footer_keycap_padding_y,
        apply: apply_footer_keycap_padding_y,
    },
    StyleKnob {
        id: FOOTER_KEYCAP_RADIUS_KNOB_ID,
        label: "Footer keycap radius",
        group: StyleKnobGroup::Footer,
        unit: StyleUnit::Px,
        min: 0.0,
        max: 24.0,
        step: 1.0,
        get: get_footer_keycap_radius,
        apply: apply_footer_keycap_radius,
    },
    StyleKnob {
        id: FOOTER_LABEL_FONT_SIZE_KNOB_ID,
        label: "Footer label font size",
        group: StyleKnobGroup::Footer,
        unit: StyleUnit::Px,
        min: 8.0,
        max: 24.0,
        step: 1.0,
        get: get_footer_label_font_size,
        apply: apply_footer_label_font_size,
    },
    StyleKnob {
        id: FOOTER_KEYCAP_FONT_SIZE_KNOB_ID,
        label: "Footer keycap font size",
        group: StyleKnobGroup::Footer,
        unit: StyleUnit::Px,
        min: 8.0,
        max: 24.0,
        step: 1.0,
        get: get_footer_keycap_font_size,
        apply: apply_footer_keycap_font_size,
    },
    StyleKnob {
        id: FOOTER_DIVIDER_ALPHA_KNOB_ID,
        label: "Footer divider alpha",
        group: StyleKnobGroup::Footer,
        unit: StyleUnit::Alpha,
        min: 0.0,
        max: 255.0,
        step: 1.0,
        get: get_footer_divider_alpha,
        apply: apply_footer_divider_alpha,
    },
    StyleKnob {
        id: FOOTER_BUTTON_BORDER_ALPHA_KNOB_ID,
        label: "Footer button border alpha",
        group: StyleKnobGroup::Footer,
        unit: StyleUnit::Alpha,
        min: 0.0,
        max: 255.0,
        step: 1.0,
        get: get_footer_button_border_alpha,
        apply: apply_footer_button_border_alpha,
    },
    StyleKnob {
        id: HEADER_INFO_FONT_SIZE_KNOB_ID,
        label: "Header info font size",
        group: StyleKnobGroup::HeaderInfoBar,
        unit: StyleUnit::Px,
        min: 8.0,
        max: 24.0,
        step: 1.0,
        get: get_header_info_font_size,
        apply: apply_header_info_font_size,
    },
    StyleKnob {
        id: HEADER_INFO_HEIGHT_KNOB_ID,
        label: "Header info height",
        group: StyleKnobGroup::HeaderInfoBar,
        unit: StyleUnit::Px,
        min: 0.0,
        max: 48.0,
        step: 1.0,
        get: get_header_info_height,
        apply: apply_header_info_height,
    },
    StyleKnob {
        id: HEADER_INFO_GAP_KNOB_ID,
        label: "Header info gap",
        group: StyleKnobGroup::HeaderInfoBar,
        unit: StyleUnit::Px,
        min: 0.0,
        max: 24.0,
        step: 1.0,
        get: get_header_info_gap,
        apply: apply_header_info_gap,
    },
    StyleKnob {
        id: HEADER_INFO_PILL_PADDING_X_KNOB_ID,
        label: "Header pill padding X",
        group: StyleKnobGroup::HeaderInfoBar,
        unit: StyleUnit::Px,
        min: 0.0,
        max: 24.0,
        step: 1.0,
        get: get_header_info_pill_padding_x,
        apply: apply_header_info_pill_padding_x,
    },
    StyleKnob {
        id: HEADER_INFO_PILL_PADDING_Y_KNOB_ID,
        label: "Header pill padding Y",
        group: StyleKnobGroup::HeaderInfoBar,
        unit: StyleUnit::Px,
        min: 0.0,
        max: 16.0,
        step: 1.0,
        get: get_header_info_pill_padding_y,
        apply: apply_header_info_pill_padding_y,
    },
    StyleKnob {
        id: HEADER_INFO_PILL_RADIUS_KNOB_ID,
        label: "Header pill radius",
        group: StyleKnobGroup::HeaderInfoBar,
        unit: StyleUnit::Px,
        min: 0.0,
        max: 24.0,
        step: 1.0,
        get: get_header_info_pill_radius,
        apply: apply_header_info_pill_radius,
    },
    StyleKnob {
        id: HEADER_INFO_OPACITY_KNOB_ID,
        label: "Header info opacity",
        group: StyleKnobGroup::HeaderInfoBar,
        unit: StyleUnit::Opacity,
        min: 0.0,
        max: 1.0,
        step: 0.01,
        get: get_header_info_opacity,
        apply: apply_header_info_opacity,
    },
    StyleKnob {
        id: HEADER_INFO_KEY_OPACITY_KNOB_ID,
        label: "Header key opacity",
        group: StyleKnobGroup::HeaderInfoBar,
        unit: StyleUnit::Opacity,
        min: 0.0,
        max: 1.0,
        step: 0.01,
        get: get_header_info_key_opacity,
        apply: apply_header_info_key_opacity,
    },
    StyleKnob {
        id: HEADER_INFO_PILL_BG_ALPHA_KNOB_ID,
        label: "Header pill bg alpha",
        group: StyleKnobGroup::HeaderInfoBar,
        unit: StyleUnit::Alpha,
        min: 0.0,
        max: 255.0,
        step: 1.0,
        get: get_header_info_pill_bg_alpha,
        apply: apply_header_info_pill_bg_alpha,
    },
    StyleKnob {
        id: HEADER_INFO_PILL_BORDER_ALPHA_KNOB_ID,
        label: "Header pill border alpha",
        group: StyleKnobGroup::HeaderInfoBar,
        unit: StyleUnit::Alpha,
        min: 0.0,
        max: 255.0,
        step: 1.0,
        get: get_header_info_pill_border_alpha,
        apply: apply_header_info_pill_border_alpha,
    },
    StyleKnob {
        id: HEADER_INFO_CONTEXT_EDGE_OUTSET_X_KNOB_ID,
        label: "Header context edge outset X",
        group: StyleKnobGroup::HeaderInfoBar,
        unit: StyleUnit::Px,
        min: 0.0,
        max: 32.0,
        step: 1.0,
        get: get_header_info_context_edge_outset_x,
        apply: apply_header_info_context_edge_outset_x,
    },
    StyleKnob {
        id: HEADER_INFO_VARIATION_BADGE_WIDTH_KNOB_ID,
        label: "Header variation badge width",
        group: StyleKnobGroup::HeaderInfoBar,
        unit: StyleUnit::Px,
        min: 0.0,
        max: 96.0,
        step: 1.0,
        get: get_header_info_variation_badge_width,
        apply: apply_header_info_variation_badge_width,
    },
];

pub fn knob_by_id(id: StyleKnobId) -> Option<&'static StyleKnob> {
    STYLE_KNOBS.iter().find(|knob| knob.id == id)
}

pub fn knob_id_from_str(value: &str) -> Option<StyleKnobId> {
    let normalized = value
        .strip_prefix("control:dev-style-tool:")
        .unwrap_or(value);
    STYLE_KNOBS
        .iter()
        .find(|knob| knob.id.as_str() == normalized)
        .map(|knob| knob.id)
}
