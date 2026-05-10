use std::sync::Arc;

use serde::{Deserialize, Serialize};

use super::capture_schema::{builtin_schema, validate, FieldRequirement, ValidationResult};
use super::filter::script_command_schema_for;
use super::fragments::{MenuSyntaxFragmentRole, MenuSyntaxFragmentStatus};
use super::mode::MenuSyntaxMode;
use super::payload::{
    AdvancedQuery, ArgvInvocation, ArtifactKind, CaptureAlias, CaptureInvocation, CommandArgSpec,
    CommandFlagSpec, DatePhrase, DateRole, IncompleteKind, MenuSyntaxHandlerSpec, Predicate,
    ShortcutPredicate,
};
use super::trigger_picker::{
    nearest_capture_target_for_slug, TriggerPickerAction, TriggerPickerMode, TriggerPickerRow,
    TriggerPickerRowKind, TriggerPickerSnapshot,
};
use crate::scripts::{Script, Scriptlet};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MenuSyntaxMainHintKind {
    AdvancedQueryGuide,
    CapturePickerCompanion,
    CaptureComposer,
    CommandPickerCompanion,
    CommandComposer,
    AdvancedQueryEmpty,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MenuSyntaxMainHintTone {
    Neutral,
    Accent,
    Info,
    Warning,
    Success,
    Muted,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MenuSyntaxMainHintChip {
    pub label: String,
    pub tone: MenuSyntaxMainHintTone,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MenuSyntaxMainHintRow {
    pub label: String,
    pub value: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub chips: Vec<MenuSyntaxMainHintChip>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MenuSyntaxFragmentPreviewSnapshot {
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub rows: Vec<MenuSyntaxFragmentPreviewRow>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MenuSyntaxFragmentPreviewRow {
    pub role: crate::menu_syntax::fragments::MenuSyntaxFragmentRole,
    pub label: String,
    pub value: String,
    pub source: String,
    pub source_span: (usize, usize),
    pub status: crate::menu_syntax::fragments::MenuSyntaxFragmentStatus,
    pub tone: MenuSyntaxMainHintTone,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub chips: Vec<MenuSyntaxMainHintChip>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum MenuSyntaxCaptureValidationStatus {
    Ready,
    Incomplete,
    Malformed,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MenuSyntaxCaptureValidationSnapshot {
    pub target: String,
    pub status: MenuSyntaxCaptureValidationStatus,
    pub can_submit: bool,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub missing_field_labels: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub malformed_field_label: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub malformed_reason: Option<String>,
    /// Run 12 Pass 14 — the same HUD nudge string the gate would show on
    /// Enter, surfaced in the snapshot so automation can verify it via
    /// `getState.menuSyntaxMainHint.captureValidation.hudMessage` without
    /// scraping the transient HUD overlay. Mirrors
    /// `CaptureGateDecision::hud_message()`. None on `Ready`; populated on
    /// `Incomplete` and `Malformed`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub hud_message: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MenuSyntaxMainHintSnapshot {
    pub kind: MenuSyntaxMainHintKind,
    pub raw_filter_text: String,
    pub title: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subtitle: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mode_chip: Option<MenuSyntaxMainHintChip>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status_chip: Option<MenuSyntaxMainHintChip>,
    /// Multi-chip alternative to `status_chip` for capture validation
    /// surfaces (mode chip + per-missing-field chips + ready chip). The
    /// existing `status_chip` is preserved for backward compatibility with
    /// non-capture surfaces. (Run 11 Pass 22.)
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub status_chips: Vec<MenuSyntaxMainHintChip>,
    /// Structured capture-validation receipt (Pass 22). Present only for
    /// `CaptureComposer` snapshots where the target has a registered
    /// schema (built-in or dynamic).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub capture_validation: Option<MenuSyntaxCaptureValidationSnapshot>,
    /// Date phrases the parser recognized as date-slot keys but could not
    /// interpret (e.g. `due:asdf`). Run 12 Pass 10 — wires the data layer
    /// shipped in Run 11 Pass 34's [[src/menu_syntax/date.rs#resolve_capture_dates]]
    /// into the snapshot so the renderer + state-receipt can warn the user.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub unresolved_dates: Vec<crate::menu_syntax::date::UnresolvedDate>,
    /// Inline AI proposal (Run 12 Pass 11). Set when the user pressed
    /// Cmd+Enter while composing power syntax; cleared on filter change.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub menu_syntax_ai_proposal: Option<crate::menu_syntax_ai::MenuSyntaxAiProposal>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub rows: Vec<MenuSyntaxMainHintRow>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fragment_preview: Option<MenuSyntaxFragmentPreviewSnapshot>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub primary_hint: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub secondary_hint: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub example: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub examples: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub warning: Option<String>,
    pub accessibility_label: String,
}

pub struct MenuSyntaxMainHintContext<'a> {
    pub raw_filter_text: &'a str,
    pub mode: &'a MenuSyntaxMode,
    pub popup_snapshot: Option<&'a TriggerPickerSnapshot>,
    pub popup_selected_row_id: Option<&'a str>,
    pub scripts: &'a [Arc<Script>],
    pub scriptlets: &'a [Arc<Scriptlet>],
    pub advanced_query_results_empty: bool,
    /// Run 12 Pass 11 — pending Cmd+Enter inline AI proposal threaded into
    /// the capture composer snapshot.
    pub menu_syntax_ai_proposal: Option<&'a crate::menu_syntax_ai::MenuSyntaxAiProposal>,
}

pub fn build_menu_syntax_main_hint(
    ctx: MenuSyntaxMainHintContext<'_>,
) -> Option<MenuSyntaxMainHintSnapshot> {
    if ctx.raw_filter_text.is_empty() {
        return None;
    }

    if ctx
        .mode
        .capture_composer_owns_input_for(ctx.raw_filter_text)
    {
        return capture_composer_hint(&ctx);
    }

    if ctx.advanced_query_results_empty {
        if let Some(query) = ctx.mode.advanced_query_for(ctx.raw_filter_text) {
            return advanced_query_empty_hint(ctx.raw_filter_text, query);
        }
    }

    if let Some(snapshot) = ctx.popup_snapshot {
        match snapshot.mode {
            TriggerPickerMode::AdvancedQuery
                if should_show_advanced_query_guide(ctx.raw_filter_text) =>
            {
                return advanced_query_guide_hint(ctx.raw_filter_text, snapshot);
            }
            TriggerPickerMode::Capture => return capture_picker_companion_hint(&ctx, snapshot),
            TriggerPickerMode::Command
                if !command_body_boundary_has_started(ctx.raw_filter_text) =>
            {
                return command_picker_companion_hint(&ctx, snapshot);
            }
            _ => {}
        }
    }

    if ctx.mode.command_owns_input_for(ctx.raw_filter_text) {
        return command_composer_hint(&ctx);
    }

    None
}

fn should_show_advanced_query_guide(raw_filter_text: &str) -> bool {
    let Some(body) = raw_filter_text.strip_prefix(':') else {
        return false;
    };
    if body.is_empty() {
        return true;
    }
    if body.chars().any(char::is_whitespace) {
        return false;
    }

    let lower = body.to_ascii_lowercase();
    if lower == "#" || lower == "tag:" {
        return true;
    }
    if lower.starts_with('#') {
        return false;
    }
    if lower.contains(':') {
        return lower.ends_with(':');
    }
    true
}

fn advanced_query_guide_hint(
    raw_filter_text: &str,
    snapshot: &TriggerPickerSnapshot,
) -> Option<MenuSyntaxMainHintSnapshot> {
    let body = raw_filter_text.strip_prefix(':').unwrap_or(raw_filter_text);
    let active = body.to_ascii_lowercase();

    if active == "#" {
        return Some(finalize_hint(MenuSyntaxMainHintSnapshot {
            kind: MenuSyntaxMainHintKind::AdvancedQueryGuide,
            status_chips: Vec::new(),
            capture_validation: None,
            unresolved_dates: Vec::new(),
            menu_syntax_ai_proposal: None,
            raw_filter_text: raw_filter_text.to_string(),
            title: "Filter by tag".to_string(),
            subtitle: Some(
                "After `:`, `#tag` narrows the launcher catalog to tagged items.".to_string(),
            ),
            mode_chip: Some(chip(": refine", MenuSyntaxMainHintTone::Accent)),
            status_chip: Some(chip("tag filter", MenuSyntaxMainHintTone::Neutral)),
            rows: tag_boundary_rows(),
            fragment_preview: None,
            primary_hint: Some("Type the tag name after `:#`.".to_string()),
            secondary_hint: Some("Use `;... #tag` to label saved data.".to_string()),
            example: Some(":#work type:script".to_string()),
            examples: vec![
                ":#work type:script".to_string(),
                ":#client/acme type:issue".to_string(),
                ";todo Send proposal #client/acme".to_string(),
            ],
            warning: None,
            accessibility_label: String::new(),
        }));
    }

    let (title, subtitle, status_label, rows) = if body.is_empty() {
        (
            "Refine launcher search".to_string(),
            "Use `:` to add filters, then type the words you want to match.".to_string(),
            "guide".to_string(),
            vec![
                hint_row(
                    "Filters",
                    "type, shortcut, source, plugin, name, description, alias, tag",
                ),
                hint_row(
                    "Tags",
                    "`:#work` filters tagged items; `#work` alone is normal search",
                ),
                hint_row(
                    "Search words",
                    "Anything after filters still uses launcher search",
                ),
            ],
        )
    } else {
        let matches = snapshot
            .rows
            .iter()
            .filter(|row| row.enabled)
            .filter_map(|row| row.token.as_deref())
            .take(4)
            .collect::<Vec<_>>()
            .join(", ");
        (
            "Choose a filter".to_string(),
            "Filters narrow launcher results before your search words run.".to_string(),
            "filtering".to_string(),
            vec![
                hint_row("Typed", body),
                hint_row(
                    "Matches",
                    if matches.is_empty() {
                        "No filter matches yet"
                    } else {
                        &matches
                    },
                ),
                hint_row("Example", ":type:script deploy"),
            ],
        )
    };

    Some(finalize_hint(MenuSyntaxMainHintSnapshot {
        kind: MenuSyntaxMainHintKind::AdvancedQueryGuide,
        status_chips: Vec::new(),
        capture_validation: None,
        unresolved_dates: Vec::new(),
        menu_syntax_ai_proposal: None,
        raw_filter_text: raw_filter_text.to_string(),
        title,
        subtitle: Some(subtitle),
        mode_chip: Some(chip(": refine", MenuSyntaxMainHintTone::Accent)),
        status_chip: Some(chip(&status_label, MenuSyntaxMainHintTone::Neutral)),
        rows,
        fragment_preview: None,
        primary_hint: Some("Pick a filter in the popup, or keep typing.".to_string()),
        secondary_hint: Some(
            "Refine is search only. It does not save or capture anything.".to_string(),
        ),
        example: Some(":type:script deploy".to_string()),
        examples: vec![
            ":type:script deploy".to_string(),
            ":#work type:script".to_string(),
            ":-type:app triage".to_string(),
            ":shortcut:any".to_string(),
        ],
        warning: None,
        accessibility_label: String::new(),
    }))
}

fn capture_picker_companion_hint(
    ctx: &MenuSyntaxMainHintContext<'_>,
    snapshot: &TriggerPickerSnapshot,
) -> Option<MenuSyntaxMainHintSnapshot> {
    let selected = selected_popup_row(snapshot, ctx.popup_selected_row_id);

    // No-match / create-handler-focused branch: when the typed slug has no
    // fuzzy matches, the picker renders only the "Create capture handler for
    // ;<slug>…" footer. The generic capture companion copy ("Choose a capture
    // target in the popup", body-composition hint, cross-target examples)
    // describes a different state, so emit a setup-focused hint instead.
    if let Some(row) = selected {
        if let TriggerPickerAction::CreateHandler { target: Some(slug) } = &row.action {
            let has_target_rows = snapshot
                .rows
                .iter()
                .any(|r| r.kind == TriggerPickerRowKind::CaptureTarget && r.enabled);
            if !has_target_rows {
                return Some(capture_create_handler_hint(ctx, slug));
            }
        }
    }

    let target = selected
        .and_then(|row| row.token.as_deref())
        .and_then(|token| token.strip_prefix(';'))
        .or(snapshot.target.as_deref());

    let mut rows = Vec::new();
    if let Some(row) = selected {
        if let Some(token) = row.token.as_deref() {
            rows.push(hint_row("Selected", token));
        }
        if let Some(detail) = row.detail.as_deref() {
            rows.push(hint_row("Target", detail));
        }
    }

    let title = target
        .map(|target| capture_title(target, selected.map(|row| row.title.as_str())))
        .unwrap_or_else(|| "Start a capture".to_string());

    let primary_hint = if let Some(target) = target {
        Some(format!(
            "Press Enter or Tab to accept ;{target}, then type the body."
        ))
    } else {
        Some("Choose a capture target in the popup.".to_string())
    };

    Some(finalize_hint(MenuSyntaxMainHintSnapshot {
        kind: MenuSyntaxMainHintKind::CapturePickerCompanion,
        status_chips: Vec::new(),
        capture_validation: None,
        unresolved_dates: Vec::new(),
        menu_syntax_ai_proposal: None,
        raw_filter_text: ctx.raw_filter_text.to_string(),
        title,
        subtitle: Some("Create local structured data without searching scripts.".to_string()),
        mode_chip: Some(chip("; capture", MenuSyntaxMainHintTone::Accent)),
        status_chip: None,
        rows,
        fragment_preview: None,
        primary_hint,
        secondary_hint: Some(
            "After choosing: type body text, #tags, p1-p4 priority, dates, URLs, or key=value fields."
                .to_string(),
        ),
        example: selected
            .and_then(|row| row.example.clone())
            .or_else(|| target.and_then(|t| target_examples(t).into_iter().next()))
            .or_else(|| Some(";todo Buy milk #errands p2 due:tomorrow".to_string())),
        examples: target
            .map(target_examples)
            .unwrap_or_else(|| vec![
                ";todo Buy milk #errands p2 due:tomorrow".to_string(),
                ";note Decision to ship parser first #product".to_string(),
                ";link https://zed.dev #rust title:\"GPUI notes\"".to_string(),
            ]),
        warning: None,
        accessibility_label: String::new(),
    }))
}

fn capture_create_handler_hint(
    ctx: &MenuSyntaxMainHintContext<'_>,
    slug: &str,
) -> MenuSyntaxMainHintSnapshot {
    let typed_token = format!(";{slug}");
    let mut rows = vec![
        hint_row("Action", "Create blank capture handler"),
        hint_row(
            "File",
            &format!("~/.scriptkit/plugins/main/scripts/capture-{slug}-<slug>.ts"),
        ),
        hint_row("Registers", &format!("capture.v1 target \"{slug}\"")),
    ];
    if let Some((nearest_slug, nearest_title)) = nearest_capture_target_for_slug(slug, ctx.scripts)
    {
        rows.push(hint_row(
            "Similar",
            &format!(";{nearest_slug} — {nearest_title}"),
        ));
    }

    finalize_hint(MenuSyntaxMainHintSnapshot {
        kind: MenuSyntaxMainHintKind::CapturePickerCompanion,
        status_chips: Vec::new(),
        capture_validation: None,
        unresolved_dates: Vec::new(),
        menu_syntax_ai_proposal: None,
        raw_filter_text: ctx.raw_filter_text.to_string(),
        title: format!("No capture target named {typed_token}"),
        subtitle: None,
        mode_chip: Some(chip("; capture", MenuSyntaxMainHintTone::Accent)),
        status_chip: Some(chip("new target", MenuSyntaxMainHintTone::Neutral)),
        rows,
        fragment_preview: None,
        primary_hint: Some("Press Enter to create the handler scaffold.".to_string()),
        secondary_hint: Some("Press Cmd+Enter to ask AI to draft the handler first.".to_string()),
        example: None,
        examples: Vec::new(),
        warning: None,
        accessibility_label: String::new(),
    })
}

fn capture_composer_hint(
    ctx: &MenuSyntaxMainHintContext<'_>,
) -> Option<MenuSyntaxMainHintSnapshot> {
    let (target, invocation) = match ctx.mode.capture_for(ctx.raw_filter_text) {
        Some(invocation) => (invocation.target.as_str(), Some(invocation)),
        None => match ctx.mode.incomplete_for(ctx.raw_filter_text) {
            Some(incomplete) => match &incomplete.kind {
                IncompleteKind::MissingCaptureBody(target) => (target.as_str(), None),
                _ => return None,
            },
            None => return None,
        },
    };

    let mut rows = Vec::new();
    if let Some(invocation) = invocation {
        rows.extend(capture_preview_rows(invocation));
    }
    if rows.is_empty() {
        rows.push(hint_row("Body", "Waiting for text"));
    }
    let schema = builtin_schema(target);
    let priority_allowed = schema.as_ref().is_some_and(|schema| {
        schema
            .optional
            .iter()
            .chain(schema.required.iter())
            .any(|field| matches!(field, FieldRequirement::Priority))
    });
    let priority_unset = invocation.is_none_or(|invocation| invocation.priority.is_none());
    if priority_allowed && priority_unset && raw_last_token_is_priority_prefix(ctx.raw_filter_text)
    {
        rows.push(MenuSyntaxMainHintRow {
            label: "Priority choices".to_string(),
            value: FieldRequirement::Priority.enum_values().join(" "),
            chips: vec![chip("schema", MenuSyntaxMainHintTone::Accent)],
        });
    }
    let has_tags = invocation
        .map(|invocation| !invocation.tags.is_empty())
        .unwrap_or(false);
    if !has_tags {
        rows.push(hint_row(
            "Tags",
            "Optional labels, e.g. #errands #client/acme",
        ));
    }
    let mut ranking_warning = None;
    if let Some(invocation) = invocation {
        let ranking = crate::menu_syntax::explain_capture_handler_ranking(ctx.scripts, invocation);
        if let Some(winner) = ranking.winner.as_ref() {
            rows.push(hint_row("Handler", &winner.script_name));
            rows.push(hint_row("Why selected", &winner.reason_parts.join(" · ")));
        } else {
            rows.push(hint_row(
                "Handler",
                &format!("No registered ;{target} handler"),
            ));
        }
        if !ranking.alternatives.is_empty() {
            let alternatives = ranking
                .alternatives
                .iter()
                .take(3)
                .map(|row| row.script_name.as_str())
                .collect::<Vec<_>>()
                .join(", ");
            rows.push(hint_row("Other matches", &alternatives));
        }
        if let Some(warning) = ranking.warning {
            rows.push(hint_row("Handler conflict", &warning));
            ranking_warning = Some(warning);
        }
    } else {
        rows.push(hint_row(
            "Handler",
            &format!("Best matching ;{target} handler"),
        ));
    }

    let has_body = invocation
        .map(|invocation| !invocation.body.trim().is_empty() || invocation.url.is_some())
        .unwrap_or(false);

    let clock = crate::menu_syntax::date::MenuSyntaxClock::local_now();
    let accepts = capture_accepts_for_target(ctx.scripts, target);
    let resolved = invocation.map(|inv| {
        crate::menu_syntax::date::resolve_capture_dates_with_accepts(inv, &clock, &accepts)
    });
    let (status_chips, capture_validation) = capture_validation_chips_and_snapshot(
        target,
        invocation,
        resolved.as_ref(),
        ctx.scripts,
        &accepts,
    );

    // Run 12 Pass 10 — surface unresolved date phrases (e.g. `due:asdf`)
    // by routing the live invocation through Pass-34's resolve_capture_dates.
    let unresolved_dates: Vec<crate::menu_syntax::date::UnresolvedDate> = resolved
        .as_ref()
        .map(|resolved| resolved.unresolved_dates.clone())
        .unwrap_or_default();
    if !unresolved_dates.is_empty() {
        let phrases: Vec<String> = unresolved_dates
            .iter()
            .map(|u| format!("{:?}: {}", u.role, u.source).to_lowercase())
            .collect();
        rows.push(hint_row("Unresolved", &phrases.join(", ")));
        rows.push(MenuSyntaxMainHintRow {
            label: "Date suggestions".to_string(),
            value: "today, tomorrow, friday, mon, 9am, noon, eod".to_string(),
            chips: vec![chip("schema", MenuSyntaxMainHintTone::Accent)],
        });
    }

    Some(finalize_hint(MenuSyntaxMainHintSnapshot {
        kind: MenuSyntaxMainHintKind::CaptureComposer,
        unresolved_dates,
        menu_syntax_ai_proposal: ctx.menu_syntax_ai_proposal.cloned(),
        raw_filter_text: ctx.raw_filter_text.to_string(),
        title: format!("Capture {target}"),
        subtitle: Some("Enter saves this as structured local data.".to_string()),
        mode_chip: Some(chip("; capture", MenuSyntaxMainHintTone::Accent)),
        status_chip: Some(chip(
            if has_body { "ready" } else { "needs body" },
            if has_body {
                MenuSyntaxMainHintTone::Success
            } else {
                MenuSyntaxMainHintTone::Muted
            },
        )),
        status_chips,
        capture_validation,
        rows,
        fragment_preview: invocation
            .zip(resolved.as_ref())
            .and_then(|(invocation, resolved)| {
                fragment_preview_for_capture(invocation, resolved, &clock, &accepts)
            }),
        primary_hint: Some(if has_body {
            "Press Enter to capture.".to_string()
        } else {
            "Type what you want to save.".to_string()
        }),
        secondary_hint: Some(if has_tags {
            "Tags label this capture for grouping later. They are not launcher filters here."
                .to_string()
        } else {
            "Tags group the saved item. p1-p4 sets priority; due:/at:/start: adds dates; key=value adds fields."
                .to_string()
        }),
        example: Some(
            target_examples(target)
                .into_iter()
                .next()
                .unwrap_or_else(|| format!(";{target} Example")),
        ),
        examples: target_examples(target),
        warning: ranking_warning,
        accessibility_label: String::new(),
    }))
}

fn capture_accepts_for_target(scripts: &[Arc<Script>], target: &str) -> Vec<String> {
    let mut accepts = crate::menu_syntax::capture_accepts_for_target_from_scripts(scripts, target);
    if accepts.is_empty() {
        accepts.extend(crate::menu_syntax::date::builtin_capture_accepts_for_target(target));
    }
    accepts
}

fn fragment_preview_for_capture(
    invocation: &CaptureInvocation,
    resolved: &crate::menu_syntax::date::ResolvedCaptureInvocation,
    clock: &crate::menu_syntax::date::MenuSyntaxClock,
    accepts: &[String],
) -> Option<MenuSyntaxFragmentPreviewSnapshot> {
    let mut rows = Vec::new();
    let body = resolved.body.trim();
    if !body.is_empty() {
        let (source, source_span) = source_for_text(invocation, body);
        rows.push(MenuSyntaxFragmentPreviewRow {
            role: MenuSyntaxFragmentRole::Subject,
            label: "Subject".to_string(),
            value: body.to_string(),
            source,
            source_span,
            status: MenuSyntaxFragmentStatus::Resolved,
            tone: MenuSyntaxMainHintTone::Neutral,
            chips: Vec::new(),
        });
    }

    for date in &resolved.dates {
        let has_end = date.end_iso.is_some();
        rows.push(MenuSyntaxFragmentPreviewRow {
            role: if has_end {
                MenuSyntaxFragmentRole::DateRange
            } else {
                MenuSyntaxFragmentRole::Date
            },
            label: if has_end { "Date range" } else { "Date" }.to_string(),
            value: date_preview_value(date, clock),
            source: date.source.clone(),
            source_span: source_span_for_fragment(invocation, &date.source, date.source_span),
            status: MenuSyntaxFragmentStatus::Resolved,
            tone: MenuSyntaxMainHintTone::Info,
            chips: vec![chip("resolved", MenuSyntaxMainHintTone::Info)],
        });
    }

    if let Some(duration) = resolved.duration_resolved.as_ref() {
        rows.push(MenuSyntaxFragmentPreviewRow {
            role: MenuSyntaxFragmentRole::Duration,
            label: "Duration".to_string(),
            value: format!("{} ({} minutes)", duration.source, duration.minutes),
            source: duration.source.clone(),
            source_span: source_span_for_fragment(
                invocation,
                &duration.source,
                duration.source_span,
            ),
            status: MenuSyntaxFragmentStatus::Resolved,
            tone: MenuSyntaxMainHintTone::Warning,
            chips: Vec::new(),
        });
    }

    if let Some(recurrence) = resolved.recurrence.as_ref() {
        rows.push(MenuSyntaxFragmentPreviewRow {
            role: MenuSyntaxFragmentRole::Recurrence,
            label: "Recurrence".to_string(),
            value: format!("{} ({})", recurrence.label, recurrence.rrule),
            source: recurrence.source.clone(),
            source_span: source_span_for_fragment(
                invocation,
                &recurrence.source,
                recurrence.source_span,
            ),
            status: MenuSyntaxFragmentStatus::Resolved,
            tone: MenuSyntaxMainHintTone::Success,
            chips: Vec::new(),
        });
    }

    if !resolved.tags.is_empty() {
        rows.push(MenuSyntaxFragmentPreviewRow {
            role: MenuSyntaxFragmentRole::Tag,
            label: "Tags".to_string(),
            value: resolved
                .tags
                .iter()
                .map(|tag| format!("#{tag}"))
                .collect::<Vec<_>>()
                .join(", "),
            source: String::new(),
            source_span: (0, 0),
            status: MenuSyntaxFragmentStatus::Resolved,
            tone: MenuSyntaxMainHintTone::Accent,
            chips: Vec::new(),
        });
    }

    for (key, value) in &resolved.kv {
        let source = format!("{key}={value}");
        let (_, source_span) = source_for_text(invocation, &source);
        rows.push(MenuSyntaxFragmentPreviewRow {
            role: MenuSyntaxFragmentRole::Kv,
            label: key.clone(),
            value: value.clone(),
            source,
            source_span,
            status: MenuSyntaxFragmentStatus::Resolved,
            tone: MenuSyntaxMainHintTone::Accent,
            chips: Vec::new(),
        });
    }

    for unresolved in &resolved.unresolved_dates {
        rows.push(MenuSyntaxFragmentPreviewRow {
            role: MenuSyntaxFragmentRole::Unresolved,
            label: "Unresolved date".to_string(),
            value: format!(
                "{}: {}",
                date_role_label(&unresolved.role),
                unresolved.source
            ),
            source: unresolved.source.clone(),
            source_span: unresolved.source_span,
            status: MenuSyntaxFragmentStatus::Unresolved,
            tone: MenuSyntaxMainHintTone::Muted,
            chips: Vec::new(),
        });
    }

    if rows.is_empty()
        && invocation.body.trim().is_empty()
        && accepts.is_empty()
        && invocation.tags.is_empty()
        && invocation.kv.is_empty()
    {
        return None;
    }

    (!rows.is_empty()).then_some(MenuSyntaxFragmentPreviewSnapshot { rows })
}

fn date_preview_value(
    date: &crate::menu_syntax::date::ResolvedDate,
    clock: &crate::menu_syntax::date::MenuSyntaxClock,
) -> String {
    let mut value = date.source.clone();
    let resolved = if let Some(end_iso) = date.end_iso.as_deref() {
        format!(
            "resolved {}-{} {}",
            compact_datetime(&date.iso),
            compact_time(end_iso),
            clock.timezone_label
        )
    } else {
        format!(
            "resolved {} {}",
            compact_datetime(&date.iso),
            clock.timezone_label
        )
    };
    value.push_str(" (");
    value.push_str(&resolved);
    value.push(')');
    value
}

fn compact_datetime(iso: &str) -> String {
    chrono::DateTime::parse_from_rfc3339(iso)
        .map(|dt| crate::formatting::format_absolute_datetime(dt.with_timezone(&chrono::Utc)))
        .unwrap_or_else(|_| iso.to_string())
}

fn compact_time(iso: &str) -> String {
    chrono::DateTime::parse_from_rfc3339(iso)
        .map(|dt| dt.format("%H:%M").to_string())
        .unwrap_or_else(|_| iso.to_string())
}

fn source_for_text(invocation: &CaptureInvocation, text: &str) -> (String, (usize, usize)) {
    let span = source_span_for_text(invocation, text).unwrap_or((0, 0));
    let source = if span.0 < span.1 {
        invocation.raw[span.0..span.1].to_string()
    } else {
        String::new()
    };
    (source, span)
}

fn source_span_for_fragment(
    invocation: &CaptureInvocation,
    source: &str,
    span: (usize, usize),
) -> (usize, usize) {
    if span.1 <= invocation.raw.len()
        && invocation.raw.is_char_boundary(span.0)
        && invocation.raw.is_char_boundary(span.1)
        && invocation.raw.get(span.0..span.1) == Some(source)
    {
        return span;
    }
    source_span_for_text(invocation, source).unwrap_or((0, 0))
}

fn source_span_for_text(invocation: &CaptureInvocation, text: &str) -> Option<(usize, usize)> {
    if text.is_empty() {
        return None;
    }
    let start = crate::menu_syntax::prefix_span_for_input(&invocation.raw)
        .map(|range| range.end)
        .unwrap_or(0);
    let relative = invocation.raw[start..].find(text)?;
    let begin = start + relative;
    let end = begin + text.len();
    (invocation.raw.is_char_boundary(begin) && invocation.raw.is_char_boundary(end))
        .then_some((begin, end))
}

fn command_picker_companion_hint(
    ctx: &MenuSyntaxMainHintContext<'_>,
    snapshot: &TriggerPickerSnapshot,
) -> Option<MenuSyntaxMainHintSnapshot> {
    let selected = selected_popup_row(snapshot, ctx.popup_selected_row_id);
    if let Some(row) = selected {
        if !row.enabled || row.badges.iter().any(|badge| badge == "duplicate") {
            return Some(finalize_hint(MenuSyntaxMainHintSnapshot {
                kind: MenuSyntaxMainHintKind::CommandPickerCompanion,
                status_chips: Vec::new(),
                capture_validation: None,
                unresolved_dates: Vec::new(),
                menu_syntax_ai_proposal: None,
                raw_filter_text: ctx.raw_filter_text.to_string(),
                title: "Ambiguous command".to_string(),
                subtitle: Some("Multiple registered commands share this command head.".to_string()),
                mode_chip: Some(chip("> run", MenuSyntaxMainHintTone::Accent)),
                status_chip: Some(chip("blocked", MenuSyntaxMainHintTone::Warning)),
                rows: row
                    .token
                    .as_deref()
                    .map(|token| vec![hint_row("Command", token)])
                    .unwrap_or_default(),
                fragment_preview: None,
                primary_hint: Some("Enter will not run this until aliases are unique.".to_string()),
                secondary_hint: Some(
                    "Give one command a unique alias or choose another command.".to_string(),
                ),
                example: None,
                examples: vec![">deploy env:dev #demo -- --dry-run".to_string()],
                warning: row.detail.clone().or_else(|| {
                    Some("Ambiguous command head; give one command a unique alias.".to_string())
                }),
                accessibility_label: String::new(),
            }));
        }
    }

    let mut rows = Vec::new();
    if let Some(row) = selected {
        if let Some(token) = row.token.as_deref() {
            rows.push(hint_row("Selected", token));
        }
        if let Some(kind) = row.badges.first() {
            rows.push(hint_row("Kind", kind));
        }
    }

    Some(finalize_hint(MenuSyntaxMainHintSnapshot {
        kind: MenuSyntaxMainHintKind::CommandPickerCompanion,
        status_chips: Vec::new(),
        capture_validation: None,
        unresolved_dates: Vec::new(),
        menu_syntax_ai_proposal: None,
        raw_filter_text: ctx.raw_filter_text.to_string(),
        title: selected
            .map(|row| format!("Run {}", row.token.as_deref().unwrap_or("a command")))
            .unwrap_or_else(|| "Run a registered command".to_string()),
        subtitle: Some("Choose a registered Script Kit command in the popup.".to_string()),
        mode_chip: Some(chip("> run", MenuSyntaxMainHintTone::Accent)),
        status_chip: None,
        rows,
        fragment_preview: None,
        primary_hint: Some("After choosing: add fields, #tags, or argv after --.".to_string()),
        secondary_hint: None,
        example: selected
            .and_then(|row| row.example.clone())
            .or_else(|| Some(">ps-env env:dev #demo -- --dry-run".to_string())),
        examples: vec![
            ">ps-env env:dev #demo -- --dry-run".to_string(),
            ">test-menu-syntax -- --watch".to_string(),
        ],
        warning: None,
        accessibility_label: String::new(),
    }))
}

fn command_composer_hint(
    ctx: &MenuSyntaxMainHintContext<'_>,
) -> Option<MenuSyntaxMainHintSnapshot> {
    let invocation = ctx.mode.command_for(ctx.raw_filter_text)?;
    let resolution = resolve_command(invocation, ctx.scripts, ctx.scriptlets);

    let mut rows = command_preview_rows(invocation);
    if rows.is_empty() {
        rows.push(hint_row("Command", &format!(">{}", invocation.head)));
    }

    // Author-declared command schema rows: if any loaded script registers a
    // `command.v1` handler with `head` matching this invocation, append rows
    // describing the expected args + flags. These are the rows the
    // `setFilter ">deploy"` getState receipt looks for in
    // `menuSyntaxMainHint.rows`.
    let command_schema = script_command_schema_for(ctx.scripts, &invocation.head);
    if let Some(spec) = command_schema.as_ref() {
        rows.extend(command_schema_rows(spec));
    }

    let (title, status_chip, primary_hint, warning) = match resolution {
        CommandHintResolution::Unique { title, kind } => {
            rows.insert(0, hint_row("Target", &format!("{title} ({kind})")));
            (
                format!("Run {}", invocation.head),
                Some(chip("ready", MenuSyntaxMainHintTone::Success)),
                "Press Enter to run the registered Script Kit command.".to_string(),
                None,
            )
        }
        CommandHintResolution::Ambiguous { count } => (
            "Ambiguous command".to_string(),
            Some(chip("blocked", MenuSyntaxMainHintTone::Warning)),
            "Enter will not run this while multiple commands match.".to_string(),
            Some(format!(
                "{count} registered commands use >{}. Give one command a unique alias.",
                invocation.head
            )),
        ),
        CommandHintResolution::None => (
            format!("No registered command named !{}", invocation.head),
            Some(chip("not found", MenuSyntaxMainHintTone::Warning)),
            "This will not run a shell command.".to_string(),
            Some("Backspace to search normally, or type ! to choose a command.".to_string()),
        ),
    };

    Some(finalize_hint(MenuSyntaxMainHintSnapshot {
        kind: MenuSyntaxMainHintKind::CommandComposer,
        status_chips: Vec::new(),
        capture_validation: None,
        unresolved_dates: Vec::new(),
        menu_syntax_ai_proposal: None,
        raw_filter_text: ctx.raw_filter_text.to_string(),
        title,
        subtitle: Some("Command invocation is explicit Script Kit execution.".to_string()),
        mode_chip: Some(chip("> run", MenuSyntaxMainHintTone::Accent)),
        status_chip,
        rows,
        fragment_preview: None,
        primary_hint: Some(primary_hint),
        secondary_hint: Some(
            "Fields use key:value; #tags are command metadata; argv after -- is passed through."
                .to_string(),
        ),
        example: Some(format!(">{} env:dev #demo -- --dry-run", invocation.head)),
        examples: vec![
            format!(">{} env:dev #demo -- --dry-run", invocation.head),
            format!(">{} -- --help", invocation.head),
        ],
        warning,
        accessibility_label: String::new(),
    }))
}

fn advanced_query_empty_hint(
    raw_filter_text: &str,
    query: &AdvancedQuery,
) -> Option<MenuSyntaxMainHintSnapshot> {
    let mut rows = Vec::new();
    if !query.predicates.is_empty() {
        rows.push(MenuSyntaxMainHintRow {
            label: "Filters".to_string(),
            value: query
                .predicates
                .iter()
                .map(predicate_user_label)
                .collect::<Vec<_>>()
                .join(" · "),
            chips: Vec::new(),
        });
    }
    if !query.free_text.is_empty() {
        rows.push(hint_row("Search words", &query.free_text));
    }

    let tag = query.predicates.iter().find_map(predicate_tag);
    let title = tag
        .map(|tag| format!("No launcher items tagged #{tag}"))
        .unwrap_or_else(|| "No matches after these filters".to_string());
    let subtitle = tag
        .map(|tag| format!("After `:`, `#{tag}` is a tag filter on the launcher catalog."))
        .unwrap_or_else(|| "`:` narrows the launcher catalog before search words run.".to_string());
    let primary_hint = tag
        .map(|tag| format!("Try another tag, remove `:#{tag}`, or change the search words."))
        .unwrap_or_else(|| "Remove a filter or change the search words.".to_string());

    Some(finalize_hint(MenuSyntaxMainHintSnapshot {
        kind: MenuSyntaxMainHintKind::AdvancedQueryEmpty,
        status_chips: Vec::new(),
        capture_validation: None,
        unresolved_dates: Vec::new(),
        menu_syntax_ai_proposal: None,
        raw_filter_text: raw_filter_text.to_string(),
        title,
        subtitle: Some(subtitle),
        mode_chip: Some(chip(": refine", MenuSyntaxMainHintTone::Accent)),
        status_chip: Some(chip("no matches", MenuSyntaxMainHintTone::Muted)),
        rows,
        fragment_preview: None,
        primary_hint: Some(primary_hint),
        secondary_hint: Some(
            "Use `:#work` to filter by tag. Plain `#work` is normal launcher search.".to_string(),
        ),
        example: Some(":type:script deploy".to_string()),
        examples: vec![
            ":#work".to_string(),
            ":tag:work".to_string(),
            ":type:script deploy".to_string(),
        ],
        warning: None,
        accessibility_label: String::new(),
    }))
}

fn tag_boundary_rows() -> Vec<MenuSyntaxMainHintRow> {
    vec![
        hint_row("#work", "Plain launcher search"),
        hint_row(":#work", "Filter launcher rows tagged #work"),
        hint_row(";... #work", "Label the captured item as #work"),
    ]
}

/// Build the multi-chip status row + structured `CaptureValidation`
/// snapshot for a `;target` capture composer state. Returns the chip Vec
/// (always at least the mode chip) and the optional validation snapshot
/// (None when the target has no registered schema, e.g. `;github`).
///
/// Story: capture-validation-snapshot (Pass 22). Wires
/// [[crate::menu_syntax::capture_schema::validate]] into the snapshot the
/// hint card consumes — so the UI can render `; capture` / `needs body`
/// chips without re-running the validation rules.
fn capture_validation_chips_and_snapshot(
    target: &str,
    invocation: Option<&CaptureInvocation>,
    resolved: Option<&crate::menu_syntax::date::ResolvedCaptureInvocation>,
    scripts: &[Arc<Script>],
    accepts: &[String],
) -> (
    Vec<MenuSyntaxMainHintChip>,
    Option<MenuSyntaxCaptureValidationSnapshot>,
) {
    let mut chips = vec![chip("; capture", MenuSyntaxMainHintTone::Accent)];
    // Run 12 Pass 15 — `capture-dynamic-target-schema`. Resolve the schema
    // through the script-aware lookup so script-declared `capture.v1`
    // specs (e.g. `;expense` from `capture-expense-ledger.ts`) flow into
    // the live snapshot's `captureValidation`. Builtin still wins when
    // present; falls back to the first matching dynamic schema.
    let Some(schema) =
        crate::menu_syntax::capture_gate::resolve_capture_schema_for_target(target, scripts)
    else {
        return (chips, None);
    };
    // Use a synthetic empty payload when the user has only typed `;target ` —
    // the schema's `missing_required` still computes the correct Vec from
    // an empty body / no date_phrases / no kv. This matches the receipt
    // contract: `setFilter ";cal"` reports both `needs body` and `needs date`.
    let synthetic;
    let payload = match invocation {
        Some(inv) => inv,
        None => {
            synthetic = CaptureInvocation {
                target: target.to_string(),
                alias_form: CaptureAlias::CapturePrefix,
                body: String::new(),
                tags: vec![],
                priority: None,
                url: None,
                duration: None,
                kv: vec![],
                date_phrases: vec![],
                raw: format!(";{target}"),
            };
            &synthetic
        }
    };
    let validation_payload = payload_for_capture_validation(payload, resolved);
    let result = validate(&validation_payload, &schema);
    // Run 12 Pass 14 — re-run the gate on the same payload+schema so the
    // exact HUD nudge string the runtime would show on Enter is part of the
    // snapshot. Pure read; same inputs as `validate` so no extra branching.
    let gate_decision = crate::menu_syntax::capture_gate::decide_capture_gate_with_accepts(
        &validation_payload,
        Some(&schema),
        accepts,
    );
    let hud_message = gate_decision.hud_message().map(|s| s.to_string());
    let snapshot = match &result {
        ValidationResult::Ready => {
            chips.push(chip("ready", MenuSyntaxMainHintTone::Success));
            MenuSyntaxCaptureValidationSnapshot {
                target: schema.target.clone(),
                status: MenuSyntaxCaptureValidationStatus::Ready,
                can_submit: true,
                missing_field_labels: Vec::new(),
                malformed_field_label: None,
                malformed_reason: None,
                hud_message,
            }
        }
        ValidationResult::Incomplete { missing } => {
            let labels: Vec<String> = missing.iter().map(|req| req.label()).collect();
            for label in &labels {
                chips.push(chip(
                    &format!("needs {label}"),
                    MenuSyntaxMainHintTone::Muted,
                ));
            }
            MenuSyntaxCaptureValidationSnapshot {
                target: schema.target.clone(),
                status: MenuSyntaxCaptureValidationStatus::Incomplete,
                can_submit: false,
                missing_field_labels: labels,
                malformed_field_label: None,
                malformed_reason: None,
                hud_message,
            }
        }
        ValidationResult::Malformed { field, reason } => {
            chips.push(chip("malformed", MenuSyntaxMainHintTone::Warning));
            MenuSyntaxCaptureValidationSnapshot {
                target: schema.target.clone(),
                status: MenuSyntaxCaptureValidationStatus::Malformed,
                can_submit: false,
                missing_field_labels: Vec::new(),
                malformed_field_label: Some(field.label()),
                malformed_reason: Some(reason.clone()),
                hud_message,
            }
        }
    };
    (chips, Some(snapshot))
}

fn payload_for_capture_validation(
    payload: &CaptureInvocation,
    resolved: Option<&crate::menu_syntax::date::ResolvedCaptureInvocation>,
) -> CaptureInvocation {
    let Some(resolved) = resolved else {
        return payload.clone();
    };
    let mut validation_payload = payload.clone();
    validation_payload.body = resolved.body.clone();
    validation_payload.duration = resolved.duration.clone();
    validation_payload.date_phrases = resolved
        .dates
        .iter()
        .map(|date| DatePhrase {
            role: date.role.clone(),
            source: date.source.clone(),
            source_span: date.source_span,
        })
        .collect();
    validation_payload
}

fn capture_preview_rows(invocation: &CaptureInvocation) -> Vec<MenuSyntaxMainHintRow> {
    let mut rows = Vec::new();
    let body = invocation.body.trim();
    if !body.is_empty() {
        rows.push(hint_row("Body", body));
    }
    if !invocation.tags.is_empty() {
        rows.push(hint_row(
            "Tags",
            &invocation
                .tags
                .iter()
                .map(|tag| format!("#{tag}"))
                .collect::<Vec<_>>()
                .join(" "),
        ));
    }
    if let Some(priority) = invocation.priority {
        rows.push(hint_row("Priority", &format!("P{priority}")));
    }
    if let Some(url) = invocation.url.as_deref() {
        rows.push(hint_row("URL", url));
    }
    if let Some(duration) = invocation.duration.as_deref() {
        rows.push(hint_row("Duration", duration));
    }
    if !invocation.date_phrases.is_empty() {
        rows.push(hint_row(
            "Dates",
            &invocation
                .date_phrases
                .iter()
                .map(|date| format!("{}:{}", date_role_label(&date.role), date.source))
                .collect::<Vec<_>>()
                .join(" | "),
        ));
    }
    if !invocation.kv.is_empty() {
        rows.push(hint_row(
            "Fields",
            &invocation
                .kv
                .iter()
                .map(|(key, value)| format!("{key}={value}"))
                .collect::<Vec<_>>()
                .join(" | "),
        ));
    }
    rows.truncate(5);
    rows
}

fn command_preview_rows(invocation: &ArgvInvocation) -> Vec<MenuSyntaxMainHintRow> {
    let mut rows = Vec::new();
    if !invocation.fields.is_empty() {
        rows.push(hint_row(
            "Fields",
            &invocation
                .fields
                .iter()
                .map(|(key, value)| format!("{key}={value}"))
                .collect::<Vec<_>>()
                .join(" | "),
        ));
    }
    if !invocation.tags.is_empty() {
        rows.push(hint_row(
            "Tags",
            &invocation
                .tags
                .iter()
                .map(|tag| format!("#{tag}"))
                .collect::<Vec<_>>()
                .join(" "),
        ));
    }
    if !invocation.argv.is_empty() {
        rows.push(hint_row("Argv", &invocation.argv.join(" ")));
    }
    rows
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum CommandHintResolution {
    None,
    Unique { title: String, kind: &'static str },
    Ambiguous { count: usize },
}

fn resolve_command(
    invocation: &ArgvInvocation,
    scripts: &[Arc<Script>],
    scriptlets: &[Arc<Scriptlet>],
) -> CommandHintResolution {
    let mut matches = Vec::new();
    for script in scripts {
        let mut hit =
            super::command_head_matches(&invocation.head, &super::script_command_head(script));
        if !hit {
            // Run 13 Pass 5 — also accept author-declared `command.v1` heads
            // from the script's menuSyntax metadata. Without this, scripts
            // that declare `head: "deploy"` (e.g. via the !head pivot) get
            // a misleading `not found` chip even though the command schema
            // rows below are populated from the same metadata.
            if let Some(declared) = super::first_command_head_for_script(script) {
                if super::command_head_matches(&invocation.head, &declared) {
                    hit = true;
                }
            }
        }
        if hit {
            matches.push((script.name.clone(), "script"));
        }
    }
    for scriptlet in scriptlets {
        if super::command_head_matches(&invocation.head, &super::scriptlet_command_head(scriptlet))
        {
            matches.push((scriptlet.name.clone(), "scriptlet"));
        }
    }

    match matches.len() {
        0 => CommandHintResolution::None,
        1 => {
            let (title, kind) = matches.remove(0);
            CommandHintResolution::Unique { title, kind }
        }
        count => CommandHintResolution::Ambiguous { count },
    }
}

fn selected_popup_row<'a>(
    snapshot: &'a TriggerPickerSnapshot,
    selected_row_id: Option<&str>,
) -> Option<&'a TriggerPickerRow> {
    selected_row_id
        .and_then(|id| snapshot.rows.iter().find(|row| row.id == id))
        .or_else(|| snapshot.rows.iter().find(|row| row.enabled))
        .or_else(|| snapshot.rows.first())
}

fn command_body_boundary_has_started(input: &str) -> bool {
    let Some(rest) = input.strip_prefix('>') else {
        return false;
    };
    let rest = rest.trim_start();
    if rest.is_empty() {
        return false;
    }
    rest.find(char::is_whitespace)
        .map(|idx| idx > 0)
        .unwrap_or(false)
}

/// Per-target example list shown in the hint card. Each entry MUST start with
/// `;<target>` (no cross-target leakage like a `;todo` example sneaking into a
/// `;cal` hint) and SHOULD include the target's required field slots so the
/// example doubles as a fix-it template the user can paste-and-edit. Falls back
/// to a generic `;todo` row for unknown targets so the hint is never empty.
/// Story: `hint-examples-target-relevant` (Run 12 user priority #3). Also
/// used by [[src/menu_syntax/capture_gate.rs#decide_capture_gate]] (Run 12
/// Pass 3) to build the HUD nudge's fix-it suggestion when Enter is blocked.
pub(crate) fn target_examples(target: &str) -> Vec<String> {
    match target {
        "cal" => vec![
            ";cal Design review start:\"friday 2pm\" #work".to_string(),
            ";cal Lunch with Anna at:\"tomorrow 12:30pm\"".to_string(),
            ";cal Sprint demo start:\"mon 10am\" end:\"mon 11am\"".to_string(),
        ],
        "todo" => vec![
            ";todo Buy milk #errands p2 due:tomorrow".to_string(),
            ";todo Send proposal #client/acme p1 due:friday".to_string(),
            ";todo Renew passport due:eom".to_string(),
        ],
        "note" => vec![
            ";note Decision to ship parser first #product".to_string(),
            ";note Q2 retrospective takeaways #team".to_string(),
            ";note Coffee chat with Sam — follow up on hiring".to_string(),
        ],
        "link" => vec![
            ";link https://zed.dev #rust title:\"GPUI notes\"".to_string(),
            ";link https://news.ycombinator.com #read-later".to_string(),
            ";link https://docs.rs/chrono #reference title:\"chrono docs\"".to_string(),
        ],
        "social" => vec![
            ";social Shipped the new launcher chrome — feedback welcome #build".to_string(),
            ";social Reading 'Designing Data-Intensive Applications' — solid #books".to_string(),
            ";social TIL: GPUI's flex_col gap saves a lot of margin churn #rust".to_string(),
        ],
        "mcal" => vec![
            ";mcal Lunch with Ryan tomorrow at 12pm til 1pm calendar=Work".to_string(),
            ";mcal Design review start:\"friday 2pm\" for 45m alarm=15".to_string(),
            ";mcal Team sync every mon from 1 til 2 calendar=Work".to_string(),
        ],
        "gcal" => vec![
            ";gcal Design review tomorrow 2pm for 45m calendarId=primary location=\"Zoom\""
                .to_string(),
            ";gcal Project kickoff start:\"friday 10am\" end:\"friday 11am\" guests=ada@example.com".to_string(),
            ";gcal Weekly planning every mon at 9am calendarId=primary location=\"Google Meet\"".to_string(),
        ],
        "reminder" => vec![
            ";reminder Submit expense report tomorrow #admin".to_string(),
            ";reminder Walk dog every day at 8am #home".to_string(),
            ";reminder Renew passport next month #errands".to_string(),
        ],
        "snooze" => vec![
            ";snooze in 30 minutes Review PR #code".to_string(),
            ";snooze tomorrow morning Reply to Sam #follow-up".to_string(),
            ";snooze next monday Revisit launch checklist".to_string(),
        ],
        "defer" => vec![
            ";defer until next week Refactor settings panel p2".to_string(),
            ";defer friday Follow up on vendor quote #ops".to_string(),
            ";defer in 2 days Triage plugin docs".to_string(),
        ],
        "github" => vec![
            ";github johnlindquist/kit Fix popup focus #bug p1".to_string(),
            ";github Review OAuth examples repo=johnlindquist/kit #demo".to_string(),
            ";github Track flaky CI url:https://github.com/johnlindquist/kit/actions #ci"
                .to_string(),
        ],
        "expense" => vec![
            ";expense Coffee amount=4.75 vendor=Bluebird #travel".to_string(),
            ";expense Client lunch amount=38.20 currency=USD vendor=\"Cafe Rio\" #client/acme"
                .to_string(),
            ";expense Taxi amount=22.00 reimbursable=true project=offsite #transport".to_string(),
        ],
        "snippet" => vec![
            ";snippet parse_capture helper lang=rust title=\"Capture parser\" #rust".to_string(),
            ";snippet Promise timeout wrapper lang=ts url:https://example.test/snippet #typescript"
                .to_string(),
            ";snippet jq filter for events lang=sh title=\"JSONL event count\"".to_string(),
        ],
        "fixture" => vec![
            ";fixture Validate metadata filter env=dev project=launcher #demo".to_string(),
            ";fixture Parser smoke case kind=search tag=power-syntax".to_string(),
            ";fixture Snapshot row state=ready owner=qa #fixture".to_string(),
        ],
        other => vec![
            format!(";{other} Capture useful context #inbox"),
            format!(";{other} Follow up with team owner=me"),
            format!(";{other} Save this for later status=open"),
        ],
    }
}

fn capture_title(target: &str, row_title: Option<&str>) -> String {
    match row_title {
        Some(title) if !title.eq_ignore_ascii_case("Capture target") => title.to_string(),
        _ => format!("Capture {target}"),
    }
}

fn date_role_label(role: &DateRole) -> &'static str {
    match role {
        DateRole::Due => "due",
        DateRole::At => "at",
        DateRole::Start => "start",
        DateRole::End => "end",
        DateRole::Inferred => "date",
    }
}

#[cfg(test)]
fn predicate_label(predicate: &Predicate) -> String {
    match predicate {
        Predicate::Type(kind) => format!("type:{}", artifact_kind_label(kind)),
        Predicate::Tag(tag) => format!("tag:{tag}"),
        Predicate::HasShortcut(ShortcutPredicate::Any) => "shortcut:any".to_string(),
        Predicate::HasShortcut(ShortcutPredicate::None) => "shortcut:none".to_string(),
        Predicate::HasShortcut(ShortcutPredicate::Literal(shortcut)) => {
            format!("shortcut:{shortcut}")
        }
        Predicate::Source(source) => format!("source:{source}"),
        Predicate::Plugin(plugin) => format!("plugin:{plugin}"),
        Predicate::Name(name) => format!("name:{name}"),
        Predicate::Desc(desc) => format!("desc:{desc}"),
        Predicate::Alias(alias) => format!("alias:{alias}"),
        Predicate::Has(has) => format!("has:{has}"),
        Predicate::MetaPath { path, value } => format!("meta.{path}:{value}"),
        Predicate::Negate(inner) => format!("-{}", predicate_label(inner)),
    }
}

fn predicate_user_label(predicate: &Predicate) -> String {
    match predicate {
        Predicate::Type(kind) => match kind {
            ArtifactKind::Script => "scripts only".to_string(),
            ArtifactKind::Scriptlet => "scriptlets only".to_string(),
            ArtifactKind::Skill => "skills only".to_string(),
            ArtifactKind::Agent => "agents only".to_string(),
            ArtifactKind::Builtin => "built-ins only".to_string(),
            ArtifactKind::App => "apps only".to_string(),
            ArtifactKind::Window => "windows only".to_string(),
            ArtifactKind::File => "files only".to_string(),
            ArtifactKind::AcpHistory => "AI conversations only".to_string(),
            ArtifactKind::ClipboardHistory => "clipboard history only".to_string(),
            ArtifactKind::Fallback => "fallbacks only".to_string(),
            ArtifactKind::Issue => "issues only".to_string(),
        },
        Predicate::Tag(tag) => format!("#{tag}"),
        Predicate::HasShortcut(ShortcutPredicate::Any) => "has shortcut".to_string(),
        Predicate::HasShortcut(ShortcutPredicate::None) => "no shortcut".to_string(),
        Predicate::HasShortcut(ShortcutPredicate::Literal(shortcut)) => {
            format!("shortcut {shortcut}")
        }
        Predicate::Source(source) => format!("source {source}"),
        Predicate::Plugin(plugin) => format!("plugin {plugin}"),
        Predicate::Name(name) => format!("name contains {name}"),
        Predicate::Desc(desc) => format!("description contains {desc}"),
        Predicate::Alias(alias) => format!("alias contains {alias}"),
        Predicate::Has(has) => format!("has {has}"),
        Predicate::MetaPath { path, value } => format!("metadata {path} is {value}"),
        Predicate::Negate(inner) => match inner.as_ref() {
            Predicate::Type(ArtifactKind::App) => "exclude apps".to_string(),
            _ => format!("exclude {}", predicate_user_label(inner)),
        },
    }
}

fn predicate_tag(predicate: &Predicate) -> Option<&str> {
    match predicate {
        Predicate::Tag(tag) => Some(tag.as_str()),
        Predicate::Negate(inner) => predicate_tag(inner),
        _ => None,
    }
}

#[cfg(test)]
fn artifact_kind_label(kind: &ArtifactKind) -> &'static str {
    match kind {
        ArtifactKind::Script => "script",
        ArtifactKind::Scriptlet => "scriptlet",
        ArtifactKind::Skill => "skill",
        ArtifactKind::Agent => "agent",
        ArtifactKind::Builtin => "builtin",
        ArtifactKind::App => "app",
        ArtifactKind::Window => "window",
        ArtifactKind::Fallback => "fallback",
        ArtifactKind::Issue => "issue",
    }
}

/// Render hint rows from an author-declared `command.v1` schema. One row
/// per arg (`label = arg.name`), one per flag (`label = flag.name`). Each
/// row carries a "required" chip when applicable and either the
/// description, allowed values, or example as the value text. The order
/// matters — the `setFilter ">deploy"` getState receipt looks for the
/// arg names ("env") and flag names ("--dry-run") in
/// `menuSyntaxMainHint.rows[].label`.
fn command_schema_rows(spec: &MenuSyntaxHandlerSpec) -> Vec<MenuSyntaxMainHintRow> {
    let mut rows = Vec::with_capacity(spec.args.len() + spec.flags.len());
    for arg in &spec.args {
        rows.push(arg_to_row(arg));
    }
    for flag in &spec.flags {
        rows.push(flag_to_row(flag));
    }
    rows
}

fn arg_to_row(arg: &CommandArgSpec) -> MenuSyntaxMainHintRow {
    let mut chips = Vec::new();
    if arg.required {
        chips.push(chip("required", MenuSyntaxMainHintTone::Warning));
    }
    let value = describe_arg_value(arg);
    MenuSyntaxMainHintRow {
        label: arg.name.clone(),
        value: truncate_hint_value(&value),
        chips,
    }
}

fn flag_to_row(flag: &CommandFlagSpec) -> MenuSyntaxMainHintRow {
    let mut chips = Vec::new();
    if flag.required {
        chips.push(chip("required", MenuSyntaxMainHintTone::Warning));
    }
    let value = describe_flag_value(flag);
    MenuSyntaxMainHintRow {
        label: flag.name.clone(),
        value: truncate_hint_value(&value),
        chips,
    }
}

fn describe_arg_value(arg: &CommandArgSpec) -> String {
    if !arg.values.is_empty() {
        return arg.values.join(" | ");
    }
    if let Some(desc) = arg.description.as_deref() {
        return desc.to_string();
    }
    if let Some(ex) = arg.example.as_deref() {
        return format!("e.g. {ex}");
    }
    String::new()
}

fn describe_flag_value(flag: &CommandFlagSpec) -> String {
    let mut parts = Vec::new();
    if let Some(alias) = flag.alias.as_deref() {
        parts.push(format!("alias {alias}"));
    }
    if !flag.values.is_empty() {
        parts.push(flag.values.join(" | "));
    }
    if let Some(desc) = flag.description.as_deref() {
        parts.push(desc.to_string());
    }
    if parts.is_empty() {
        if let Some(ex) = flag.example.as_deref() {
            parts.push(format!("e.g. {ex}"));
        }
    }
    parts.join(" — ")
}

fn hint_row(label: &str, value: &str) -> MenuSyntaxMainHintRow {
    MenuSyntaxMainHintRow {
        label: label.to_string(),
        value: truncate_hint_value(value),
        chips: Vec::new(),
    }
}

fn chip(label: &str, tone: MenuSyntaxMainHintTone) -> MenuSyntaxMainHintChip {
    MenuSyntaxMainHintChip {
        label: label.to_string(),
        tone,
    }
}

fn truncate_hint_value(value: &str) -> String {
    const MAX_CHARS: usize = 80;
    if value.chars().count() <= MAX_CHARS {
        return value.to_string();
    }
    let mut out: String = value.chars().take(MAX_CHARS.saturating_sub(3)).collect();
    out.push_str("...");
    out
}

fn finalize_hint(mut hint: MenuSyntaxMainHintSnapshot) -> MenuSyntaxMainHintSnapshot {
    hint.raw_filter_text = truncate_raw_filter_text(&hint.raw_filter_text);
    hint.accessibility_label = accessibility_label(&hint);
    hint
}

fn truncate_raw_filter_text(value: &str) -> String {
    const MAX_CHARS: usize = 200;
    if value.chars().count() <= MAX_CHARS {
        return value.to_string();
    }
    let mut out: String = value.chars().take(MAX_CHARS.saturating_sub(3)).collect();
    out.push_str("...");
    out
}

fn raw_last_token_is_priority_prefix(raw: &str) -> bool {
    matches!(raw.split_whitespace().last(), Some(t) if t.eq_ignore_ascii_case("p"))
}

fn accessibility_label(hint: &MenuSyntaxMainHintSnapshot) -> String {
    let mut parts = vec![hint.title.clone()];
    if let Some(subtitle) = hint.subtitle.as_ref() {
        parts.push(subtitle.clone());
    }
    if let Some(primary) = hint.primary_hint.as_ref() {
        parts.push(primary.clone());
    }
    for row in &hint.rows {
        parts.push(format!("{} {}", row.label, row.value));
    }
    if let Some(warning) = hint.warning.as_ref() {
        parts.push(format!("Warning: {warning}"));
    }
    parts.join(". ")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::menu_syntax::{
        build_trigger_picker_snapshot, parse_advanced_query, TriggerPickerContext,
    };
    use std::path::PathBuf;

    fn script(name: &str, alias: Option<&str>) -> Arc<Script> {
        Arc::new(Script {
            name: name.to_string(),
            alias: alias.map(str::to_string),
            path: PathBuf::from(format!("/tmp/{}.ts", name.to_ascii_lowercase())),
            extension: "ts".to_string(),
            ..Default::default()
        })
    }

    fn mcal_script() -> Arc<Script> {
        use crate::metadata_parser::TypedMetadata;
        use serde_json::json;
        use std::collections::HashMap;

        let mut extra: HashMap<String, serde_json::Value> = HashMap::new();
        extra.insert(
            "menuSyntax".to_string(),
            json!([{
                "family": "capture.v1",
                "targets": ["mcal"],
                "accepts": ["tags", "date", "dateRange", "duration", "recurrence", "kv"],
                "required": ["body", "date"],
                "label": "Add event to macOS Calendar",
                "payloadSchema": "kit://schema/menu-syntax/payload-v1",
                "defaultHandler": true
            }]),
        );
        Arc::new(Script {
            name: "Create macOS Calendar Event".to_string(),
            alias: None,
            path: PathBuf::from("/tmp/create-mac-calendar-event.ts"),
            extension: "ts".to_string(),
            typed_metadata: Some(TypedMetadata {
                extra,
                ..Default::default()
            }),
            ..Default::default()
        })
    }

    fn capture_hint_for(raw: &str, scripts: &[Arc<Script>]) -> MenuSyntaxMainHintSnapshot {
        let targets = crate::menu_syntax::registered_capture_targets_from_scripts(scripts);
        let mode = MenuSyntaxMode::from_input_with_capture_targets(raw, &targets);
        build_menu_syntax_main_hint(MenuSyntaxMainHintContext {
            raw_filter_text: raw,
            mode: &mode,
            popup_snapshot: None,
            popup_selected_row_id: None,
            scripts,
            scriptlets: &[],
            advanced_query_results_empty: false,
            menu_syntax_ai_proposal: None,
        })
        .expect("capture hint")
    }

    fn scriptlet(name: &str, command: Option<&str>) -> Arc<Scriptlet> {
        Arc::new(Scriptlet {
            name: name.to_string(),
            description: None,
            code: String::new(),
            tool: "ts".to_string(),
            shortcut: None,
            keyword: None,
            group: None,
            plugin_id: "main".to_string(),
            plugin_title: None,
            file_path: None,
            command: command.map(str::to_string),
            alias: None,
        })
    }

    #[test]
    fn unknown_slug_no_match_hint_is_setup_focused() {
        let mode = MenuSyntaxMode::from_input(";gcal");
        let snapshot = build_trigger_picker_snapshot(";gcal", &TriggerPickerContext::default())
            .expect("gcal snapshot");
        let hint = build_menu_syntax_main_hint(MenuSyntaxMainHintContext {
            raw_filter_text: ";gcal",
            mode: &mode,
            popup_snapshot: Some(&snapshot),
            popup_selected_row_id: Some("footer:create-handler"),
            scripts: &[],
            scriptlets: &[],
            advanced_query_results_empty: false,
            menu_syntax_ai_proposal: None,
        })
        .expect("hint");

        assert_eq!(hint.kind, MenuSyntaxMainHintKind::CapturePickerCompanion);
        assert_eq!(hint.title, "No capture target named ;gcal");
        assert_eq!(hint.subtitle, None);
        assert!(hint.examples.is_empty(), "no examples in no-match state");
        assert_eq!(hint.example, None);
        assert_eq!(
            hint.status_chip.as_ref().map(|c| c.label.as_str()),
            Some("new target")
        );
        assert!(hint
            .primary_hint
            .as_deref()
            .unwrap()
            .contains("Press Enter to create the handler scaffold"));
        assert!(hint
            .secondary_hint
            .as_deref()
            .unwrap()
            .contains("Cmd+Enter"));
        let row_labels: Vec<&str> = hint.rows.iter().map(|r| r.label.as_str()).collect();
        assert!(row_labels.contains(&"Action"));
        assert!(row_labels.contains(&"File"));
        assert!(row_labels.contains(&"Registers"));
        for row in &hint.rows {
            assert_ne!(row.label, "Selected");
        }
        // Near-miss "Similar" line should fire for ;gcal -> ;cal (one edit away).
        let similar = hint
            .rows
            .iter()
            .find(|r| r.label == "Similar")
            .expect("similar row for ;gcal -> ;cal");
        assert!(similar.value.contains(";cal"));
        assert!(similar.value.contains("Calendar event"));
    }

    #[test]
    fn unknown_slug_no_match_hint_drops_choose_target_copy() {
        let mode = MenuSyntaxMode::from_input(";zzzz");
        let snapshot = build_trigger_picker_snapshot(";zzzz", &TriggerPickerContext::default())
            .expect("zzzz snapshot");
        let hint = build_menu_syntax_main_hint(MenuSyntaxMainHintContext {
            raw_filter_text: ";zzzz",
            mode: &mode,
            popup_snapshot: Some(&snapshot),
            popup_selected_row_id: Some("footer:create-handler"),
            scripts: &[],
            scriptlets: &[],
            advanced_query_results_empty: false,
            menu_syntax_ai_proposal: None,
        })
        .expect("hint");

        let primary = hint.primary_hint.unwrap_or_default();
        let secondary = hint.secondary_hint.unwrap_or_default();
        assert!(!primary.contains("Choose a capture target"));
        assert!(!secondary.contains("After choosing"));
        // No near-miss for ;zzzz against built-ins (todo/cal/note/social/link).
        assert!(hint.rows.iter().all(|r| r.label != "Similar"));
    }

    #[test]
    fn known_slug_picker_companion_keeps_examples() {
        // Sanity check: the no-match branch must not steal the existing
        // ;todo/;cal/;note/;social/;link companion behavior.
        let mode = MenuSyntaxMode::from_input(";todo");
        let snapshot = build_trigger_picker_snapshot(";todo", &TriggerPickerContext::default())
            .expect("todo snapshot");
        let hint = build_menu_syntax_main_hint(MenuSyntaxMainHintContext {
            raw_filter_text: ";todo",
            mode: &mode,
            popup_snapshot: Some(&snapshot),
            popup_selected_row_id: Some("target:todo"),
            scripts: &[],
            scriptlets: &[],
            advanced_query_results_empty: false,
            menu_syntax_ai_proposal: None,
        })
        .expect("hint");

        assert_eq!(hint.title, "Todo inbox");
        assert!(!hint.examples.is_empty());
        assert!(hint.examples.iter().all(|e| e.starts_with(";todo")));
    }

    #[test]
    fn semicolon_picker_companion_describes_selected_target() {
        let mode = MenuSyntaxMode::from_input(";");
        let snapshot = build_trigger_picker_snapshot("+", &TriggerPickerContext::default())
            .expect("plus snapshot");
        let hint = build_menu_syntax_main_hint(MenuSyntaxMainHintContext {
            raw_filter_text: "+",
            mode: &mode,
            popup_snapshot: Some(&snapshot),
            popup_selected_row_id: Some("target:todo"),
            scripts: &[],
            scriptlets: &[],
            advanced_query_results_empty: false,
            menu_syntax_ai_proposal: None,
        })
        .expect("hint");

        assert_eq!(hint.kind, MenuSyntaxMainHintKind::CapturePickerCompanion);
        assert_eq!(hint.title, "Todo inbox");
        assert!(hint
            .primary_hint
            .as_deref()
            .unwrap()
            .contains("accept ;todo"));
    }

    #[test]
    fn capture_composer_previews_payload() {
        let raw = ";todo Renew passport #errands p1 due:tomorrow";
        let mode = MenuSyntaxMode::from_input(raw);
        let hint = build_menu_syntax_main_hint(MenuSyntaxMainHintContext {
            raw_filter_text: raw,
            mode: &mode,
            popup_snapshot: None,
            popup_selected_row_id: None,
            scripts: &[],
            scriptlets: &[],
            advanced_query_results_empty: false,
            menu_syntax_ai_proposal: None,
        })
        .expect("hint");

        assert_eq!(hint.kind, MenuSyntaxMainHintKind::CaptureComposer);
        assert_eq!(hint.title, "Capture todo");
        assert!(hint
            .rows
            .iter()
            .any(|row| row.label == "Body" && row.value == "Renew passport"));
        assert!(hint
            .rows
            .iter()
            .any(|row| row.label == "Tags" && row.value == "#errands"));
        assert!(hint
            .rows
            .iter()
            .any(|row| row.label == "Priority" && row.value == "P1"));
    }

    #[test]
    fn capture_composer_explains_tags_as_labels() {
        let raw = ";todo Buy milk";
        let mode = MenuSyntaxMode::from_input(raw);
        let hint = build_menu_syntax_main_hint(MenuSyntaxMainHintContext {
            raw_filter_text: raw,
            mode: &mode,
            popup_snapshot: None,
            popup_selected_row_id: None,
            scripts: &[],
            scriptlets: &[],
            advanced_query_results_empty: false,
            menu_syntax_ai_proposal: None,
        })
        .expect("hint");

        assert_eq!(hint.kind, MenuSyntaxMainHintKind::CaptureComposer);
        assert!(hint
            .rows
            .iter()
            .any(|row| row.label == "Tags" && row.value.contains("#errands")));
        assert!(hint
            .secondary_hint
            .as_deref()
            .unwrap()
            .contains("Tags group the saved item"));
        assert!(hint
            .examples
            .iter()
            .any(|example| example.contains("#errands")));
    }

    #[test]
    fn unregistered_semicolon_head_gets_no_hint() {
        let raw = ";github issue #bug";
        let mode = MenuSyntaxMode::from_input(raw);
        assert!(build_menu_syntax_main_hint(MenuSyntaxMainHintContext {
            raw_filter_text: raw,
            mode: &mode,
            popup_snapshot: None,
            popup_selected_row_id: None,
            scripts: &[],
            scriptlets: &[],
            advanced_query_results_empty: false,
            menu_syntax_ai_proposal: None,
        })
        .is_none());
    }

    #[test]
    fn registered_semicolon_head_gets_capture_hint() {
        let raw = ";github issue #bug";
        let targets = vec!["github".to_string()];
        let mode = MenuSyntaxMode::from_input_with_capture_targets(raw, &targets);
        let hint = build_menu_syntax_main_hint(MenuSyntaxMainHintContext {
            raw_filter_text: raw,
            mode: &mode,
            popup_snapshot: None,
            popup_selected_row_id: None,
            scripts: &[],
            scriptlets: &[],
            advanced_query_results_empty: false,
            menu_syntax_ai_proposal: None,
        })
        .expect("registered target hint");

        assert_eq!(hint.kind, MenuSyntaxMainHintKind::CaptureComposer);
        assert_eq!(hint.title, "Capture github");
    }

    #[test]
    fn command_composer_previews_fields_tags_and_argv() {
        let raw = ">ps-env env:dev project:launcher #demo -- --dry-run alpha";
        let mode = MenuSyntaxMode::from_input(raw);
        let scripts = vec![script("Power Syntax Env", Some("ps-env"))];
        let hint = build_menu_syntax_main_hint(MenuSyntaxMainHintContext {
            raw_filter_text: raw,
            mode: &mode,
            popup_snapshot: None,
            popup_selected_row_id: None,
            scripts: &scripts,
            scriptlets: &[],
            advanced_query_results_empty: false,
            menu_syntax_ai_proposal: None,
        })
        .expect("hint");

        assert_eq!(hint.kind, MenuSyntaxMainHintKind::CommandComposer);
        assert_eq!(hint.title, "Run ps-env");
        assert!(hint
            .rows
            .iter()
            .any(|row| row.label == "Fields" && row.value.contains("env=dev")));
        assert!(hint
            .rows
            .iter()
            .any(|row| row.label == "Tags" && row.value == "#demo"));
        assert!(hint
            .rows
            .iter()
            .any(|row| row.label == "Argv" && row.value.contains("--dry-run")));
    }

    #[test]
    fn unknown_command_warns_without_shell_semantics() {
        let raw = "!important";
        let mode = MenuSyntaxMode::from_input(raw);
        let hint = build_menu_syntax_main_hint(MenuSyntaxMainHintContext {
            raw_filter_text: raw,
            mode: &mode,
            popup_snapshot: None,
            popup_selected_row_id: None,
            scripts: &[],
            scriptlets: &[],
            advanced_query_results_empty: false,
            menu_syntax_ai_proposal: None,
        })
        .expect("hint");

        assert_eq!(hint.kind, MenuSyntaxMainHintKind::CommandComposer);
        assert!(hint.title.contains("No registered command"));
        assert!(hint
            .primary_hint
            .as_deref()
            .unwrap()
            .contains("not run a shell"));
    }

    #[test]
    fn duplicate_command_warns() {
        let raw = "!ps-dupe";
        let mode = MenuSyntaxMode::from_input(raw);
        let scripts = vec![script("Duplicate Script", Some("ps-dupe"))];
        let scriptlets = vec![scriptlet("Duplicate Scriptlet", Some("ps-dupe"))];
        let hint = build_menu_syntax_main_hint(MenuSyntaxMainHintContext {
            raw_filter_text: raw,
            mode: &mode,
            popup_snapshot: None,
            popup_selected_row_id: None,
            scripts: &scripts,
            scriptlets: &scriptlets,
            advanced_query_results_empty: false,
            menu_syntax_ai_proposal: None,
        })
        .expect("hint");

        assert_eq!(hint.kind, MenuSyntaxMainHintKind::CommandComposer);
        assert_eq!(hint.title, "Ambiguous command");
        assert!(hint.warning.as_deref().unwrap().contains("2 registered"));
    }

    #[test]
    fn bare_colon_main_hint_explains_refine() {
        let raw = ":";
        let mode = MenuSyntaxMode::from_input(raw);
        let snapshot =
            build_trigger_picker_snapshot(raw, &TriggerPickerContext::default()).expect("snapshot");
        let hint = build_menu_syntax_main_hint(MenuSyntaxMainHintContext {
            raw_filter_text: raw,
            mode: &mode,
            popup_snapshot: Some(&snapshot),
            popup_selected_row_id: None,
            scripts: &[],
            scriptlets: &[],
            advanced_query_results_empty: false,
            menu_syntax_ai_proposal: None,
        })
        .expect("hint");

        assert_eq!(hint.kind, MenuSyntaxMainHintKind::AdvancedQueryGuide);
        assert_eq!(hint.title, "Refine launcher search");
        assert!(hint.subtitle.as_deref().unwrap().contains("add filters"));
        assert!(hint
            .examples
            .iter()
            .any(|example| example == ":#work type:script"));
    }

    #[test]
    fn colon_hash_main_hint_explains_tag_filter_boundary() {
        let raw = ":#";
        let mode = MenuSyntaxMode::from_input(raw);
        let snapshot =
            build_trigger_picker_snapshot(raw, &TriggerPickerContext::default()).expect("snapshot");
        let hint = build_menu_syntax_main_hint(MenuSyntaxMainHintContext {
            raw_filter_text: raw,
            mode: &mode,
            popup_snapshot: Some(&snapshot),
            popup_selected_row_id: Some("qualifier:#"),
            scripts: &[],
            scriptlets: &[],
            advanced_query_results_empty: false,
            menu_syntax_ai_proposal: None,
        })
        .expect("hint");

        assert_eq!(hint.kind, MenuSyntaxMainHintKind::AdvancedQueryGuide);
        assert_eq!(hint.title, "Filter by tag");
        assert!(hint
            .rows
            .iter()
            .any(|row| row.label == "#work" && row.value.contains("Plain")));
        assert!(hint
            .rows
            .iter()
            .any(|row| row.label == ":#work" && row.value.contains("Filter")));
        assert!(hint
            .rows
            .iter()
            .any(|row| row.label == ";... #work" && row.value.contains("Label")));
    }

    #[test]
    fn advanced_query_empty_summarizes_predicates() {
        let raw = ":#work type:script nohit";
        let mode = MenuSyntaxMode::from_input(raw);
        let hint = build_menu_syntax_main_hint(MenuSyntaxMainHintContext {
            raw_filter_text: raw,
            mode: &mode,
            popup_snapshot: None,
            popup_selected_row_id: None,
            scripts: &[],
            scriptlets: &[],
            advanced_query_results_empty: true,
            menu_syntax_ai_proposal: None,
        })
        .expect("hint");

        assert_eq!(hint.kind, MenuSyntaxMainHintKind::AdvancedQueryEmpty);
        assert_eq!(hint.title, "No launcher items tagged #work");
        assert!(hint.rows.iter().any(|row| {
            row.label == "Filters"
                && row.value.contains("#work")
                && row.value.contains("scripts only")
        }));
        assert!(hint
            .rows
            .iter()
            .any(|row| row.label == "Search words" && row.value == "nohit"));
        assert!(hint
            .secondary_hint
            .as_deref()
            .unwrap()
            .contains("Plain `#work` is normal launcher search"));
    }

    #[test]
    fn plain_top_level_tag_gets_no_hint() {
        let raw = "#work";
        let mode = MenuSyntaxMode::from_input(raw);
        assert!(build_menu_syntax_main_hint(MenuSyntaxMainHintContext {
            raw_filter_text: raw,
            mode: &mode,
            popup_snapshot: None,
            popup_selected_row_id: None,
            scripts: &[],
            scriptlets: &[],
            advanced_query_results_empty: true,
            menu_syntax_ai_proposal: None,
        })
        .is_none());
    }

    #[test]
    fn command_composer_renders_schema_rows_for_registered_head() {
        // sdk-command-schema: a script that registers a `command.v1`
        // handler with `head: deploy`, args `[env]`, flags `[--dry-run]`
        // makes `setFilter ">deploy"` getState surface "env" and
        // "--dry-run" as labels in `menuSyntaxMainHint.rows`.
        use crate::metadata_parser::TypedMetadata;
        use serde_json::json;
        use std::collections::HashMap;

        let mut extra: HashMap<String, serde_json::Value> = HashMap::new();
        extra.insert(
            "menuSyntax".to_string(),
            json!([{
                "family": "command.v1",
                "head": "deploy",
                "label": "Deploy a service",
                "args": [
                    {"name": "env", "required": true,
                     "values": ["prod", "staging", "dev"]}
                ],
                "flags": [
                    {"name": "--dry-run", "alias": "-n",
                     "description": "Print the plan without applying"}
                ],
                "usage": ">deploy -- <env> [--dry-run]"
            }]),
        );
        let typed = TypedMetadata {
            extra,
            ..Default::default()
        };
        let s = Arc::new(Script {
            name: "Deploy".to_string(),
            alias: None,
            path: PathBuf::from("/tmp/deploy.ts"),
            extension: "ts".to_string(),
            typed_metadata: Some(typed),
            ..Default::default()
        });

        let raw = ">deploy";
        let mode = MenuSyntaxMode::from_input(raw);
        let hint = build_menu_syntax_main_hint(MenuSyntaxMainHintContext {
            raw_filter_text: raw,
            mode: &mode,
            popup_snapshot: None,
            popup_selected_row_id: None,
            scripts: std::slice::from_ref(&s),
            scriptlets: &[],
            advanced_query_results_empty: false,
            menu_syntax_ai_proposal: None,
        })
        .expect("hint");

        assert_eq!(hint.kind, MenuSyntaxMainHintKind::CommandComposer);
        let labels: Vec<&str> = hint.rows.iter().map(|r| r.label.as_str()).collect();
        assert!(
            labels.contains(&"env"),
            "expected `env` arg row, got rows: {labels:?}"
        );
        assert!(
            labels.contains(&"--dry-run"),
            "expected `--dry-run` flag row, got rows: {labels:?}"
        );
        // The arg's `required: true` becomes a "required" chip on the env row.
        let env_row = hint
            .rows
            .iter()
            .find(|r| r.label == "env")
            .expect("env row");
        assert!(
            env_row.chips.iter().any(|c| c.label == "required"),
            "expected `required` chip on env row, got: {:?}",
            env_row.chips
        );
        // The arg's `values` list becomes the row value text so authors see
        // accepted choices in the hint card.
        assert_eq!(env_row.value, "prod | staging | dev");
        let dry_row = hint
            .rows
            .iter()
            .find(|r| r.label == "--dry-run")
            .expect("--dry-run row");
        assert!(
            dry_row.value.contains("Print the plan"),
            "expected description in flag value, got: {}",
            dry_row.value
        );
    }

    #[test]
    fn command_composer_without_schema_omits_schema_rows() {
        // Negative pin: command_composer_hint must not invent schema rows
        // when no script registers a matching command.v1 handler. This pins
        // the `script_command_schema_for` dependency — a regression that
        // returned a stub spec by default would surface ghost rows.
        let raw = "!unknown";
        let mode = MenuSyntaxMode::from_input(raw);
        let hint = build_menu_syntax_main_hint(MenuSyntaxMainHintContext {
            raw_filter_text: raw,
            mode: &mode,
            popup_snapshot: None,
            popup_selected_row_id: None,
            scripts: &[],
            scriptlets: &[],
            advanced_query_results_empty: false,
            menu_syntax_ai_proposal: None,
        })
        .expect("hint");

        assert_eq!(hint.kind, MenuSyntaxMainHintKind::CommandComposer);
        // The default `Command !unknown` row remains, but no `env` /
        // `--dry-run` schema rows should exist.
        let labels: Vec<&str> = hint.rows.iter().map(|r| r.label.as_str()).collect();
        assert!(
            !labels.contains(&"env") && !labels.contains(&"--dry-run"),
            "schema rows leaked through without a registered handler: {labels:?}"
        );
    }

    #[test]
    fn predicate_label_handles_negation() {
        let query = parse_advanced_query(":-type:app has:menuSyntax");
        let labels = query
            .predicates
            .iter()
            .map(predicate_label)
            .collect::<Vec<_>>();
        assert_eq!(labels, vec!["-type:app", "has:menuSyntax"]);
    }

    // ========================================================================
    // capture_validation_chips_and_snapshot (Pass 22)
    // ========================================================================

    #[test]
    fn capture_validation_cal_with_no_invocation_yields_two_needs_chips() {
        // Receipt from story: setFilter ";cal" → statusChips: [
        //   {"; capture"}, {"needs body"}, {"needs date"}
        // ] and captureValidation.status = incomplete.
        let (chips, validation) =
            capture_validation_chips_and_snapshot("cal", None, None, &[], &[]);
        let labels: Vec<&str> = chips.iter().map(|c| c.label.as_str()).collect();
        assert_eq!(labels, vec!["; capture", "needs body", "needs date"]);
        let v = validation.expect("cal has a builtin schema");
        assert_eq!(v.status, MenuSyntaxCaptureValidationStatus::Incomplete);
        assert!(!v.can_submit);
        assert_eq!(v.target, "cal");
        assert_eq!(
            v.missing_field_labels,
            vec!["body".to_string(), "date".to_string()]
        );
    }

    #[test]
    fn capture_validation_cal_with_body_and_date_yields_ready() {
        let mut inv = CaptureInvocation {
            target: "cal".to_string(),
            alias_form: CaptureAlias::CapturePrefix,
            body: "Design review".to_string(),
            tags: vec![],
            priority: None,
            url: None,
            duration: None,
            kv: vec![],
            date_phrases: vec![],
            raw: ";cal Design review start:friday".to_string(),
        };
        inv.date_phrases
            .push(crate::menu_syntax::payload::DatePhrase {
                role: DateRole::Start,
                source: "friday".to_string(),
                source_span: (0, 6),
            });
        let (chips, validation) =
            capture_validation_chips_and_snapshot("cal", Some(&inv), None, &[], &[]);
        let labels: Vec<&str> = chips.iter().map(|c| c.label.as_str()).collect();
        assert_eq!(labels, vec!["; capture", "ready"]);
        let v = validation.unwrap();
        assert_eq!(v.status, MenuSyntaxCaptureValidationStatus::Ready);
        assert!(v.can_submit);
        assert!(v.missing_field_labels.is_empty());
    }

    #[test]
    fn capture_validation_unknown_target_returns_only_mode_chip_no_snapshot() {
        let (chips, validation) =
            capture_validation_chips_and_snapshot("github", None, None, &[], &[]);
        let labels: Vec<&str> = chips.iter().map(|c| c.label.as_str()).collect();
        assert_eq!(labels, vec!["; capture"]);
        assert!(
            validation.is_none(),
            "no builtin schema for github → no snapshot; doctor flags this elsewhere"
        );
    }

    #[test]
    fn capture_validation_link_with_bad_url_yields_malformed() {
        let inv = CaptureInvocation {
            target: "link".to_string(),
            alias_form: CaptureAlias::CapturePrefix,
            body: String::new(),
            tags: vec![],
            priority: None,
            url: Some("ftp://nope".to_string()),
            duration: None,
            kv: vec![],
            date_phrases: vec![],
            raw: ";link ftp://nope".to_string(),
        };
        let (chips, validation) =
            capture_validation_chips_and_snapshot("link", Some(&inv), None, &[], &[]);
        let labels: Vec<&str> = chips.iter().map(|c| c.label.as_str()).collect();
        assert_eq!(labels, vec!["; capture", "malformed"]);
        let v = validation.unwrap();
        assert_eq!(v.status, MenuSyntaxCaptureValidationStatus::Malformed);
        assert!(!v.can_submit);
        assert_eq!(v.malformed_field_label.as_deref(), Some("url"));
        assert!(v.malformed_reason.as_deref().unwrap().contains("http"));
    }

    #[test]
    fn capture_validation_uses_resolved_nl_state_for_mcal() {
        let scripts = vec![mcal_script()];
        let hint = capture_hint_for(";mcal Lunch with Ryan tomorrow at 12pm til 1pm", &scripts);
        let labels: Vec<&str> = hint
            .status_chips
            .iter()
            .map(|chip| chip.label.as_str())
            .collect();
        assert_eq!(labels, vec!["; capture", "ready"]);
        assert!(!labels.contains(&"needs date"));
        let validation = hint.capture_validation.expect("validation");
        assert_eq!(validation.status, MenuSyntaxCaptureValidationStatus::Ready);
        assert!(validation.can_submit);
        assert!(validation.missing_field_labels.is_empty());
    }

    #[test]
    fn capture_validation_mcal_date_only_needs_body_not_date() {
        let scripts = vec![mcal_script()];
        let hint = capture_hint_for(";mcal tomorrow at 12pm til 1pm", &scripts);
        let labels: Vec<&str> = hint
            .status_chips
            .iter()
            .map(|chip| chip.label.as_str())
            .collect();
        assert_eq!(labels, vec!["; capture", "needs body"]);
        assert!(!labels.contains(&"needs date"));
        let validation = hint.capture_validation.expect("validation");
        assert_eq!(validation.missing_field_labels, vec!["body".to_string()]);
    }

    #[test]
    fn capture_validation_snapshot_serializes_to_camel_case() {
        let snapshot = MenuSyntaxCaptureValidationSnapshot {
            target: "cal".to_string(),
            status: MenuSyntaxCaptureValidationStatus::Incomplete,
            can_submit: false,
            missing_field_labels: vec!["body".to_string(), "date".to_string()],
            malformed_field_label: None,
            malformed_reason: None,
            hud_message: Some(";cal needs body and date".to_string()),
        };
        let json = serde_json::to_string(&snapshot).unwrap();
        assert!(json.contains("\"canSubmit\":false"), "got {json}");
        assert!(json.contains("\"missingFieldLabels\""), "got {json}");
        assert!(json.contains("\"status\":\"incomplete\""), "got {json}");
        // Empty optional fields are skipped
        assert!(!json.contains("malformedFieldLabel"), "got {json}");
    }

    // -------- target_examples (Run 12 Pass 2 — hint-examples-target-relevant) --------

    #[test]
    fn target_examples_for_cal_all_start_with_semicolon_cal() {
        let examples = target_examples("cal");
        assert!(!examples.is_empty(), ";cal must have ≥1 example");
        for ex in &examples {
            assert!(
                ex.starts_with(";cal "),
                "all ;cal examples must start with `;cal `, got: {ex}"
            );
        }
    }

    #[test]
    fn target_examples_for_cal_have_no_todo_leakage() {
        // Falsifier: this is the exact bug the user reported in screenshot
        // /Users/johnlindquist/screenshots/CleanShot 2026-04-25 at 09.27.22@2x.png
        // — `;cal` previously showed a `;todo Send proposal …` example mixed
        // in. After this story ships, a `;cal` hint must NEVER contain `;todo`.
        let examples = target_examples("cal");
        for ex in &examples {
            assert!(
                !ex.contains(";todo"),
                "`;cal` example MUST NOT contain `;todo`, got: {ex}"
            );
        }
    }

    #[test]
    fn target_examples_for_cal_include_a_date_slot() {
        // ;cal requires a date — the example should double as a fix-it
        // template, so at least one example must show a date key.
        let examples = target_examples("cal");
        let has_date = examples.iter().any(|ex| {
            ex.contains("start:")
                || ex.contains("at:")
                || ex.contains("due:")
                || ex.contains("end:")
        });
        assert!(
            has_date,
            ";cal examples must include at least one date slot (start:/at:/due:/end:), got: {examples:?}"
        );
    }

    #[test]
    fn target_examples_for_todo_all_start_with_semicolon_todo() {
        let examples = target_examples("todo");
        assert!(!examples.is_empty());
        for ex in &examples {
            assert!(ex.starts_with(";todo "), "got: {ex}");
        }
    }

    #[test]
    fn target_examples_for_unknown_target_falls_back_with_correct_verb() {
        // Custom user-defined targets get the generic example list, but each
        // example MUST still start with the user's actual verb — no `;todo`
        // leakage even on the fallback path.
        let examples = target_examples("custom");
        assert!(!examples.is_empty());
        for ex in &examples {
            assert!(
                ex.starts_with(";custom "),
                "fallback example must use the actual target verb, got: {ex}"
            );
            assert!(
                !ex.contains(";todo"),
                "fallback must not leak ;todo, got: {ex}"
            );
        }
    }

    #[test]
    fn target_examples_for_shipped_dynamic_targets_match_their_handlers() {
        let cases = [
            ("github", ["johnlindquist/kit", "repo=", "url:"]),
            ("expense", ["amount=", "vendor=", "reimbursable="]),
            ("snippet", ["lang=", "title=", "url:"]),
            ("fixture", ["env=", "kind=", "state="]),
            ("gcal", ["calendarId=", "start:", "guests="]),
            ("mcal", ["calendar=", "alarm=", "start:"]),
            ("reminder", ["tomorrow", "every day", "next month"]),
            ("snooze", ["in 30 minutes", "tomorrow", "next monday"]),
            ("defer", ["until next week", "friday", "in 2 days"]),
        ];

        for (target, expected_fragments) in cases {
            let examples = target_examples(target);
            assert_eq!(examples.len(), 3, "{target} should ship three examples");
            for example in &examples {
                assert!(
                    example.starts_with(&format!(";{target} ")),
                    "{target} example must use its own target, got: {example}"
                );
                assert!(
                    !example.contains("Buy milk") && !example.contains("Send proposal"),
                    "{target} example leaked generic todo copy: {example}"
                );
            }
            for fragment in expected_fragments {
                assert!(
                    examples.iter().any(|example| example.contains(fragment)),
                    "{target} examples should include `{fragment}`, got: {examples:?}"
                );
            }
        }
    }

    #[test]
    fn main_hint_snapshot_omits_fragment_preview_when_none() {
        let snapshot = MenuSyntaxMainHintSnapshot {
            kind: MenuSyntaxMainHintKind::CaptureComposer,
            raw_filter_text: ";mcal Lunch".to_string(),
            title: "Capture mcal".to_string(),
            subtitle: None,
            mode_chip: None,
            status_chip: None,
            status_chips: Vec::new(),
            capture_validation: None,
            unresolved_dates: Vec::new(),
            menu_syntax_ai_proposal: None,
            rows: Vec::new(),
            fragment_preview: None,
            primary_hint: None,
            secondary_hint: None,
            example: None,
            examples: Vec::new(),
            warning: None,
            accessibility_label: String::new(),
        };
        let json = serde_json::to_string(&snapshot).unwrap();
        assert!(!json.contains("fragmentPreview"), "{json}");
    }

    #[test]
    fn fragment_preview_snapshot_serializes_camel_case() {
        let preview = MenuSyntaxFragmentPreviewSnapshot {
            rows: vec![MenuSyntaxFragmentPreviewRow {
                role: crate::menu_syntax::fragments::MenuSyntaxFragmentRole::DateRange,
                label: "When".to_string(),
                value: "tomorrow 12-1".to_string(),
                source: "tomorrow 12pm til 1pm".to_string(),
                source_span: (5, 27),
                status: crate::menu_syntax::fragments::MenuSyntaxFragmentStatus::Resolved,
                tone: MenuSyntaxMainHintTone::Info,
                chips: vec![MenuSyntaxMainHintChip {
                    label: "range".to_string(),
                    tone: MenuSyntaxMainHintTone::Accent,
                }],
            }],
        };
        let json = serde_json::to_string(&preview).unwrap();
        assert!(json.contains("\"sourceSpan\":[5,27]"), "{json}");
        assert!(json.contains("\"dateRange\""), "{json}");
        assert!(json.contains("\"tone\":\"Info\""), "{json}");
    }

    #[test]
    fn capture_composer_fragment_preview_for_mcal_range() {
        let scripts = vec![mcal_script()];
        let hint = capture_hint_for(";mcal Lunch with Ryan tomorrow at 12pm til 1pm", &scripts);
        let preview = hint.fragment_preview.expect("fragment preview");
        assert!(preview.rows.iter().any(
            |row| row.role == MenuSyntaxFragmentRole::Subject && row.value == "Lunch with Ryan"
        ));
        assert!(preview.rows.iter().any(|row| {
            row.role == MenuSyntaxFragmentRole::DateRange
                && row.label == "Date range"
                && row.value.contains("resolved")
        }));
        let range = preview
            .rows
            .iter()
            .find(|row| row.role == MenuSyntaxFragmentRole::DateRange)
            .expect("range row");
        assert_eq!(range.source, "tomorrow at 12pm til 1pm");
        assert_eq!(range.source_span, (22, 46));
    }

    #[test]
    fn capture_composer_fragment_preview_for_mcal_duration() {
        let scripts = vec![mcal_script()];
        let hint = capture_hint_for(";mcal Lunch with Ryan tom 12pm for 30mins", &scripts);
        let preview = hint.fragment_preview.expect("fragment preview");
        assert!(preview.rows.iter().any(|row| {
            row.role == MenuSyntaxFragmentRole::Duration && row.value.contains("30 minutes")
        }));
    }

    #[test]
    fn capture_composer_fragment_preview_for_mcal_recurrence() {
        let scripts = vec![mcal_script()];
        let hint = capture_hint_for(";mcal Lunch w/ Ryan every mon from 1 til 2", &scripts);
        let preview = hint.fragment_preview.expect("fragment preview");
        assert!(preview.rows.iter().any(|row| {
            row.role == MenuSyntaxFragmentRole::Recurrence
                && row.value.contains("FREQ=WEEKLY;BYDAY=MO")
        }));
    }

    #[test]
    fn capture_composer_fragment_preview_marks_unresolved_muted() {
        let scripts = vec![mcal_script()];
        let hint = capture_hint_for(";mcal Lunch start:asdf", &scripts);
        let preview = hint.fragment_preview.expect("fragment preview");
        assert!(preview.rows.iter().any(|row| {
            row.role == MenuSyntaxFragmentRole::Unresolved
                && row.status == MenuSyntaxFragmentStatus::Unresolved
                && row.tone == MenuSyntaxMainHintTone::Muted
        }));
    }

    #[test]
    fn main_hint_snapshot_omits_fragment_preview_when_capture_empty() {
        let scripts = vec![mcal_script()];
        let targets = crate::menu_syntax::registered_capture_targets_from_scripts(&scripts);
        let raw = ";mcal ";
        let mode = MenuSyntaxMode::from_input_with_capture_targets(raw, &targets);
        let hint = build_menu_syntax_main_hint(MenuSyntaxMainHintContext {
            raw_filter_text: raw,
            mode: &mode,
            popup_snapshot: None,
            popup_selected_row_id: None,
            scripts: &scripts,
            scriptlets: &[],
            advanced_query_results_empty: false,
            menu_syntax_ai_proposal: None,
        })
        .expect("hint");
        let json = serde_json::to_string(&hint).unwrap();
        assert!(!json.contains("fragmentPreview"), "{json}");
    }

    #[test]
    fn existing_non_capture_hint_json_unchanged_with_fragment_preview_field() {
        let raw = ":type:script nope";
        let mode = MenuSyntaxMode::from_input(raw);
        let hint = build_menu_syntax_main_hint(MenuSyntaxMainHintContext {
            raw_filter_text: raw,
            mode: &mode,
            popup_snapshot: None,
            popup_selected_row_id: None,
            scripts: &[],
            scriptlets: &[],
            advanced_query_results_empty: true,
            menu_syntax_ai_proposal: None,
        })
        .expect("hint");
        let json = serde_json::to_string(&hint).unwrap();
        assert!(!json.contains("fragmentPreview"), "{json}");
    }
}
