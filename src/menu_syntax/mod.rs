pub mod action_effects;
pub mod actions;
pub mod ai;
pub mod artifacts;
pub mod capture;
pub mod capture_gate;
pub mod capture_history_picker;
pub mod capture_schema;
pub mod command;
pub mod date;
pub mod doctor;
pub mod execute;
pub mod filter;
pub mod fragments;
pub mod grammar_payload;
pub mod handler_index;
pub mod history;
pub mod ics;
pub mod main_hint;
pub mod metadata;
pub mod mode;
pub mod nl_anchor;
pub mod nl_duration;
pub mod nl_phrase;
pub mod nl_recurrence;
pub mod nl_time;
pub mod parse;
pub mod payload;
pub mod query;
pub mod quote;
pub mod retention;
pub mod schema_overrides;
pub mod skill;
pub mod templates;
pub mod trigger_picker;
pub mod trigger_picker_keys;

pub use action_effects::{apply_safe_effect, ActionEffect};
pub use actions::{
    current_actions as current_menu_syntax_actions, MenuSyntaxAction, MenuSyntaxActionKind,
    MenuSyntaxActionState,
};
pub use ai::{
    capture_request as capture_ai_request, missing_required_labels,
    parse_response as parse_ai_response, AiParseError, DatePhraseHint, MenuSyntaxAiRequest,
    MenuSyntaxAiResponse, MenuSyntaxAiState,
};
#[allow(unused_imports)]
pub use artifacts::{
    read_all_artifacts, read_jsonl_artifact, read_payload_dir, CaptureArtifact,
    CaptureArtifactKind, ReadArtifactReport,
};
pub use capture::{parse_capture, CaptureParse};
pub use capture_gate::{
    decide_capture_gate, decide_capture_gate_for_script, resolve_capture_schema_for_script,
    resolve_capture_schema_for_target, CaptureGateDecision,
};
#[allow(unused_imports)]
pub use capture_history_picker::{
    build_history_picker_snapshot, build_history_picker_snapshot_with_overrides,
    detect_history_picker_context, snapshot_from_filter_text,
    snapshot_from_filter_text_with_overrides, HistoryPickerKind, HistoryPickerRow,
    HistoryPickerSnapshot,
};
pub use capture_schema::{
    builtin_schema, builtin_target_slugs, validate as validate_capture_payload, CaptureFieldSchema,
    FieldRequirement, ValidationResult,
};
pub use command::{
    command_head_matches, command_slug, script_command_head, scriptlet_command_head,
};
pub use date::{
    normalize_smart_quotes, parse_date_phrase_result, resolve_capture_dates,
    resolve_capture_dates_with_accepts, resolve_date_phrase, DateGranularity, DateParseResult,
    MenuSyntaxClock, RecurrenceFrequency, RecurrenceWeekday, ResolvedCaptureInvocation,
    ResolvedDate, ResolvedDuration, ResolvedRecurrence, UnresolvedDate,
};
#[allow(unused_imports)]
pub use doctor::{
    validate as doctor_validate, validate_at_path as doctor_validate_at_path, DoctorIssue,
    DoctorReport, DoctorSeverity,
};
pub use execute::{
    build_capture_payload, command_env, payload_env, write_payload_tempfile, MenuSyntaxHandlerKind,
    MenuSyntaxHandlerRef, MenuSyntaxPayload, MENU_SYNTAX_PAYLOAD_SCHEMA_ID,
    MENU_SYNTAX_PAYLOAD_VERSION,
};
pub use filter::{
    apply_advanced_query, capture_accepts_for_target_from_scripts, first_command_head_for_script,
    first_concrete_capture_target_for_script, matches_predicate,
    registered_capture_targets_from_scripts, result_kind, script_command_schema_for,
    script_handles_capture, script_menu_syntax_specs, scripts_handling_capture,
};
#[allow(unused_imports)]
pub use fragments::{MenuSyntaxFragment, MenuSyntaxFragmentRole, MenuSyntaxFragmentStatus};
#[allow(unused_imports)]
pub use grammar_payload::{
    DateEntry as GrammarDateEntry, FieldEntry as GrammarFieldEntry, FieldKind as GrammarFieldKind,
    GrammarPayload, GrammarSurface, GrammarVerb, TagEntry as GrammarTagEntry,
    GRAMMAR_PAYLOAD_VERSION,
};
#[allow(unused_imports)]
pub use handler_index::{
    explain_capture_handler_ranking, rank_handlers_for_target, rank_scripts_handling_capture,
    CaptureHandlerRankingExplanation, CaptureHandlerRankingRow, HandlerScore, RankedHandler,
};
#[allow(unused_imports)]
pub use history::{
    build_argv_pool, build_tag_pool, build_value_pool, read_argv_pool, read_key_pool,
    read_tag_pool, record_argv, record_tags, ArgvFrequency, ArgvHistoryEntry, CommandHistoryStore,
    HistoryStore, TagFrequency, TagHistoryEntry, ValueFrequency, ValueHistoryEntry,
    ARGV_HISTORY_FILENAME, COMMANDS_DIR, KEYS_DIR, KEY_HISTORY_SUFFIX, TAG_HISTORY_FILENAME,
};
#[allow(unused_imports)]
pub use main_hint::{
    build_menu_syntax_main_hint, MenuSyntaxCaptureValidationSnapshot,
    MenuSyntaxCaptureValidationStatus, MenuSyntaxFragmentPreviewRow, MenuSyntaxMainHintChip,
    MenuSyntaxMainHintContext, MenuSyntaxMainHintKind, MenuSyntaxMainHintRow,
    MenuSyntaxMainHintSnapshot, MenuSyntaxMainHintTone,
};
pub use metadata::{
    dynamic_capture_schema_from_spec, handler_specs_from_extra_map, handler_specs_from_value,
    handler_specs_from_yaml_like_string, parse_field_requirement_token,
};
pub use mode::{
    capture_body_boundary_has_started, capture_body_boundary_has_started_with_targets,
    free_text_for_search, input_spans_for_input, input_spans_for_input_with_targets,
    prefix_span_for_input, prefix_span_for_input_with_targets, MenuSyntaxInputSpan, MenuSyntaxMode,
};
pub use parse::{parse, MenuSyntaxParse};
pub use payload::{
    is_known_capture_target, AdvancedQuery, ArgvInvocation, ArtifactKind, CaptureAlias,
    CaptureInvocation, CommandArgSpec, CommandFlagSpec, DatePhrase, DateRole, IncompleteKind,
    IncompleteSyntax, MenuSyntaxHandlerSpec, Predicate, ShortcutPredicate, KNOWN_CAPTURE_TARGETS,
};
pub use query::parse_advanced_query;
pub use quote::quote_for_filter_value;
#[allow(unused_imports)]
pub use retention::{
    apply_retention_plan, plan_retention, AppliedRetention, PayloadListing, RetentionConfig,
    RetentionPlan, AGE_CUTOFF_DAYS_DEFAULT, HARD_CAP_DEFAULT, KEEP_NEWEST_DEFAULT,
};
#[allow(unused_imports)]
pub use schema_overrides::{
    capture_kv_enum_values_for_specs, merge_enum_with_history, RankedSlotValue, SlotValueSource,
};
#[allow(unused_imports)]
pub use skill::{skill_specs_from_value, SkillSpec};
pub use templates::render_capture_handler_template;
pub use trigger_picker::{
    build_trigger_picker_snapshot, create_capture_handler_scaffold,
    nearest_capture_target_for_slug, registered_capture_targets,
    resolve_capture_handler_destination, CaptureHandlerScaffoldEffects, CreatedCaptureHandler,
    OsCaptureHandlerScaffoldEffects, TriggerPickerAction, TriggerPickerContext, TriggerPickerMode,
    TriggerPickerRow, TriggerPickerRowKind, TriggerPickerSnapshot,
};
#[allow(unused_imports)]
pub use trigger_picker_keys::{
    apply_intent, first_selectable_index, last_selectable_index, next_selectable_index,
    prev_selectable_index, InlinePickerKeyIntent, TriggerPickerIntentOutcome,
};
