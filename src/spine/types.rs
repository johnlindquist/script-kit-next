use std::ops::Range;

/// A single grammar segment parsed from the input string.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SpineSegment {
    /// The kind of segment (sigil-derived).
    pub kind: SpineSegmentKind,
    /// Byte range in the original input string.
    pub byte_range: Range<usize>,
    /// The raw text of this segment (including sigil).
    pub raw: String,
    /// Resolution state (resolved against known entities, unknown, or unresolved).
    pub resolution: SpineSegmentResolution,
}

/// What kind of grammar segment this is.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SpineSegmentKind {
    /// Free text (no sigil prefix). Includes the instruction tail.
    FreeText,
    /// `@` context mention, e.g. `@selection`, `@file:readme.md`
    ContextMention {
        /// The context type, e.g. "selection", "file"
        context_type: String,
        /// Sub-query after `:`, e.g. "readme.md" in `@file:readme.md`
        sub_query: Option<String>,
    },
    /// `/` slash command, e.g. `/rewrite`
    SlashCommand { command: String },
    /// `|` profile, e.g. `|creative`
    Profile { profile_id: String },
    /// `.` style (sugar for `|style /rewrite @selection`), e.g. `.professional`
    Style { style_id: String },
    /// `;` capture target, e.g. `;todo`
    Capture { target: String, args: String },
    /// `:` list filter / advanced query, e.g. `:type:script`
    ListFilter { query: String },
    /// `>` project/cwd selector, e.g. `>:dev`
    ProjectCwd {
        /// Sub-query after `:`, e.g. "dev" in `>:dev`
        sub_query: Option<String>,
    },
    /// `-` flow search, e.g. `-gmail` — search/stage an mdflow flow. The
    /// flow-roster twin of the `/` command search.
    Flow { query: String },
    /// `~`, `?`, `!` — mode exit sigils
    ModeExit { sigil: char, rest: String },
}

/// Whether a segment has been resolved against known entities.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SpineSegmentResolution {
    /// Not yet checked against any catalog.
    Unresolved,
    /// Matched a known entity.
    Resolved {
        id: String,
        label: String,
        source: String,
    },
    /// Typed but does not match any known entity.
    Unknown {
        raw: String,
        preflight_instruction: String,
    },
}

/// The full parse result for a Spine input string.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct SpineParse {
    /// Ordered segments parsed from the input.
    pub segments: Vec<SpineSegment>,
    /// The original input string.
    pub input: String,
}

/// Projection of which segment the cursor is currently inside.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SpineCursorProjection {
    /// Index into `SpineParse::segments` for the active segment.
    pub active_segment_index: usize,
    /// Kind of the active segment (convenience copy).
    pub active_segment_kind: SpineSegmentKind,
    /// The query text within the active segment that should drive list filtering.
    /// For `@file:read` this would be `"read"`. For `/rew` this would be `"rew"`.
    pub active_query: String,
    /// Whether the cursor is in a free-text tail after prompt-builder segments.
    pub is_tail: bool,
    /// Whether prompt-builder segments exist (determines whether free-text tail
    /// shows recent prompts vs normal unified search).
    pub has_prompt_segments: bool,
}
