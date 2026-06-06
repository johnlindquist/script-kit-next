use super::catalog::{StyleUnit, StyleValue};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AgentChatStyleDef {
    pub transcript: AgentChatTranscriptStyle,
    pub markdown: AgentChatMarkdownStyle,
    pub user_message: AgentChatMessageStyle,
    pub assistant_message: AgentChatMessageStyle,
    pub collapsible: AgentChatCollapsibleStyle,
    pub error: AgentChatErrorStyle,
    pub system: AgentChatSystemStyle,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AgentChatTranscriptStyle {
    pub row_padding_x: f32,
    pub row_padding_bottom: f32,
    pub dense_row_padding_bottom: f32,
    pub response_start_margin_top: f32,
    pub turn_margin_top: f32,
    pub turn_padding_top: f32,
    pub turn_divider_alpha: f32,
    pub focused_preview_padding_x: f32,
    pub focused_preview_padding_bottom: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AgentChatMarkdownStyle {
    pub body_font_size: f32,
    pub paragraph_gap: f32,
    pub heading_1_font_size: f32,
    pub heading_2_font_size: f32,
    pub heading_3_font_size: f32,
    pub code_block_font_size: f32,
    pub code_block_padding_x: f32,
    pub code_block_padding_y: f32,
    pub code_block_radius: f32,
    pub code_block_bg_alpha: f32,
    pub code_block_border_alpha: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AgentChatMessageStyle {
    pub padding_x: f32,
    pub padding_y: f32,
    pub dense_padding_y: f32,
    pub radius: f32,
    pub bg_alpha: f32,
    pub max_width: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AgentChatCollapsibleStyle {
    pub padding_x: f32,
    pub padding_y: f32,
    pub body_padding_top: f32,
    pub max_body_height: f32,
    pub thought_header_opacity: f32,
    pub tool_header_opacity: f32,
    pub status_opacity: f32,
    pub thought_border_alpha: f32,
    pub tool_border_alpha: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AgentChatErrorStyle {
    pub padding_x: f32,
    pub padding_y: f32,
    pub radius: f32,
    pub bg_alpha: f32,
    pub border_alpha: f32,
    pub label_opacity: f32,
    pub hint_opacity: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AgentChatSystemStyle {
    pub padding_x: f32,
    pub padding_y: f32,
    pub opacity: f32,
    pub border_alpha: f32,
}

pub fn base_agent_chat_style() -> AgentChatStyleDef {
    AgentChatStyleDef {
        transcript: AgentChatTranscriptStyle {
            row_padding_x: 8.0,
            row_padding_bottom: 4.0,
            dense_row_padding_bottom: 1.0,
            response_start_margin_top: 4.0,
            turn_margin_top: 8.0,
            turn_padding_top: 8.0,
            turn_divider_alpha: 0x18 as f32,
            focused_preview_padding_x: 8.0,
            focused_preview_padding_bottom: 4.0,
        },
        markdown: AgentChatMarkdownStyle {
            body_font_size: 13.0,
            paragraph_gap: 0.28,
            heading_1_font_size: 16.0,
            heading_2_font_size: 15.0,
            heading_3_font_size: 14.0,
            code_block_font_size: 12.0,
            code_block_padding_x: 7.0,
            code_block_padding_y: 4.0,
            code_block_radius: 5.0,
            code_block_bg_alpha: 0xA0 as f32,
            code_block_border_alpha: 0x40 as f32,
        },
        user_message: AgentChatMessageStyle {
            padding_x: 12.0,
            padding_y: 8.0,
            dense_padding_y: 3.0,
            radius: 8.0,
            bg_alpha: 0x06 as f32,
            max_width: 520.0,
        },
        assistant_message: AgentChatMessageStyle {
            padding_x: 12.0,
            padding_y: 4.0,
            dense_padding_y: 2.0,
            radius: 0.0,
            bg_alpha: 0.0,
            max_width: 620.0,
        },
        collapsible: AgentChatCollapsibleStyle {
            padding_x: 12.0,
            padding_y: 2.0,
            body_padding_top: 4.0,
            max_body_height: 200.0,
            thought_header_opacity: 0.50,
            tool_header_opacity: 0.55,
            status_opacity: 0.35,
            thought_border_alpha: 0x18 as f32,
            tool_border_alpha: 0x30 as f32,
        },
        error: AgentChatErrorStyle {
            padding_x: 12.0,
            padding_y: 8.0,
            radius: 8.0,
            bg_alpha: 0x10 as f32,
            border_alpha: 0x80 as f32,
            label_opacity: 0.75,
            hint_opacity: 0.40,
        },
        system: AgentChatSystemStyle {
            padding_x: 12.0,
            padding_y: 4.0,
            opacity: 0.60,
            border_alpha: 0x30 as f32,
        },
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct AgentChatKnobId(&'static str);

impl AgentChatKnobId {
    pub const fn new(value: &'static str) -> Self {
        Self(value)
    }

    pub const fn as_str(self) -> &'static str {
        self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AgentChatKnobGroup {
    Transcript,
    Markdown,
    UserMessage,
    AssistantMessage,
    CollapsibleBlocks,
    ErrorAndSystem,
}

impl AgentChatKnobGroup {
    pub const fn label(self) -> &'static str {
        match self {
            Self::Transcript => "Transcript layout",
            Self::Markdown => "Markdown rendering",
            Self::UserMessage => "User messages",
            Self::AssistantMessage => "Assistant messages",
            Self::CollapsibleBlocks => "Tool and thought blocks",
            Self::ErrorAndSystem => "Error and system messages",
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct AgentChatKnob {
    pub id: AgentChatKnobId,
    pub label: &'static str,
    pub group: AgentChatKnobGroup,
    pub unit: StyleUnit,
    pub min: f32,
    pub max: f32,
    pub step: f32,
    pub get: fn(&AgentChatStyleDef) -> StyleValue,
    pub apply: fn(&mut AgentChatStyleDef, StyleValue),
}

impl AgentChatKnob {
    pub fn clamp_value(self, value: StyleValue) -> StyleValue {
        match value {
            StyleValue::Number(number) => StyleValue::Number(number.clamp(self.min, self.max)),
        }
    }
}

macro_rules! f32_knob {
    ($id_const:ident, $get_fn:ident, $apply_fn:ident, $id:literal, $section:ident.$field:ident) => {
        pub const $id_const: AgentChatKnobId = AgentChatKnobId::new($id);
        fn $get_fn(def: &AgentChatStyleDef) -> StyleValue {
            StyleValue::Number(def.$section.$field)
        }
        fn $apply_fn(def: &mut AgentChatStyleDef, value: StyleValue) {
            let StyleValue::Number(value) = value;
            def.$section.$field = value;
        }
    };
}

f32_knob!(
    AGENT_CHAT_TRANSCRIPT_ROW_PADDING_X,
    get_transcript_row_padding_x,
    apply_transcript_row_padding_x,
    "agentChat.transcript.rowPaddingX",
    transcript.row_padding_x
);
f32_knob!(
    AGENT_CHAT_TRANSCRIPT_ROW_PADDING_BOTTOM,
    get_transcript_row_padding_bottom,
    apply_transcript_row_padding_bottom,
    "agentChat.transcript.rowPaddingBottom",
    transcript.row_padding_bottom
);
f32_knob!(
    AGENT_CHAT_TRANSCRIPT_DENSE_ROW_PADDING_BOTTOM,
    get_transcript_dense_row_padding_bottom,
    apply_transcript_dense_row_padding_bottom,
    "agentChat.transcript.denseRowPaddingBottom",
    transcript.dense_row_padding_bottom
);
f32_knob!(
    AGENT_CHAT_TRANSCRIPT_RESPONSE_START_MARGIN_TOP,
    get_transcript_response_start_margin_top,
    apply_transcript_response_start_margin_top,
    "agentChat.transcript.responseStartMarginTop",
    transcript.response_start_margin_top
);
f32_knob!(
    AGENT_CHAT_TRANSCRIPT_TURN_MARGIN_TOP,
    get_transcript_turn_margin_top,
    apply_transcript_turn_margin_top,
    "agentChat.transcript.turnMarginTop",
    transcript.turn_margin_top
);
f32_knob!(
    AGENT_CHAT_TRANSCRIPT_TURN_PADDING_TOP,
    get_transcript_turn_padding_top,
    apply_transcript_turn_padding_top,
    "agentChat.transcript.turnPaddingTop",
    transcript.turn_padding_top
);
f32_knob!(
    AGENT_CHAT_TRANSCRIPT_TURN_DIVIDER_ALPHA,
    get_transcript_turn_divider_alpha,
    apply_transcript_turn_divider_alpha,
    "agentChat.transcript.turnDividerAlpha",
    transcript.turn_divider_alpha
);
f32_knob!(
    AGENT_CHAT_TRANSCRIPT_FOCUSED_PREVIEW_PADDING_X,
    get_transcript_focused_preview_padding_x,
    apply_transcript_focused_preview_padding_x,
    "agentChat.transcript.focusedPreviewPaddingX",
    transcript.focused_preview_padding_x
);
f32_knob!(
    AGENT_CHAT_TRANSCRIPT_FOCUSED_PREVIEW_PADDING_BOTTOM,
    get_transcript_focused_preview_padding_bottom,
    apply_transcript_focused_preview_padding_bottom,
    "agentChat.transcript.focusedPreviewPaddingBottom",
    transcript.focused_preview_padding_bottom
);
f32_knob!(
    AGENT_CHAT_MARKDOWN_BODY_FONT_SIZE,
    get_markdown_body_font_size,
    apply_markdown_body_font_size,
    "agentChat.markdown.bodyFontSize",
    markdown.body_font_size
);
f32_knob!(
    AGENT_CHAT_MARKDOWN_PARAGRAPH_GAP,
    get_markdown_paragraph_gap,
    apply_markdown_paragraph_gap,
    "agentChat.markdown.paragraphGap",
    markdown.paragraph_gap
);
f32_knob!(
    AGENT_CHAT_MARKDOWN_HEADING_1_FONT_SIZE,
    get_markdown_heading_1_font_size,
    apply_markdown_heading_1_font_size,
    "agentChat.markdown.heading1FontSize",
    markdown.heading_1_font_size
);
f32_knob!(
    AGENT_CHAT_MARKDOWN_HEADING_2_FONT_SIZE,
    get_markdown_heading_2_font_size,
    apply_markdown_heading_2_font_size,
    "agentChat.markdown.heading2FontSize",
    markdown.heading_2_font_size
);
f32_knob!(
    AGENT_CHAT_MARKDOWN_HEADING_3_FONT_SIZE,
    get_markdown_heading_3_font_size,
    apply_markdown_heading_3_font_size,
    "agentChat.markdown.heading3FontSize",
    markdown.heading_3_font_size
);
f32_knob!(
    AGENT_CHAT_MARKDOWN_CODE_BLOCK_FONT_SIZE,
    get_markdown_code_block_font_size,
    apply_markdown_code_block_font_size,
    "agentChat.markdown.codeBlockFontSize",
    markdown.code_block_font_size
);
f32_knob!(
    AGENT_CHAT_MARKDOWN_CODE_BLOCK_PADDING_X,
    get_markdown_code_block_padding_x,
    apply_markdown_code_block_padding_x,
    "agentChat.markdown.codeBlockPaddingX",
    markdown.code_block_padding_x
);
f32_knob!(
    AGENT_CHAT_MARKDOWN_CODE_BLOCK_PADDING_Y,
    get_markdown_code_block_padding_y,
    apply_markdown_code_block_padding_y,
    "agentChat.markdown.codeBlockPaddingY",
    markdown.code_block_padding_y
);
f32_knob!(
    AGENT_CHAT_MARKDOWN_CODE_BLOCK_RADIUS,
    get_markdown_code_block_radius,
    apply_markdown_code_block_radius,
    "agentChat.markdown.codeBlockRadius",
    markdown.code_block_radius
);
f32_knob!(
    AGENT_CHAT_MARKDOWN_CODE_BLOCK_BG_ALPHA,
    get_markdown_code_block_bg_alpha,
    apply_markdown_code_block_bg_alpha,
    "agentChat.markdown.codeBlockBgAlpha",
    markdown.code_block_bg_alpha
);
f32_knob!(
    AGENT_CHAT_MARKDOWN_CODE_BLOCK_BORDER_ALPHA,
    get_markdown_code_block_border_alpha,
    apply_markdown_code_block_border_alpha,
    "agentChat.markdown.codeBlockBorderAlpha",
    markdown.code_block_border_alpha
);
f32_knob!(
    AGENT_CHAT_USER_PADDING_X,
    get_user_padding_x,
    apply_user_padding_x,
    "agentChat.user.paddingX",
    user_message.padding_x
);
f32_knob!(
    AGENT_CHAT_USER_PADDING_Y,
    get_user_padding_y,
    apply_user_padding_y,
    "agentChat.user.paddingY",
    user_message.padding_y
);
f32_knob!(
    AGENT_CHAT_USER_DENSE_PADDING_Y,
    get_user_dense_padding_y,
    apply_user_dense_padding_y,
    "agentChat.user.densePaddingY",
    user_message.dense_padding_y
);
f32_knob!(
    AGENT_CHAT_USER_RADIUS,
    get_user_radius,
    apply_user_radius,
    "agentChat.user.radius",
    user_message.radius
);
f32_knob!(
    AGENT_CHAT_USER_BG_ALPHA,
    get_user_bg_alpha,
    apply_user_bg_alpha,
    "agentChat.user.bgAlpha",
    user_message.bg_alpha
);
f32_knob!(
    AGENT_CHAT_USER_MAX_WIDTH,
    get_user_max_width,
    apply_user_max_width,
    "agentChat.user.maxWidth",
    user_message.max_width
);
f32_knob!(
    AGENT_CHAT_ASSISTANT_PADDING_X,
    get_assistant_padding_x,
    apply_assistant_padding_x,
    "agentChat.assistant.paddingX",
    assistant_message.padding_x
);
f32_knob!(
    AGENT_CHAT_ASSISTANT_PADDING_Y,
    get_assistant_padding_y,
    apply_assistant_padding_y,
    "agentChat.assistant.paddingY",
    assistant_message.padding_y
);
f32_knob!(
    AGENT_CHAT_ASSISTANT_DENSE_PADDING_Y,
    get_assistant_dense_padding_y,
    apply_assistant_dense_padding_y,
    "agentChat.assistant.densePaddingY",
    assistant_message.dense_padding_y
);
f32_knob!(
    AGENT_CHAT_ASSISTANT_RADIUS,
    get_assistant_radius,
    apply_assistant_radius,
    "agentChat.assistant.radius",
    assistant_message.radius
);
f32_knob!(
    AGENT_CHAT_ASSISTANT_BG_ALPHA,
    get_assistant_bg_alpha,
    apply_assistant_bg_alpha,
    "agentChat.assistant.bgAlpha",
    assistant_message.bg_alpha
);
f32_knob!(
    AGENT_CHAT_ASSISTANT_MAX_WIDTH,
    get_assistant_max_width,
    apply_assistant_max_width,
    "agentChat.assistant.maxWidth",
    assistant_message.max_width
);
f32_knob!(
    AGENT_CHAT_COLLAPSIBLE_PADDING_X,
    get_collapsible_padding_x,
    apply_collapsible_padding_x,
    "agentChat.collapsible.paddingX",
    collapsible.padding_x
);
f32_knob!(
    AGENT_CHAT_COLLAPSIBLE_PADDING_Y,
    get_collapsible_padding_y,
    apply_collapsible_padding_y,
    "agentChat.collapsible.paddingY",
    collapsible.padding_y
);
f32_knob!(
    AGENT_CHAT_COLLAPSIBLE_BODY_PADDING_TOP,
    get_collapsible_body_padding_top,
    apply_collapsible_body_padding_top,
    "agentChat.collapsible.bodyPaddingTop",
    collapsible.body_padding_top
);
f32_knob!(
    AGENT_CHAT_COLLAPSIBLE_MAX_BODY_HEIGHT,
    get_collapsible_max_body_height,
    apply_collapsible_max_body_height,
    "agentChat.collapsible.maxBodyHeight",
    collapsible.max_body_height
);
f32_knob!(
    AGENT_CHAT_COLLAPSIBLE_THOUGHT_HEADER_OPACITY,
    get_collapsible_thought_header_opacity,
    apply_collapsible_thought_header_opacity,
    "agentChat.collapsible.thoughtHeaderOpacity",
    collapsible.thought_header_opacity
);
f32_knob!(
    AGENT_CHAT_COLLAPSIBLE_TOOL_HEADER_OPACITY,
    get_collapsible_tool_header_opacity,
    apply_collapsible_tool_header_opacity,
    "agentChat.collapsible.toolHeaderOpacity",
    collapsible.tool_header_opacity
);
f32_knob!(
    AGENT_CHAT_COLLAPSIBLE_STATUS_OPACITY,
    get_collapsible_status_opacity,
    apply_collapsible_status_opacity,
    "agentChat.collapsible.statusOpacity",
    collapsible.status_opacity
);
f32_knob!(
    AGENT_CHAT_COLLAPSIBLE_THOUGHT_BORDER_ALPHA,
    get_collapsible_thought_border_alpha,
    apply_collapsible_thought_border_alpha,
    "agentChat.collapsible.thoughtBorderAlpha",
    collapsible.thought_border_alpha
);
f32_knob!(
    AGENT_CHAT_COLLAPSIBLE_TOOL_BORDER_ALPHA,
    get_collapsible_tool_border_alpha,
    apply_collapsible_tool_border_alpha,
    "agentChat.collapsible.toolBorderAlpha",
    collapsible.tool_border_alpha
);
f32_knob!(
    AGENT_CHAT_ERROR_PADDING_X,
    get_error_padding_x,
    apply_error_padding_x,
    "agentChat.error.paddingX",
    error.padding_x
);
f32_knob!(
    AGENT_CHAT_ERROR_PADDING_Y,
    get_error_padding_y,
    apply_error_padding_y,
    "agentChat.error.paddingY",
    error.padding_y
);
f32_knob!(
    AGENT_CHAT_ERROR_RADIUS,
    get_error_radius,
    apply_error_radius,
    "agentChat.error.radius",
    error.radius
);
f32_knob!(
    AGENT_CHAT_ERROR_BG_ALPHA,
    get_error_bg_alpha,
    apply_error_bg_alpha,
    "agentChat.error.bgAlpha",
    error.bg_alpha
);
f32_knob!(
    AGENT_CHAT_ERROR_BORDER_ALPHA,
    get_error_border_alpha,
    apply_error_border_alpha,
    "agentChat.error.borderAlpha",
    error.border_alpha
);
f32_knob!(
    AGENT_CHAT_ERROR_LABEL_OPACITY,
    get_error_label_opacity,
    apply_error_label_opacity,
    "agentChat.error.labelOpacity",
    error.label_opacity
);
f32_knob!(
    AGENT_CHAT_ERROR_HINT_OPACITY,
    get_error_hint_opacity,
    apply_error_hint_opacity,
    "agentChat.error.hintOpacity",
    error.hint_opacity
);
f32_knob!(
    AGENT_CHAT_SYSTEM_PADDING_X,
    get_system_padding_x,
    apply_system_padding_x,
    "agentChat.system.paddingX",
    system.padding_x
);
f32_knob!(
    AGENT_CHAT_SYSTEM_PADDING_Y,
    get_system_padding_y,
    apply_system_padding_y,
    "agentChat.system.paddingY",
    system.padding_y
);
f32_knob!(
    AGENT_CHAT_SYSTEM_OPACITY,
    get_system_opacity,
    apply_system_opacity,
    "agentChat.system.opacity",
    system.opacity
);
f32_knob!(
    AGENT_CHAT_SYSTEM_BORDER_ALPHA,
    get_system_border_alpha,
    apply_system_border_alpha,
    "agentChat.system.borderAlpha",
    system.border_alpha
);

pub const AGENT_CHAT_KNOBS: &[AgentChatKnob] = &[
    knob(
        AGENT_CHAT_TRANSCRIPT_ROW_PADDING_X,
        "Transcript row padding X",
        AgentChatKnobGroup::Transcript,
        StyleUnit::Px,
        0.0,
        40.0,
        1.0,
        get_transcript_row_padding_x,
        apply_transcript_row_padding_x,
    ),
    knob(
        AGENT_CHAT_TRANSCRIPT_ROW_PADDING_BOTTOM,
        "Transcript row bottom padding",
        AgentChatKnobGroup::Transcript,
        StyleUnit::Px,
        0.0,
        32.0,
        1.0,
        get_transcript_row_padding_bottom,
        apply_transcript_row_padding_bottom,
    ),
    knob(
        AGENT_CHAT_TRANSCRIPT_DENSE_ROW_PADDING_BOTTOM,
        "Dense row bottom padding",
        AgentChatKnobGroup::Transcript,
        StyleUnit::Px,
        0.0,
        16.0,
        1.0,
        get_transcript_dense_row_padding_bottom,
        apply_transcript_dense_row_padding_bottom,
    ),
    knob(
        AGENT_CHAT_TRANSCRIPT_RESPONSE_START_MARGIN_TOP,
        "Response start margin top",
        AgentChatKnobGroup::Transcript,
        StyleUnit::Px,
        0.0,
        32.0,
        1.0,
        get_transcript_response_start_margin_top,
        apply_transcript_response_start_margin_top,
    ),
    knob(
        AGENT_CHAT_TRANSCRIPT_TURN_MARGIN_TOP,
        "New turn margin top",
        AgentChatKnobGroup::Transcript,
        StyleUnit::Px,
        0.0,
        40.0,
        1.0,
        get_transcript_turn_margin_top,
        apply_transcript_turn_margin_top,
    ),
    knob(
        AGENT_CHAT_TRANSCRIPT_TURN_PADDING_TOP,
        "New turn padding top",
        AgentChatKnobGroup::Transcript,
        StyleUnit::Px,
        0.0,
        40.0,
        1.0,
        get_transcript_turn_padding_top,
        apply_transcript_turn_padding_top,
    ),
    knob(
        AGENT_CHAT_TRANSCRIPT_TURN_DIVIDER_ALPHA,
        "Turn divider alpha",
        AgentChatKnobGroup::Transcript,
        StyleUnit::Alpha,
        0.0,
        255.0,
        1.0,
        get_transcript_turn_divider_alpha,
        apply_transcript_turn_divider_alpha,
    ),
    knob(
        AGENT_CHAT_TRANSCRIPT_FOCUSED_PREVIEW_PADDING_X,
        "Focused preview padding X",
        AgentChatKnobGroup::Transcript,
        StyleUnit::Px,
        0.0,
        32.0,
        1.0,
        get_transcript_focused_preview_padding_x,
        apply_transcript_focused_preview_padding_x,
    ),
    knob(
        AGENT_CHAT_TRANSCRIPT_FOCUSED_PREVIEW_PADDING_BOTTOM,
        "Focused preview padding bottom",
        AgentChatKnobGroup::Transcript,
        StyleUnit::Px,
        0.0,
        32.0,
        1.0,
        get_transcript_focused_preview_padding_bottom,
        apply_transcript_focused_preview_padding_bottom,
    ),
    knob(
        AGENT_CHAT_MARKDOWN_BODY_FONT_SIZE,
        "Markdown body font size",
        AgentChatKnobGroup::Markdown,
        StyleUnit::Px,
        9.0,
        24.0,
        0.5,
        get_markdown_body_font_size,
        apply_markdown_body_font_size,
    ),
    knob(
        AGENT_CHAT_MARKDOWN_PARAGRAPH_GAP,
        "Paragraph gap",
        AgentChatKnobGroup::Markdown,
        StyleUnit::Px,
        0.0,
        1.2,
        0.01,
        get_markdown_paragraph_gap,
        apply_markdown_paragraph_gap,
    ),
    knob(
        AGENT_CHAT_MARKDOWN_HEADING_1_FONT_SIZE,
        "Heading 1 font size",
        AgentChatKnobGroup::Markdown,
        StyleUnit::Px,
        10.0,
        32.0,
        0.5,
        get_markdown_heading_1_font_size,
        apply_markdown_heading_1_font_size,
    ),
    knob(
        AGENT_CHAT_MARKDOWN_HEADING_2_FONT_SIZE,
        "Heading 2 font size",
        AgentChatKnobGroup::Markdown,
        StyleUnit::Px,
        10.0,
        30.0,
        0.5,
        get_markdown_heading_2_font_size,
        apply_markdown_heading_2_font_size,
    ),
    knob(
        AGENT_CHAT_MARKDOWN_HEADING_3_FONT_SIZE,
        "Heading 3 font size",
        AgentChatKnobGroup::Markdown,
        StyleUnit::Px,
        10.0,
        28.0,
        0.5,
        get_markdown_heading_3_font_size,
        apply_markdown_heading_3_font_size,
    ),
    knob(
        AGENT_CHAT_MARKDOWN_CODE_BLOCK_FONT_SIZE,
        "Code block font size",
        AgentChatKnobGroup::Markdown,
        StyleUnit::Px,
        9.0,
        22.0,
        0.5,
        get_markdown_code_block_font_size,
        apply_markdown_code_block_font_size,
    ),
    knob(
        AGENT_CHAT_MARKDOWN_CODE_BLOCK_PADDING_X,
        "Code block padding X",
        AgentChatKnobGroup::Markdown,
        StyleUnit::Px,
        0.0,
        24.0,
        1.0,
        get_markdown_code_block_padding_x,
        apply_markdown_code_block_padding_x,
    ),
    knob(
        AGENT_CHAT_MARKDOWN_CODE_BLOCK_PADDING_Y,
        "Code block padding Y",
        AgentChatKnobGroup::Markdown,
        StyleUnit::Px,
        0.0,
        20.0,
        1.0,
        get_markdown_code_block_padding_y,
        apply_markdown_code_block_padding_y,
    ),
    knob(
        AGENT_CHAT_MARKDOWN_CODE_BLOCK_RADIUS,
        "Code block radius",
        AgentChatKnobGroup::Markdown,
        StyleUnit::Px,
        0.0,
        18.0,
        1.0,
        get_markdown_code_block_radius,
        apply_markdown_code_block_radius,
    ),
    knob(
        AGENT_CHAT_MARKDOWN_CODE_BLOCK_BG_ALPHA,
        "Code block background alpha",
        AgentChatKnobGroup::Markdown,
        StyleUnit::Alpha,
        0.0,
        255.0,
        1.0,
        get_markdown_code_block_bg_alpha,
        apply_markdown_code_block_bg_alpha,
    ),
    knob(
        AGENT_CHAT_MARKDOWN_CODE_BLOCK_BORDER_ALPHA,
        "Code block border alpha",
        AgentChatKnobGroup::Markdown,
        StyleUnit::Alpha,
        0.0,
        255.0,
        1.0,
        get_markdown_code_block_border_alpha,
        apply_markdown_code_block_border_alpha,
    ),
    knob(
        AGENT_CHAT_USER_PADDING_X,
        "User padding X",
        AgentChatKnobGroup::UserMessage,
        StyleUnit::Px,
        0.0,
        40.0,
        1.0,
        get_user_padding_x,
        apply_user_padding_x,
    ),
    knob(
        AGENT_CHAT_USER_PADDING_Y,
        "User padding Y",
        AgentChatKnobGroup::UserMessage,
        StyleUnit::Px,
        0.0,
        32.0,
        1.0,
        get_user_padding_y,
        apply_user_padding_y,
    ),
    knob(
        AGENT_CHAT_USER_DENSE_PADDING_Y,
        "User dense padding Y",
        AgentChatKnobGroup::UserMessage,
        StyleUnit::Px,
        0.0,
        20.0,
        1.0,
        get_user_dense_padding_y,
        apply_user_dense_padding_y,
    ),
    knob(
        AGENT_CHAT_USER_RADIUS,
        "User bubble radius",
        AgentChatKnobGroup::UserMessage,
        StyleUnit::Px,
        0.0,
        24.0,
        1.0,
        get_user_radius,
        apply_user_radius,
    ),
    knob(
        AGENT_CHAT_USER_BG_ALPHA,
        "User bubble alpha",
        AgentChatKnobGroup::UserMessage,
        StyleUnit::Alpha,
        0.0,
        255.0,
        1.0,
        get_user_bg_alpha,
        apply_user_bg_alpha,
    ),
    knob(
        AGENT_CHAT_USER_MAX_WIDTH,
        "User role-split max width",
        AgentChatKnobGroup::UserMessage,
        StyleUnit::Px,
        240.0,
        900.0,
        1.0,
        get_user_max_width,
        apply_user_max_width,
    ),
    knob(
        AGENT_CHAT_ASSISTANT_PADDING_X,
        "Assistant padding X",
        AgentChatKnobGroup::AssistantMessage,
        StyleUnit::Px,
        0.0,
        40.0,
        1.0,
        get_assistant_padding_x,
        apply_assistant_padding_x,
    ),
    knob(
        AGENT_CHAT_ASSISTANT_PADDING_Y,
        "Assistant padding Y",
        AgentChatKnobGroup::AssistantMessage,
        StyleUnit::Px,
        0.0,
        32.0,
        1.0,
        get_assistant_padding_y,
        apply_assistant_padding_y,
    ),
    knob(
        AGENT_CHAT_ASSISTANT_DENSE_PADDING_Y,
        "Assistant dense padding Y",
        AgentChatKnobGroup::AssistantMessage,
        StyleUnit::Px,
        0.0,
        20.0,
        1.0,
        get_assistant_dense_padding_y,
        apply_assistant_dense_padding_y,
    ),
    knob(
        AGENT_CHAT_ASSISTANT_RADIUS,
        "Assistant bubble radius",
        AgentChatKnobGroup::AssistantMessage,
        StyleUnit::Px,
        0.0,
        24.0,
        1.0,
        get_assistant_radius,
        apply_assistant_radius,
    ),
    knob(
        AGENT_CHAT_ASSISTANT_BG_ALPHA,
        "Assistant bubble alpha",
        AgentChatKnobGroup::AssistantMessage,
        StyleUnit::Alpha,
        0.0,
        255.0,
        1.0,
        get_assistant_bg_alpha,
        apply_assistant_bg_alpha,
    ),
    knob(
        AGENT_CHAT_ASSISTANT_MAX_WIDTH,
        "Assistant role-split max width",
        AgentChatKnobGroup::AssistantMessage,
        StyleUnit::Px,
        240.0,
        980.0,
        1.0,
        get_assistant_max_width,
        apply_assistant_max_width,
    ),
    knob(
        AGENT_CHAT_COLLAPSIBLE_PADDING_X,
        "Block padding X",
        AgentChatKnobGroup::CollapsibleBlocks,
        StyleUnit::Px,
        0.0,
        40.0,
        1.0,
        get_collapsible_padding_x,
        apply_collapsible_padding_x,
    ),
    knob(
        AGENT_CHAT_COLLAPSIBLE_PADDING_Y,
        "Block padding Y",
        AgentChatKnobGroup::CollapsibleBlocks,
        StyleUnit::Px,
        0.0,
        24.0,
        1.0,
        get_collapsible_padding_y,
        apply_collapsible_padding_y,
    ),
    knob(
        AGENT_CHAT_COLLAPSIBLE_BODY_PADDING_TOP,
        "Block body padding top",
        AgentChatKnobGroup::CollapsibleBlocks,
        StyleUnit::Px,
        0.0,
        32.0,
        1.0,
        get_collapsible_body_padding_top,
        apply_collapsible_body_padding_top,
    ),
    knob(
        AGENT_CHAT_COLLAPSIBLE_MAX_BODY_HEIGHT,
        "Block max body height",
        AgentChatKnobGroup::CollapsibleBlocks,
        StyleUnit::Px,
        60.0,
        600.0,
        1.0,
        get_collapsible_max_body_height,
        apply_collapsible_max_body_height,
    ),
    knob(
        AGENT_CHAT_COLLAPSIBLE_THOUGHT_HEADER_OPACITY,
        "Thought header opacity",
        AgentChatKnobGroup::CollapsibleBlocks,
        StyleUnit::Opacity,
        0.0,
        1.0,
        0.01,
        get_collapsible_thought_header_opacity,
        apply_collapsible_thought_header_opacity,
    ),
    knob(
        AGENT_CHAT_COLLAPSIBLE_TOOL_HEADER_OPACITY,
        "Tool header opacity",
        AgentChatKnobGroup::CollapsibleBlocks,
        StyleUnit::Opacity,
        0.0,
        1.0,
        0.01,
        get_collapsible_tool_header_opacity,
        apply_collapsible_tool_header_opacity,
    ),
    knob(
        AGENT_CHAT_COLLAPSIBLE_STATUS_OPACITY,
        "Block status opacity",
        AgentChatKnobGroup::CollapsibleBlocks,
        StyleUnit::Opacity,
        0.0,
        1.0,
        0.01,
        get_collapsible_status_opacity,
        apply_collapsible_status_opacity,
    ),
    knob(
        AGENT_CHAT_COLLAPSIBLE_THOUGHT_BORDER_ALPHA,
        "Thought border alpha",
        AgentChatKnobGroup::CollapsibleBlocks,
        StyleUnit::Alpha,
        0.0,
        255.0,
        1.0,
        get_collapsible_thought_border_alpha,
        apply_collapsible_thought_border_alpha,
    ),
    knob(
        AGENT_CHAT_COLLAPSIBLE_TOOL_BORDER_ALPHA,
        "Tool border alpha",
        AgentChatKnobGroup::CollapsibleBlocks,
        StyleUnit::Alpha,
        0.0,
        255.0,
        1.0,
        get_collapsible_tool_border_alpha,
        apply_collapsible_tool_border_alpha,
    ),
    knob(
        AGENT_CHAT_ERROR_PADDING_X,
        "Error padding X",
        AgentChatKnobGroup::ErrorAndSystem,
        StyleUnit::Px,
        0.0,
        40.0,
        1.0,
        get_error_padding_x,
        apply_error_padding_x,
    ),
    knob(
        AGENT_CHAT_ERROR_PADDING_Y,
        "Error padding Y",
        AgentChatKnobGroup::ErrorAndSystem,
        StyleUnit::Px,
        0.0,
        32.0,
        1.0,
        get_error_padding_y,
        apply_error_padding_y,
    ),
    knob(
        AGENT_CHAT_ERROR_RADIUS,
        "Error radius",
        AgentChatKnobGroup::ErrorAndSystem,
        StyleUnit::Px,
        0.0,
        24.0,
        1.0,
        get_error_radius,
        apply_error_radius,
    ),
    knob(
        AGENT_CHAT_ERROR_BG_ALPHA,
        "Error background alpha",
        AgentChatKnobGroup::ErrorAndSystem,
        StyleUnit::Alpha,
        0.0,
        255.0,
        1.0,
        get_error_bg_alpha,
        apply_error_bg_alpha,
    ),
    knob(
        AGENT_CHAT_ERROR_BORDER_ALPHA,
        "Error border alpha",
        AgentChatKnobGroup::ErrorAndSystem,
        StyleUnit::Alpha,
        0.0,
        255.0,
        1.0,
        get_error_border_alpha,
        apply_error_border_alpha,
    ),
    knob(
        AGENT_CHAT_ERROR_LABEL_OPACITY,
        "Error label opacity",
        AgentChatKnobGroup::ErrorAndSystem,
        StyleUnit::Opacity,
        0.0,
        1.0,
        0.01,
        get_error_label_opacity,
        apply_error_label_opacity,
    ),
    knob(
        AGENT_CHAT_ERROR_HINT_OPACITY,
        "Error hint opacity",
        AgentChatKnobGroup::ErrorAndSystem,
        StyleUnit::Opacity,
        0.0,
        1.0,
        0.01,
        get_error_hint_opacity,
        apply_error_hint_opacity,
    ),
    knob(
        AGENT_CHAT_SYSTEM_PADDING_X,
        "System padding X",
        AgentChatKnobGroup::ErrorAndSystem,
        StyleUnit::Px,
        0.0,
        40.0,
        1.0,
        get_system_padding_x,
        apply_system_padding_x,
    ),
    knob(
        AGENT_CHAT_SYSTEM_PADDING_Y,
        "System padding Y",
        AgentChatKnobGroup::ErrorAndSystem,
        StyleUnit::Px,
        0.0,
        32.0,
        1.0,
        get_system_padding_y,
        apply_system_padding_y,
    ),
    knob(
        AGENT_CHAT_SYSTEM_OPACITY,
        "System opacity",
        AgentChatKnobGroup::ErrorAndSystem,
        StyleUnit::Opacity,
        0.0,
        1.0,
        0.01,
        get_system_opacity,
        apply_system_opacity,
    ),
    knob(
        AGENT_CHAT_SYSTEM_BORDER_ALPHA,
        "System border alpha",
        AgentChatKnobGroup::ErrorAndSystem,
        StyleUnit::Alpha,
        0.0,
        255.0,
        1.0,
        get_system_border_alpha,
        apply_system_border_alpha,
    ),
];

const fn knob(
    id: AgentChatKnobId,
    label: &'static str,
    group: AgentChatKnobGroup,
    unit: StyleUnit,
    min: f32,
    max: f32,
    step: f32,
    get: fn(&AgentChatStyleDef) -> StyleValue,
    apply: fn(&mut AgentChatStyleDef, StyleValue),
) -> AgentChatKnob {
    AgentChatKnob {
        id,
        label,
        group,
        unit,
        min,
        max,
        step,
        get,
        apply,
    }
}

pub fn agent_chat_knob_by_id(id: AgentChatKnobId) -> Option<&'static AgentChatKnob> {
    AGENT_CHAT_KNOBS.iter().find(|knob| knob.id == id)
}

pub fn agent_chat_knob_id_from_str(value: &str) -> Option<AgentChatKnobId> {
    AGENT_CHAT_KNOBS
        .iter()
        .find(|knob| knob.id.as_str() == value)
        .map(|knob| knob.id)
}
