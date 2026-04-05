use std::{
    fs,
    path::{Path, PathBuf},
};

use anyhow::{Context, Result};

const REPORT_SLUG: &str = "prompt-chrome-consistency";
const REPORT_TITLE: &str = "Prompt Chrome Consistency Audit";
const REPORT_SCOPE_EXCLUSIONS: &[&str] =
    &["ACP compact-chat popup surfaces (for example src/ai/acp/model_selector_popup.rs)"];

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum AuditSeverity {
    Info,
    Warning,
    Error,
}

impl AuditSeverity {
    fn as_str(&self) -> &'static str {
        match self {
            Self::Info => "info",
            Self::Warning => "warning",
            Self::Error => "error",
        }
    }

    fn sort_rank(&self) -> u8 {
        match self {
            Self::Error => 0,
            Self::Warning => 1,
            Self::Info => 2,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AuditFinding {
    pub severity: AuditSeverity,
    pub title: &'static str,
    pub summary: String,
    pub evidence: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AuditSurfaceResult {
    pub surface: &'static str,
    pub files: Vec<&'static str>,
    pub findings: Vec<AuditFinding>,
}

impl AuditSurfaceResult {
    pub fn status(&self) -> &'static str {
        if self
            .findings
            .iter()
            .any(|finding| matches!(finding.severity, AuditSeverity::Error))
        {
            "error"
        } else if self
            .findings
            .iter()
            .any(|finding| matches!(finding.severity, AuditSeverity::Warning))
        {
            "warning"
        } else {
            "pass"
        }
    }

    pub fn has_only_info_findings(&self) -> bool {
        !self.findings.is_empty()
            && self
                .findings
                .iter()
                .all(|finding| matches!(finding.severity, AuditSeverity::Info))
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AuditReport {
    pub slug: &'static str,
    pub title: &'static str,
    pub summary: String,
    pub surfaces: Vec<AuditSurfaceResult>,
}

#[derive(Clone, Copy, Debug)]
struct SurfaceSpec {
    surface: &'static str,
    files: &'static [&'static str],
    accepted_hint_surfaces: &'static [&'static str],
}

const SURFACES: &[SurfaceSpec] = &[
    SurfaceSpec {
        surface: "render_prompts::select",
        files: &[
            "src/render_prompts/other.rs",
            "src/prompts/select/render.rs",
        ],
        accepted_hint_surfaces: &["render_prompts::select"],
    },
    SurfaceSpec {
        surface: "render_prompts::arg",
        files: &["src/render_prompts/arg/render.rs"],
        accepted_hint_surfaces: &["render_prompts::arg"],
    },
    SurfaceSpec {
        surface: "render_prompts::form",
        files: &["src/render_prompts/form/render.rs"],
        accepted_hint_surfaces: &["render_prompts::form"],
    },
    SurfaceSpec {
        surface: "render_prompts::chat",
        files: &[
            "src/render_prompts/other.rs",
            "src/prompts/chat/render_core.rs",
        ],
        accepted_hint_surfaces: &["prompts::chat", "prompts::chat::mini"],
    },
    SurfaceSpec {
        surface: "render_prompts::term",
        files: &["src/render_prompts/term.rs"],
        accepted_hint_surfaces: &["render_prompts::term"],
    },
    SurfaceSpec {
        surface: "prompts::path",
        files: &["src/prompts/path/render.rs"],
        accepted_hint_surfaces: &["prompts::path"],
    },
    SurfaceSpec {
        surface: "clipboard_history",
        files: &[
            "src/render_builtins/clipboard.rs",
            "src/render_builtins/clipboard_history_layout.rs",
        ],
        accepted_hint_surfaces: &["clipboard_history"],
    },
    SurfaceSpec {
        surface: "file_search",
        files: &["src/render_builtins/file_search.rs"],
        accepted_hint_surfaces: &["file_search"],
    },
];

fn contains_any(haystack: &str, needles: &[&str]) -> bool {
    needles.iter().any(|needle| haystack.contains(needle))
}

fn has_file_search_skeleton_loading(haystack: &str) -> bool {
    contains_any(
        haystack,
        &["show static skeleton rows", "Render 6 skeleton rows"],
    )
}

fn has_text_only_file_search_loading(haystack: &str) -> bool {
    haystack.contains("is_loading && filtered_len == 0")
        && haystack.contains(".justify_center()")
        && haystack.contains(".child(\"Searching...\")")
}

const FILE_SEARCH_LIVE_FILE: &str = "src/render_builtins/file_search.rs";
const FILE_SEARCH_STALE_LAYOUT_FILE: &str = "src/render_builtins/file_search_layout.rs";

fn read_optional_source(repo_root: &Path, file: &str) -> Option<String> {
    fs::read_to_string(repo_root.join(file)).ok()
}

fn has_file_search_duplicate_layout_markers(source: &str) -> bool {
    contains_any(
        source,
        &[
            "render_minimal_list_prompt_scaffold(",
            "render_expanded_view_scaffold(",
            "render_expanded_view_scaffold_with_hints(",
            "emit_prompt_hint_audit(\"file_search\"",
            "file_search_chrome_checkpoint",
        ],
    )
}

fn check_file_search_duplicate_layout(repo_root: &Path) -> bool {
    read_optional_source(repo_root, FILE_SEARCH_STALE_LAYOUT_FILE)
        .map(|source| has_file_search_duplicate_layout_markers(&source))
        .unwrap_or(false)
}

fn info(title: &'static str, summary: impl Into<String>, evidence: Vec<String>) -> AuditFinding {
    AuditFinding {
        severity: AuditSeverity::Info,
        title,
        summary: summary.into(),
        evidence,
    }
}

fn warning(title: &'static str, summary: impl Into<String>, evidence: Vec<String>) -> AuditFinding {
    AuditFinding {
        severity: AuditSeverity::Warning,
        title,
        summary: summary.into(),
        evidence,
    }
}

fn read_source_files(
    repo_root: &Path,
    files: &[&'static str],
) -> Result<Vec<(&'static str, String)>> {
    let mut loaded = Vec::with_capacity(files.len());
    for file in files {
        let path = repo_root.join(file);
        let source = fs::read_to_string(&path)
            .with_context(|| format!("failed to read {}", path.display()))?;
        loaded.push((*file, source));
    }
    Ok(loaded)
}

fn documented_exception_surfaces<'a>(surfaces: &'a [AuditSurfaceResult]) -> Vec<&'a str> {
    surfaces
        .iter()
        .filter(|surface| surface.has_only_info_findings())
        .map(|surface| surface.surface)
        .collect()
}

fn audit_surface(spec: SurfaceSpec, repo_root: &Path) -> Result<AuditSurfaceResult> {
    let sources = read_source_files(repo_root, spec.files)?;
    let combined = sources
        .iter()
        .map(|(_, source)| source.as_str())
        .collect::<Vec<_>>()
        .join("\n");

    let mut findings = Vec::new();

    let has_runtime_chrome_audit = combined.contains("emit_prompt_chrome_audit(")
        && combined.contains(&format!("\"{}\"", spec.surface));

    if !has_runtime_chrome_audit {
        findings.push(warning(
            "missing runtime chrome audit",
            format!(
                "{} does not currently declare a runtime `emit_prompt_chrome_audit(...)` for its surface name.",
                spec.surface
            ),
            spec.files.iter().map(|file| file.to_string()).collect(),
        ));
    }

    let has_runtime_hint_audit = spec.accepted_hint_surfaces.iter().any(|surface_name| {
        combined.contains(&format!("emit_prompt_hint_audit(\"{}\"", surface_name))
    });

    let clearly_uses_universal_hints = contains_any(
        &combined,
        &[
            "universal_prompt_hints()",
            "clickable_universal_hint_strip(cx)",
            "render_universal_prompt_hint_strip()",
            "render_expanded_view_scaffold(",
        ],
    );

    let uses_expanded_scaffold = contains_any(
        &combined,
        &[
            "render_expanded_view_scaffold(",
            "render_expanded_view_scaffold_with_hints(",
            "render_expanded_view_prompt_shell(",
        ],
    );

    match spec.surface {
        "file_search" => {
            if has_file_search_skeleton_loading(&combined)
                && has_text_only_file_search_loading(&combined)
            {
                findings.push(warning(
                    "loading state mismatch",
                    "File Search defines skeleton loading rows in source, but the live layout still collapses loading to a centered `Searching...` label. Keep a single intentional loading state on the real runtime surface.",
                    vec![FILE_SEARCH_LIVE_FILE.to_string()],
                ));
            }

            // `↵ Open` / `↵ Browse` are accepted contextual primary-action labels
            // for file search.  Only flag the OLD non-three-key hints (`⌘↵ Ask AI`,
            // `⇥ Navigate`) that broke the canonical pattern.
            let has_legacy_non_canonical_hints = contains_any(
                &combined,
                &[
                    "\\u{2318}\\u{21b5} Ask AI",
                    "\u{2318}\u{21b5} Ask AI",
                    "\\u{21e5} Navigate",
                    "\u{21e5} Navigate",
                ],
            );

            if has_legacy_non_canonical_hints {
                findings.push(warning(
                    "non-universal footer hints",
                    "File Search mini mode still advertises `⌘↵ Ask AI` and/or `⇥ Navigate` instead of the canonical `⌘K Actions`, `Tab AI` slots.",
                    vec![FILE_SEARCH_LIVE_FILE.to_string()],
                ));
            }

            if has_legacy_non_canonical_hints && !has_runtime_hint_audit {
                findings.push(warning(
                    "missing prompt hint audit",
                    "File Search does not emit `emit_prompt_hint_audit(\"file_search\", ...)`, so its mini-mode footer drift bypasses the shared hint-contract warning path.",
                    vec![FILE_SEARCH_LIVE_FILE.to_string()],
                ));
            }

            // Contextual primary label (`↵ Open` / `↵ Browse`) paired with
            // `⌘K Actions` + `Tab AI` is an accepted three-key variant.
            let has_contextual_primary = contains_any(
                &combined,
                &[
                    "\\u{21b5} Open",
                    "\u{21b5} Open",
                    "\\u{21b5} Browse",
                    "\u{21b5} Browse",
                ],
            );
            let has_canonical_actions_and_ai = combined.contains("K Actions")
                && combined.contains("Tab AI");

            if has_contextual_primary && has_canonical_actions_and_ai && has_runtime_hint_audit {
                findings.push(info(
                    "contextual primary label follows three-key pattern",
                    "File Search uses `↵ Open` / `↵ Browse` as the primary action label instead of `↵ Run`, paired with canonical `⌘K Actions` and `Tab AI`. This is an accepted contextual variant of the three-key footer pattern.",
                    vec![FILE_SEARCH_LIVE_FILE.to_string()],
                ));
            }

            if check_file_search_duplicate_layout(repo_root) {
                findings.push(warning(
                    "duplicate file_search layout source",
                    format!(
                        "file_search chrome markers still exist in `{}` even though the live surface is audited from `{}`. Keep one source of truth or the report can pass on stale code.",
                        FILE_SEARCH_STALE_LAYOUT_FILE, FILE_SEARCH_LIVE_FILE
                    ),
                    vec![
                        FILE_SEARCH_LIVE_FILE.to_string(),
                        FILE_SEARCH_STALE_LAYOUT_FILE.to_string(),
                    ],
                ));
                tracing::warn!(
                    target: "script_kit::audit",
                    report_slug = REPORT_SLUG,
                    surface = "file_search",
                    duplicate_source = FILE_SEARCH_STALE_LAYOUT_FILE,
                    "file_search duplicate layout source detected"
                );
            }
        }

        "clipboard_history" => {
            if !uses_expanded_scaffold {
                findings.push(info(
                    "manual expanded layout",
                    "Clipboard History is not routing through a shared expanded-view scaffold or shell, which makes future chrome drift easier to reintroduce.",
                    vec![
                        "src/render_builtins/clipboard.rs".to_string(),
                        "src/render_builtins/clipboard_history_layout.rs".to_string(),
                    ],
                ));
            }

            if !has_runtime_hint_audit && !clearly_uses_universal_hints {
                findings.push(warning(
                    "missing prompt hint audit",
                    "Clipboard History should keep a discoverable hint-contract marker in source so the report can distinguish intentional from accidental footer changes.",
                    vec![
                        "src/render_builtins/clipboard.rs".to_string(),
                        "src/render_builtins/clipboard_history_layout.rs".to_string(),
                    ],
                ));
            }
        }

        "render_prompts::term" => {
            if contains_any(
                &combined,
                &[
                    "custom_hint_strip",
                    "render_terminal_prompt_hint_strip(",
                    "Bun verify required",
                    "\u{2318}\u{21b5} Apply",
                    "\u{2318}W Close",
                ],
            ) {
                findings.push(info(
                    "contextual footer exception",
                    "Term intentionally owns a contextual footer. Keep it documented as an exception in the report instead of forcing universal hints onto the terminal surface.",
                    vec!["src/render_prompts/term.rs".to_string()],
                ));
            }
        }

        _ => {
            if !has_runtime_hint_audit && !clearly_uses_universal_hints {
                findings.push(warning(
                    "missing prompt hint audit",
                    format!(
                        "{} does not clearly emit `emit_prompt_hint_audit(...)` and does not obviously route through the universal hint helpers.",
                        spec.surface
                    ),
                    spec.files.iter().map(|file| file.to_string()).collect(),
                ));
            }
        }
    }

    findings.sort_by_key(|finding| (finding.severity.sort_rank(), finding.title));

    let result = AuditSurfaceResult {
        surface: spec.surface,
        files: spec.files.to_vec(),
        findings,
    };

    tracing::info!(
        target: "script_kit::audit",
        surface = result.surface,
        status = result.status(),
        finding_count = result.findings.len(),
        "prompt chrome surface audited"
    );

    if !result.findings.is_empty() {
        tracing::warn!(
            target: "script_kit::audit",
            surface = result.surface,
            status = result.status(),
            finding_count = result.findings.len(),
            "prompt chrome surface drift detected"
        );
    }

    Ok(result)
}

pub fn build_prompt_chrome_consistency_report(repo_root: &Path) -> Result<AuditReport> {
    let mut surfaces = Vec::with_capacity(SURFACES.len());
    for spec in SURFACES {
        surfaces.push(audit_surface(*spec, repo_root)?);
    }

    let warning_count = surfaces
        .iter()
        .filter(|surface| surface.status() == "warning")
        .count();
    let error_count = surfaces
        .iter()
        .filter(|surface| surface.status() == "error")
        .count();
    let pass_count = surfaces.len() - warning_count - error_count;

    let exceptions = documented_exception_surfaces(&surfaces);

    let summary = if warning_count == 0 && error_count == 0 && exceptions.is_empty() {
        format!(
            "Scanned {} prompt/builtin surfaces. {} pass, {} warning, {} error. No current drift markers were detected.",
            surfaces.len(),
            pass_count,
            warning_count,
            error_count
        )
    } else if warning_count == 0 && error_count == 0 {
        format!(
            "Scanned {} prompt/builtin surfaces. {} pass, {} warning, {} error. {} intentional exception{} documented: {}.",
            surfaces.len(),
            pass_count,
            warning_count,
            error_count,
            exceptions.len(),
            if exceptions.len() == 1 { "" } else { "s" },
            exceptions.join(", ")
        )
    } else {
        let drift_surfaces = surfaces
            .iter()
            .filter(|surface| surface.status() == "warning" || surface.status() == "error")
            .map(|surface| surface.surface)
            .collect::<Vec<_>>()
            .join(", ");
        format!(
            "Scanned {} prompt/builtin surfaces. {} pass, {} warning, {} error. Highest-leverage current drifts: {}.",
            surfaces.len(),
            pass_count,
            warning_count,
            error_count,
            drift_surfaces
        )
    };

    tracing::info!(
        target: "script_kit::audit",
        slug = REPORT_SLUG,
        pass_count,
        warning_count,
        error_count,
        exception_count = exceptions.len(),
        exceptions = ?exceptions,
        scope_exclusions = ?REPORT_SCOPE_EXCLUSIONS,
        "prompt chrome audit summary built"
    );

    Ok(AuditReport {
        slug: REPORT_SLUG,
        title: REPORT_TITLE,
        summary,
        surfaces,
    })
}

pub fn render_prompt_chrome_consistency_markdown(report: &AuditReport) -> String {
    let exceptions = documented_exception_surfaces(&report.surfaces);

    let mut lines = vec![
        format!("# {}", report.title),
        String::new(),
        "## Summary".to_string(),
        report.summary.clone(),
        String::new(),
        "## Scope Notes".to_string(),
        format!(
            "- Scope: prompt and builtin chrome surfaces only. Excluded this pass: {}.",
            REPORT_SCOPE_EXCLUSIONS.join(", ")
        ),
        "- Verification precondition: keep only one visible target window per GPUI window kind when using `simulateGpuiEvent`; ambiguous same-kind routing now fails closed.".to_string(),
    ];

    if !exceptions.is_empty() {
        lines.push(format!(
            "- Intentional exception{}: {}.",
            if exceptions.len() == 1 { "" } else { "s" },
            exceptions.join(", ")
        ));
    }

    lines.push(String::new());
    lines.push("## Surface Status".to_string());
    lines.push("| Surface | Status | Files |".to_string());
    lines.push("| --- | --- | --- |".to_string());

    for surface in &report.surfaces {
        lines.push(format!(
            "| {} | {} | `{}` |",
            surface.surface,
            surface.status(),
            surface.files.join("`, `")
        ));
    }

    lines.push(String::new());
    lines.push("## Findings".to_string());

    for surface in &report.surfaces {
        lines.push(format!("### {}", surface.surface));

        if surface.findings.is_empty() {
            lines.push(
                "- pass — no drift markers detected in the audited source files.".to_string(),
            );
            lines.push(String::new());
            continue;
        }

        for finding in &surface.findings {
            lines.push(format!(
                "- {} — **{}**",
                finding.severity.as_str(),
                finding.title
            ));
            lines.push(format!("  - {}", finding.summary));
            if !finding.evidence.is_empty() {
                lines.push(format!("  - Evidence: `{}`", finding.evidence.join("`, `")));
            }
        }

        lines.push(String::new());
    }

    lines.join("\n")
}

pub fn write_prompt_chrome_consistency_report(
    repo_root: &Path,
    output_root: &Path,
) -> Result<PathBuf> {
    let report = build_prompt_chrome_consistency_report(repo_root)?;
    let markdown = render_prompt_chrome_consistency_markdown(&report);

    let audit_dir = output_root.join("audit");
    fs::create_dir_all(&audit_dir)
        .with_context(|| format!("failed to create {}", audit_dir.display()))?;

    let output_path = audit_dir.join(format!("{}.md", report.slug));
    fs::write(&output_path, markdown)
        .with_context(|| format!("failed to write {}", output_path.display()))?;

    tracing::info!(
        target: "script_kit::audit",
        slug = report.slug,
        output = %output_path.display(),
        surface_count = report.surfaces.len(),
        "wrote audit report"
    );

    Ok(output_path)
}

// ── Workflow Affordance Consistency Report ──────────────────────────

const WORKFLOW_REPORT_SLUG: &str = "workflow-affordance-consistency";
const WORKFLOW_REPORT_TITLE: &str = "Workflow Affordance Consistency Audit";

#[derive(Clone, Copy, Debug)]
struct WorkflowSurfaceSpec {
    surface: &'static str,
    files: &'static [&'static str],
}

const WORKFLOW_SURFACES: &[WorkflowSurfaceSpec] = &[
    WorkflowSurfaceSpec {
        surface: "actions_dialog",
        files: &["src/actions/dialog.rs"],
    },
    WorkflowSurfaceSpec {
        surface: "clipboard_history",
        files: &[
            "src/render_builtins/clipboard.rs",
            "src/render_builtins/clipboard_history_layout.rs",
        ],
    },
    WorkflowSurfaceSpec {
        surface: "file_search",
        files: &["src/render_builtins/file_search.rs"],
    },
    WorkflowSurfaceSpec {
        surface: "render_prompts::chat",
        files: &[
            "src/render_prompts/other.rs",
            "src/prompts/chat/render_core.rs",
        ],
    },
    WorkflowSurfaceSpec {
        surface: "render_prompts::term",
        files: &["src/render_prompts/term.rs"],
    },
    WorkflowSurfaceSpec {
        surface: "prompts::path",
        files: &["src/prompts/path/render.rs"],
    },
];

fn audit_workflow_affordance_surface(
    spec: WorkflowSurfaceSpec,
    repo_root: &Path,
) -> Result<AuditSurfaceResult> {
    let sources = read_source_files(repo_root, spec.files)?;
    let combined = sources
        .iter()
        .map(|(_, source)| source.as_str())
        .collect::<Vec<_>>()
        .join("\n");

    let mut findings = Vec::new();

    match spec.surface {
        "actions_dialog" => {
            let has_runtime_contract = combined.contains("ActionsDialogRuntimeAudit")
                && combined.contains("actions_dialog_runtime_contract_violation")
                && combined.contains("ACTIONS_DIALOG_EXPECT_SEARCH_POSITION")
                && combined.contains("ACTIONS_DIALOG_EXPECT_FOOTER_HINT_COUNT");

            let has_three_key_footer = combined.contains("\"↵ Run\"")
                && combined.contains("\"⌘K Actions\"")
                && combined.contains("\"Tab AI\"");

            if has_runtime_contract && has_three_key_footer {
                findings.push(info(
                    "command palette contract is audited",
                    "Actions dialog already declares a machine-readable runtime contract for top search, footer hints, and chrome regressions. Treat it as the baseline command surface for every keyboard-first workflow.",
                    vec!["src/actions/dialog.rs".to_string()],
                ));
            } else {
                findings.push(warning(
                    "command palette contract is not fully reportable",
                    "Actions dialog is the core power-user surface, but the audited source no longer proves both the runtime contract and the canonical three-key footer together.",
                    vec!["src/actions/dialog.rs".to_string()],
                ));
            }
        }

        "clipboard_history" => {
            let uses_shared_expanded_scaffold =
                combined.contains("render_expanded_view_scaffold_with_hints(");
            let has_hint_audit =
                combined.contains("emit_prompt_hint_audit(\"clipboard_history\"");

            if uses_shared_expanded_scaffold && has_hint_audit {
                findings.push(info(
                    "expanded clipboard workflow is reportable",
                    "Clipboard History already routes through the shared expanded scaffold and emits footer hint audits, so its list-plus-preview workflow is visible to the audit system.",
                    vec![
                        "src/render_builtins/clipboard.rs".to_string(),
                        "src/render_builtins/clipboard_history_layout.rs".to_string(),
                    ],
                ));
            } else {
                findings.push(warning(
                    "expanded clipboard workflow drift is hard to prove",
                    "Clipboard History should keep both the shared expanded scaffold and explicit footer hint audit so the markdown report can prove workflow parity instead of relying on visual inspection.",
                    vec![
                        "src/render_builtins/clipboard.rs".to_string(),
                        "src/render_builtins/clipboard_history_layout.rs".to_string(),
                    ],
                ));
            }
        }

        "file_search" => {
            let has_mini_layout = combined.contains("render_minimal_list_prompt_scaffold(");
            let has_expanded_layout = combined.contains("render_expanded_view_scaffold(");
            let has_layout_checkpoint = combined.contains("file_search_chrome_checkpoint");
            let has_hint_audit = combined.contains("emit_prompt_hint_audit(\"file_search\"");
            let has_universal_hints = combined.contains("universal_prompt_hints()")
                || combined.contains("live_file_search_hints(");
            let has_presentation_switch =
                combined.contains("matches!(presentation, FileSearchPresentation::Mini)");
            let has_mini_chrome_audit =
                combined.contains("PromptChromeAudit::minimal_list(\"file_search\"");
            let has_expanded_chrome_audit =
                combined.contains("PromptChromeAudit::expanded(\"file_search\"");

            if has_mini_layout
                && has_expanded_layout
                && has_layout_checkpoint
                && has_hint_audit
                && has_universal_hints
                && has_presentation_switch
                && has_mini_chrome_audit
                && has_expanded_chrome_audit
            {
                findings.push(info(
                    "mini and expanded file search are both auditable",
                    "File Search already exposes both its compact and split-view workflows in source, emits distinct runtime chrome audits for each presentation, and keeps the mini footer on the canonical three-key hint strip.",
                    vec![FILE_SEARCH_LIVE_FILE.to_string()],
                ));
            } else if !has_hint_audit || !has_universal_hints {
                findings.push(warning(
                    "file search mini footer is not provably universal",
                    "File Search should keep its mini mode on the canonical `↵ Run`, `⌘K Actions`, `Tab AI` footer and emit `emit_prompt_hint_audit(\"file_search\", ...)`, otherwise the workflow report cannot prove shortcut parity.",
                    vec![FILE_SEARCH_LIVE_FILE.to_string()],
                ));
            } else {
                findings.push(warning(
                    "file search runtime chrome audit is not mode-aware",
                    "File Search renders both mini and expanded layouts, but the audited source does not yet prove that it emits distinct `PromptChromeAudit::minimal_list(...)` and `PromptChromeAudit::expanded(...)` contracts behind the `presentation` switch.",
                    vec![FILE_SEARCH_LIVE_FILE.to_string()],
                ));
            }

            if check_file_search_duplicate_layout(repo_root) {
                findings.push(warning(
                    "duplicate file_search layout source",
                    format!(
                        "workflow audit found file_search layout markers in `{}` even though the live workflow is audited from `{}`.",
                        FILE_SEARCH_STALE_LAYOUT_FILE, FILE_SEARCH_LIVE_FILE
                    ),
                    vec![
                        FILE_SEARCH_LIVE_FILE.to_string(),
                        FILE_SEARCH_STALE_LAYOUT_FILE.to_string(),
                    ],
                ));
                tracing::warn!(
                    target: "script_kit::audit",
                    report_slug = WORKFLOW_REPORT_SLUG,
                    surface = "file_search",
                    duplicate_source = FILE_SEARCH_STALE_LAYOUT_FILE,
                    "workflow audit detected duplicate file_search layout source"
                );
            }
        }

        "render_prompts::chat" => {
            let has_mini_hint_audit =
                combined.contains("emit_prompt_hint_audit(\"prompts::chat::mini\"");
            let has_full_hint_audit =
                combined.contains("emit_prompt_hint_audit(\"prompts::chat\"");
            let has_status_leading = combined.contains("footer_status_text(")
                && combined.contains("render_hint_strip_leading_text(");

            if has_mini_hint_audit && has_full_hint_audit && has_status_leading {
                findings.push(info(
                    "chat teaches the same shortcuts in mini and full modes",
                    "Chat already audits both its mini and full footers and carries status text as leading helper content instead of changing the shortcut vocabulary.",
                    vec![
                        "src/render_prompts/other.rs".to_string(),
                        "src/prompts/chat/render_core.rs".to_string(),
                    ],
                ));
            } else {
                findings.push(warning(
                    "chat shortcut discoverability is not fully reportable",
                    "Chat should keep explicit hint audits for both mini and full modes plus a single helper-text path so the report can verify parity between the compact and rich chat shells.",
                    vec![
                        "src/render_prompts/other.rs".to_string(),
                        "src/prompts/chat/render_core.rs".to_string(),
                    ],
                ));
            }
        }

        "render_prompts::term" => {
            let has_custom_exception = combined.contains("surface: \"render_prompts::term\"")
                && combined.contains("footer_mode: \"custom_hint_strip\"")
                && combined.contains("exception_reason: Some(\"terminal_owns_contextual_footer\")");

            if has_custom_exception {
                findings.push(info(
                    "terminal exception is explicit",
                    "Term keeps a contextual footer on purpose, and the exception is already encoded in the chrome audit payload instead of hiding as silent drift.",
                    vec!["src/render_prompts/term.rs".to_string()],
                ));
            } else {
                findings.push(warning(
                    "terminal exception is no longer explicit",
                    "The terminal surface should keep its custom footer documented as an audit exception so the report distinguishes intentional workflow specialization from accidental drift.",
                    vec!["src/render_prompts/term.rs".to_string()],
                ));
            }
        }

        "prompts::path" => {
            let has_minimal_scaffold =
                combined.contains("render_minimal_list_prompt_scaffold(");
            let has_hint_audit = combined.contains("emit_prompt_hint_audit(\"prompts::path\"");
            let has_chrome_audit = combined.contains("emit_prompt_chrome_audit(")
                && combined.contains("\"prompts::path\"");

            if has_minimal_scaffold && has_hint_audit && has_chrome_audit {
                findings.push(info(
                    "path prompt is fully auditable",
                    "Path prompt now emits both chrome and hint audits while staying on the shared minimal scaffold, so it participates in the same keyboard-first consistency report as the rest of the mini surfaces.",
                    vec!["src/prompts/path/render.rs".to_string()],
                ));
            } else if has_minimal_scaffold && has_hint_audit {
                findings.push(warning(
                    "missing runtime chrome audit",
                    "Path prompt already uses the shared minimal scaffold and universal footer hints, but it still lacks `emit_prompt_chrome_audit(...)`, so the report cannot prove shell parity at runtime.",
                    vec!["src/prompts/path/render.rs".to_string()],
                ));
            } else {
                findings.push(warning(
                    "path prompt drifted from the shared mini-shell contract",
                    "Path prompt should stay on the shared minimal scaffold and emit the universal footer hint audit, otherwise the keyboard-first report loses coverage for filesystem navigation.",
                    vec!["src/prompts/path/render.rs".to_string()],
                ));
            }
        }

        _ => {}
    }

    findings.sort_by_key(|finding| (finding.severity.sort_rank(), finding.title));

    let result = AuditSurfaceResult {
        surface: spec.surface,
        files: spec.files.to_vec(),
        findings,
    };

    tracing::info!(
        target: "script_kit::audit",
        report_slug = WORKFLOW_REPORT_SLUG,
        surface = result.surface,
        status = result.status(),
        finding_count = result.findings.len(),
        "workflow affordance surface audited"
    );

    if result.status() != "pass" {
        tracing::warn!(
            target: "script_kit::audit",
            report_slug = WORKFLOW_REPORT_SLUG,
            surface = result.surface,
            status = result.status(),
            finding_count = result.findings.len(),
            "workflow affordance drift detected"
        );
    }

    Ok(result)
}

pub fn build_workflow_affordance_consistency_report(repo_root: &Path) -> Result<AuditReport> {
    let mut surfaces = Vec::with_capacity(WORKFLOW_SURFACES.len());
    for spec in WORKFLOW_SURFACES {
        surfaces.push(audit_workflow_affordance_surface(*spec, repo_root)?);
    }

    let warning_count = surfaces
        .iter()
        .filter(|surface| surface.status() == "warning")
        .count();
    let error_count = surfaces
        .iter()
        .filter(|surface| surface.status() == "error")
        .count();
    let pass_count = surfaces.len() - warning_count - error_count;

    let summary = if warning_count == 0 && error_count == 0 {
        format!(
            "Scanned {} workflow surfaces. {} pass, {} warning, {} error. Keyboard-first affordances are consistent across the audited surfaces.",
            surfaces.len(),
            pass_count,
            warning_count,
            error_count
        )
    } else {
        let drift_surfaces = surfaces
            .iter()
            .filter(|surface| surface.status() == "warning" || surface.status() == "error")
            .map(|surface| surface.surface)
            .collect::<Vec<_>>()
            .join(", ");
        format!(
            "Scanned {} workflow surfaces. {} pass, {} warning, {} error. Highest-leverage gaps: {}.",
            surfaces.len(),
            pass_count,
            warning_count,
            error_count,
            drift_surfaces
        )
    };

    tracing::info!(
        target: "script_kit::audit",
        slug = WORKFLOW_REPORT_SLUG,
        pass_count,
        warning_count,
        error_count,
        "workflow affordance audit summary built"
    );

    Ok(AuditReport {
        slug: WORKFLOW_REPORT_SLUG,
        title: WORKFLOW_REPORT_TITLE,
        summary,
        surfaces,
    })
}

pub fn render_workflow_affordance_consistency_markdown(report: &AuditReport) -> String {
    let mut lines = vec![
        format!("# {}", report.title),
        String::new(),
        "## Summary".to_string(),
        report.summary.clone(),
        String::new(),
        "## What This Checks".to_string(),
        "- Keyboard-first consistency across command surfaces: universal three-key footer, mini-vs-expanded parity, explicit exceptions, and reportable runtime audits.".to_string(),
        String::new(),
        "## Surface Status".to_string(),
        "| Surface | Status | Files |".to_string(),
        "| --- | --- | --- |".to_string(),
    ];

    for surface in &report.surfaces {
        lines.push(format!(
            "| {} | {} | `{}` |",
            surface.surface,
            surface.status(),
            surface.files.join("`, `")
        ));
    }

    lines.push(String::new());
    lines.push("## Findings".to_string());

    for surface in &report.surfaces {
        lines.push(format!("### {}", surface.surface));

        if surface.findings.is_empty() {
            lines.push(
                "- pass — no workflow-affordance drift markers detected.".to_string(),
            );
            lines.push(String::new());
            continue;
        }

        for finding in &surface.findings {
            lines.push(format!(
                "- {} — **{}**",
                finding.severity.as_str(),
                finding.title
            ));
            lines.push(format!("  - {}", finding.summary));
            if !finding.evidence.is_empty() {
                lines.push(format!("  - Evidence: `{}`", finding.evidence.join("`, `")));
            }
        }

        lines.push(String::new());
    }

    lines.join("\n")
}

pub fn write_workflow_affordance_consistency_report(
    repo_root: &Path,
    output_root: &Path,
) -> Result<PathBuf> {
    let report = build_workflow_affordance_consistency_report(repo_root)?;
    let markdown = render_workflow_affordance_consistency_markdown(&report);

    let audit_dir = output_root.join("audit");
    fs::create_dir_all(&audit_dir)
        .with_context(|| format!("failed to create {}", audit_dir.display()))?;

    let output_path = audit_dir.join(format!("{}.md", report.slug));
    fs::write(&output_path, markdown)
        .with_context(|| format!("failed to write {}", output_path.display()))?;

    tracing::info!(
        target: "script_kit::audit",
        slug = report.slug,
        output = %output_path.display(),
        surface_count = report.surfaces.len(),
        "wrote audit report"
    );

    Ok(output_path)
}

pub fn write_standard_audit_reports(
    repo_root: &Path,
    output_root: &Path,
) -> Result<Vec<PathBuf>> {
    let outputs = vec![
        write_prompt_chrome_consistency_report(repo_root, output_root)?,
        write_workflow_affordance_consistency_report(repo_root, output_root)?,
        write_command_bar_consistency_report(repo_root, output_root)?,
    ];

    tracing::info!(
        target: "script_kit::audit",
        report_count = outputs.len(),
        outputs = ?outputs,
        "wrote standard audit report bundle"
    );

    Ok(outputs)
}

// ── Command Bar Consistency Report ──────────────────────────────────

const COMMAND_BAR_REPORT_SLUG: &str = "command-bar-consistency";
const COMMAND_BAR_REPORT_TITLE: &str = "Command Bar Consistency Audit";

#[derive(Clone, Copy, Debug)]
struct CommandBarSurfaceSpec {
    surface: &'static str,
    audit_surface: &'static str,
    preset_fn: &'static str,
    files: &'static [&'static str],
}

const COMMAND_BAR_SURFACES: &[CommandBarSurfaceSpec] = &[
    CommandBarSurfaceSpec {
        surface: "command_bar::main_menu",
        audit_surface: "main_menu",
        preset_fn: "main_menu_style",
        files: &["src/actions/command_bar.rs"],
    },
    CommandBarSurfaceSpec {
        surface: "command_bar::no_search",
        audit_surface: "no_search",
        preset_fn: "no_search",
        files: &["src/actions/command_bar.rs"],
    },
    CommandBarSurfaceSpec {
        surface: "command_bar::notes",
        audit_surface: "notes",
        preset_fn: "notes_style",
        files: &["src/actions/command_bar.rs"],
    },
    CommandBarSurfaceSpec {
        surface: "command_bar::ai",
        audit_surface: "ai",
        preset_fn: "ai_style",
        files: &["src/actions/command_bar.rs"],
    },
];

fn extract_function_block<'a>(source: &'a str, fn_name: &str) -> &'a str {
    let public_marker = format!("pub fn {}(", fn_name);
    let private_marker = format!("fn {}(", fn_name);
    let Some(start) = source
        .find(&public_marker)
        .or_else(|| source.find(&private_marker))
    else {
        return "";
    };
    let tail = &source[start..];
    let next_pub = tail[1..].find("\n    pub fn ").map(|i| i + 1).unwrap_or(tail.len());
    let next_priv = tail[1..].find("\n    fn ").map(|i| i + 1).unwrap_or(tail.len());
    let next_cfg = tail[1..].find("\n#[cfg(").map(|i| i + 1).unwrap_or(tail.len());
    let end = next_pub.min(next_priv).min(next_cfg);
    &tail[..end]
}

fn extract_command_bar_validate_arm<'a>(source: &'a str, audit_surface: &str) -> &'a str {
    let marker = format!("\"{}\" => {{", audit_surface);
    let Some(start) = source.find(&marker) else {
        return "";
    };
    let tail = &source[start..];
    let next_arm = tail[1..].find("\n            \"").map(|i| i + 1).unwrap_or(tail.len());
    let next_default = tail[1..].find("\n            _ =>").map(|i| i + 1).unwrap_or(tail.len());
    let end = next_arm.min(next_default);
    &tail[..end]
}

fn audit_command_bar_surface(
    spec: CommandBarSurfaceSpec,
    repo_root: &Path,
) -> Result<AuditSurfaceResult> {
    let sources = read_source_files(repo_root, spec.files)?;
    let combined = sources
        .iter()
        .map(|(_, source)| source.as_str())
        .collect::<Vec<_>>()
        .join("\n");

    let preset_fn_body = extract_function_block(&combined, spec.preset_fn);
    let validate_arm = extract_command_bar_validate_arm(&combined, spec.audit_surface);

    let has_preset_constructor = !preset_fn_body.is_empty();
    let has_runtime_audit_emit = preset_fn_body.contains(&format!(
        "emit_command_bar_chrome_audit(\"{}\"",
        spec.audit_surface
    ));
    let has_validation_arm = !validate_arm.is_empty()
        && validate_arm.contains("\"search_position\"")
        && validate_arm.contains("\"section_mode\"")
        && validate_arm.contains("\"anchor\"");

    let mut findings = Vec::new();

    if has_preset_constructor && has_runtime_audit_emit && has_validation_arm {
        findings.push(info(
            "command bar preset is reportable",
            format!(
                "{} is configured by `{}` with the audited search/section/anchor contract and emits `emit_command_bar_chrome_audit(\"{}\", ...)`, so the preset can be persisted into `./audit/{}.md`.",
                spec.surface, spec.preset_fn, spec.audit_surface, COMMAND_BAR_REPORT_SLUG
            ),
            spec.files.iter().map(|file| file.to_string()).collect(),
        ));
    } else {
        if !has_preset_constructor {
            findings.push(warning(
                "missing preset constructor",
                format!(
                    "{} is listed in the command-bar audit, but `{}` is not present in `src/actions/command_bar.rs`.",
                    spec.surface, spec.preset_fn
                ),
                spec.files.iter().map(|file| file.to_string()).collect(),
            ));
        }
        if has_preset_constructor && !has_runtime_audit_emit {
            findings.push(warning(
                "missing runtime command bar audit",
                format!(
                    "{} builds a preset config but does not emit `emit_command_bar_chrome_audit(\"{}\", ...)`, so runtime drift will not show up in logs or markdown.",
                    spec.surface, spec.audit_surface
                ),
                spec.files.iter().map(|file| file.to_string()).collect(),
            ));
        }
        if !has_validation_arm {
            findings.push(warning(
                "missing validation arm",
                format!(
                    "{} does not have a `CommandBarChromeAudit::validate()` match arm for `{}`, so the report cannot distinguish intentional from accidental drift.",
                    spec.surface, spec.audit_surface
                ),
                spec.files.iter().map(|file| file.to_string()).collect(),
            ));
        }
    }

    findings.sort_by_key(|finding| (finding.severity.sort_rank(), finding.title));

    let result = AuditSurfaceResult {
        surface: spec.surface,
        files: spec.files.to_vec(),
        findings,
    };

    tracing::info!(
        target: "script_kit::audit",
        report_slug = COMMAND_BAR_REPORT_SLUG,
        surface = result.surface,
        status = result.status(),
        finding_count = result.findings.len(),
        "command bar surface audited"
    );

    if result.status() != "pass" {
        tracing::warn!(
            target: "script_kit::audit",
            report_slug = COMMAND_BAR_REPORT_SLUG,
            surface = result.surface,
            status = result.status(),
            finding_count = result.findings.len(),
            "command bar consistency drift detected"
        );
    }

    Ok(result)
}

pub fn build_command_bar_consistency_report(repo_root: &Path) -> Result<AuditReport> {
    let mut surfaces = Vec::with_capacity(COMMAND_BAR_SURFACES.len());
    for spec in COMMAND_BAR_SURFACES {
        surfaces.push(audit_command_bar_surface(*spec, repo_root)?);
    }

    let warning_count = surfaces
        .iter()
        .filter(|surface| surface.status() == "warning")
        .count();
    let error_count = surfaces
        .iter()
        .filter(|surface| surface.status() == "error")
        .count();
    let pass_count = surfaces.len() - warning_count - error_count;

    let summary = if warning_count == 0 && error_count == 0 {
        format!(
            "Scanned {} command bar presets. {} pass, {} warning, {} error. Every audited preset is visible in source, validated by runtime chrome rules, and persisted as markdown.",
            surfaces.len(),
            pass_count,
            warning_count,
            error_count
        )
    } else {
        let drift_surfaces = surfaces
            .iter()
            .filter(|surface| surface.status() == "warning" || surface.status() == "error")
            .map(|surface| surface.surface)
            .collect::<Vec<_>>()
            .join(", ");
        format!(
            "Scanned {} command bar presets. {} pass, {} warning, {} error. Highest-leverage gaps: {}.",
            surfaces.len(),
            pass_count,
            warning_count,
            error_count,
            drift_surfaces
        )
    };

    tracing::info!(
        target: "script_kit::audit",
        slug = COMMAND_BAR_REPORT_SLUG,
        pass_count,
        warning_count,
        error_count,
        "command bar audit summary built"
    );

    Ok(AuditReport {
        slug: COMMAND_BAR_REPORT_SLUG,
        title: COMMAND_BAR_REPORT_TITLE,
        summary,
        surfaces,
    })
}

pub fn render_command_bar_consistency_markdown(report: &AuditReport) -> String {
    let mut lines = vec![
        format!("# {}", report.title),
        String::new(),
        "## Summary".to_string(),
        report.summary.clone(),
        String::new(),
        "## What This Checks".to_string(),
        "- CommandBar preset parity: constructor presence, runtime `emit_command_bar_chrome_audit(...)`, and validated search/section/anchor contract for `main_menu`, `no_search`, `notes`, and `ai`.".to_string(),
        String::new(),
        "## Surface Status".to_string(),
        "| Surface | Status | Files |".to_string(),
        "| --- | --- | --- |".to_string(),
    ];

    for surface in &report.surfaces {
        lines.push(format!(
            "| {} | {} | `{}` |",
            surface.surface,
            surface.status(),
            surface.files.join("`, `")
        ));
    }

    lines.push(String::new());
    lines.push("## Findings".to_string());

    for surface in &report.surfaces {
        lines.push(format!("### {}", surface.surface));

        if surface.findings.is_empty() {
            lines.push("- pass — no command-bar drift markers detected.".to_string());
            lines.push(String::new());
            continue;
        }

        for finding in &surface.findings {
            lines.push(format!(
                "- {} — **{}**",
                finding.severity.as_str(),
                finding.title
            ));
            lines.push(format!("  - {}", finding.summary));
            if !finding.evidence.is_empty() {
                lines.push(format!("  - Evidence: `{}`", finding.evidence.join("`, `")));
            }
        }

        lines.push(String::new());
    }

    lines.join("\n")
}

pub fn write_command_bar_consistency_report(
    repo_root: &Path,
    output_root: &Path,
) -> Result<PathBuf> {
    let report = build_command_bar_consistency_report(repo_root)?;
    let markdown = render_command_bar_consistency_markdown(&report);

    let audit_dir = output_root.join("audit");
    fs::create_dir_all(&audit_dir)
        .with_context(|| format!("failed to create {}", audit_dir.display()))?;

    let output_path = audit_dir.join(format!("{}.md", report.slug));
    fs::write(&output_path, markdown)
        .with_context(|| format!("failed to write {}", output_path.display()))?;

    tracing::info!(
        target: "script_kit::audit",
        slug = report.slug,
        output = %output_path.display(),
        surface_count = report.surfaces.len(),
        "wrote audit report"
    );

    Ok(output_path)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    fn report() -> AuditReport {
        build_prompt_chrome_consistency_report(Path::new(env!("CARGO_MANIFEST_DIR")))
            .expect("report should build from current repo sources")
    }

    fn surface<'a>(report: &'a AuditReport, name: &str) -> &'a AuditSurfaceResult {
        report
            .surfaces
            .iter()
            .find(|surface| surface.surface == name)
            .expect("surface should exist in report")
    }

    #[test]
    fn prompt_chrome_consistency_report_all_surfaces_pass_or_have_known_warnings() {
        let report = report();
        let error_count = report
            .surfaces
            .iter()
            .filter(|s| s.status() == "error")
            .count();
        assert_eq!(report.surfaces.len(), 8, "expected 8 audited surfaces");
        assert_eq!(error_count, 0, "expected 0 errors");

        // file_search may carry a known "duplicate file_search layout source" warning
        // while the stale layout file still exists. All other surfaces must pass clean.
        let unexpected_warnings: Vec<_> = report
            .surfaces
            .iter()
            .filter(|s| s.status() == "warning" && s.surface != "file_search")
            .map(|s| s.surface)
            .collect();
        assert!(
            unexpected_warnings.is_empty(),
            "unexpected warnings on: {:?}",
            unexpected_warnings
        );
    }

    #[test]
    fn prompt_chrome_consistency_report_file_search_detects_duplicate_layout() {
        let report = report();
        let file_search = surface(&report, "file_search");
        let has_duplicate_warning = file_search
            .findings
            .iter()
            .any(|f| f.title == "duplicate file_search layout source");
        // The stale layout file still exists, so the duplicate warning should fire
        let stale_exists =
            Path::new(env!("CARGO_MANIFEST_DIR")).join(FILE_SEARCH_STALE_LAYOUT_FILE).exists();
        assert_eq!(
            has_duplicate_warning, stale_exists,
            "duplicate layout warning should match stale file existence"
        );
    }

    #[test]
    fn prompt_chrome_consistency_report_path_prompt_passes() {
        let report = report();
        let path = surface(&report, "prompts::path");
        assert_eq!(path.status(), "pass");
    }

    #[test]
    fn prompt_chrome_consistency_report_clipboard_history_passes() {
        let report = report();
        let clipboard_history = surface(&report, "clipboard_history");
        assert_eq!(clipboard_history.status(), "pass");
    }

    #[test]
    fn prompt_chrome_consistency_report_keeps_term_as_documented_exception() {
        let report = report();
        let term = surface(&report, "render_prompts::term");
        assert_eq!(term.status(), "pass");
        assert!(term.has_only_info_findings());
        assert!(term
            .findings
            .iter()
            .any(|finding| finding.title == "contextual footer exception"));
    }

    #[test]
    fn prompt_chrome_consistency_summary_is_well_formed() {
        let report = report();
        // Summary should always contain the scan line
        assert!(
            report.summary.contains("Scanned"),
            "summary should start with scan line: {}",
            report.summary
        );
        // If the stale layout file still exists, the summary may mention drifts
        // for file_search — that is the expected behavior.
        let stale_exists =
            Path::new(env!("CARGO_MANIFEST_DIR")).join(FILE_SEARCH_STALE_LAYOUT_FILE).exists();
        if !stale_exists {
            assert!(
                !report.summary.contains("Highest-leverage current drifts"),
                "summary should not mention drifts once the stale layout file is removed: {}",
                report.summary
            );
        }
    }

    #[test]
    fn workflow_affordance_report_all_surfaces_pass_or_have_known_warnings() {
        let report = build_workflow_affordance_consistency_report(Path::new(env!(
            "CARGO_MANIFEST_DIR"
        )))
        .expect("workflow report should build from current repo sources");

        let error_count = report
            .surfaces
            .iter()
            .filter(|surface| surface.status() == "error")
            .count();

        assert_eq!(report.surfaces.len(), 6, "expected 6 audited surfaces");
        assert_eq!(error_count, 0, "expected 0 errors");

        // file_search may carry a known "duplicate file_search layout source" warning
        // while the stale layout file still exists. All other surfaces must pass clean.
        let unexpected_warnings: Vec<_> = report
            .surfaces
            .iter()
            .filter(|s| s.status() == "warning" && s.surface != "file_search")
            .map(|s| s.surface)
            .collect();
        assert!(
            unexpected_warnings.is_empty(),
            "unexpected warnings on: {:?}",
            unexpected_warnings
        );
    }

    #[test]
    fn workflow_affordance_report_path_prompt_passes() {
        let report = build_workflow_affordance_consistency_report(Path::new(env!(
            "CARGO_MANIFEST_DIR"
        )))
        .expect("workflow report should build from current repo sources");
        let path_prompt = surface(&report, "prompts::path");

        assert_eq!(path_prompt.status(), "pass");
        assert!(path_prompt
            .findings
            .iter()
            .any(|finding| finding.title == "path prompt is fully auditable"));
    }

    #[test]
    fn prompt_chrome_consistency_markdown_contains_scope_notes() {
        let report = report();
        let markdown = render_prompt_chrome_consistency_markdown(&report);

        assert!(markdown.contains("# Prompt Chrome Consistency Audit"));
        assert!(markdown.contains("## Summary"));
        assert!(markdown.contains("## Scope Notes"));
        assert!(markdown.contains("Excluded this pass:"));
        assert!(markdown.contains("model_selector_popup.rs"));
        assert!(markdown.contains("## Surface Status"));
        assert!(markdown.contains("## Findings"));
        assert!(markdown.contains("Intentional exception: render_prompts::term"));
    }

    #[test]
    fn prompt_chrome_consistency_report_writes_markdown_to_temp_output_root() {
        let repo_root = Path::new(env!("CARGO_MANIFEST_DIR"));
        let temp = tempdir().expect("tempdir should be created");
        let output = write_prompt_chrome_consistency_report(repo_root, temp.path())
            .expect("report should write to temp output root");

        assert!(output.ends_with("audit/prompt-chrome-consistency.md"));
        assert!(output.exists());

        let markdown =
            fs::read_to_string(&output).expect("written report should be readable as text");
        assert!(markdown.contains("# Prompt Chrome Consistency Audit"));
        assert!(markdown.contains("## Scope Notes"));
        assert!(markdown.contains("## Findings"));
    }

    #[test]
    #[ignore = "writes ./audit/prompt-chrome-consistency.md into the repository"]
    fn write_prompt_chrome_consistency_report_to_repo_audit_dir() {
        let repo_root = Path::new(env!("CARGO_MANIFEST_DIR"));
        let output = write_prompt_chrome_consistency_report(repo_root, repo_root)
            .expect("report should write into the repository audit directory");

        assert!(output.ends_with("audit/prompt-chrome-consistency.md"));
        assert!(output.exists());
    }
}
