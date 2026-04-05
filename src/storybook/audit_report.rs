use std::{
    fs,
    path::{Path, PathBuf},
};

use anyhow::{Context, Result};

const REPORT_SLUG: &str = "prompt-chrome-consistency";
const REPORT_TITLE: &str = "Prompt Chrome Consistency Audit";

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
        surface: "clipboard_history",
        files: &["src/render_builtins/clipboard_history_layout.rs"],
        accepted_hint_surfaces: &["clipboard_history"],
    },
    SurfaceSpec {
        surface: "file_search",
        files: &[
            "src/render_builtins/file_search.rs",
            "src/render_builtins/file_search_layout.rs",
        ],
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

    match spec.surface {
        "file_search" => {
            if has_file_search_skeleton_loading(&combined)
                && has_text_only_file_search_loading(&combined)
            {
                findings.push(warning(
                    "loading state mismatch",
                    "File Search defines skeleton loading rows in source, but the live layout still collapses loading to a centered `Searching...` label. Keep a single intentional loading state on the real runtime surface.",
                    vec![
                        "src/render_builtins/file_search.rs".to_string(),
                        "src/render_builtins/file_search_layout.rs".to_string(),
                    ],
                ));
            }

            let has_custom_file_search_hints = contains_any(
                &combined,
                &[
                    "\\u{21b5} Open",
                    "\u{21b5} Open",
                    "\\u{2318}\\u{21b5} Ask AI",
                    "\u{2318}\u{21b5} Ask AI",
                    "\\u{21e5} Navigate",
                    "\u{21e5} Navigate",
                ],
            );

            if has_custom_file_search_hints {
                findings.push(warning(
                    "non-universal footer hints",
                    "File Search mini mode still advertises `\u{21b5} Open`, `\u{2318}\u{21b5} Ask AI`, and `\u{21e5} Navigate` instead of the canonical `\u{21b5} Run`, `\u{2318}K Actions`, `Tab AI` trio.",
                    vec!["src/render_builtins/file_search_layout.rs".to_string()],
                ));
            }

            if has_custom_file_search_hints && !has_runtime_hint_audit {
                findings.push(warning(
                    "missing prompt hint audit",
                    "File Search does not emit `emit_prompt_hint_audit(\"file_search\", ...)`, so its mini-mode footer drift bypasses the shared hint-contract warning path.",
                    vec!["src/render_builtins/file_search_layout.rs".to_string()],
                ));
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

    let drift_surfaces = surfaces
        .iter()
        .filter(|surface| surface.status() == "warning" || surface.status() == "error")
        .map(|surface| surface.surface)
        .collect::<Vec<_>>()
        .join(", ");

    let summary = if drift_surfaces.is_empty() {
        format!(
            "Scanned {} prompt/builtin surfaces. {} pass, {} warning, {} error. No current drift markers were detected.",
            surfaces.len(),
            pass_count,
            warning_count,
            error_count
        )
    } else {
        format!(
            "Scanned {} prompt/builtin surfaces. {} pass, {} warning, {} error. Highest-leverage current drifts: {}.",
            surfaces.len(),
            pass_count,
            warning_count,
            error_count,
            drift_surfaces
        )
    };

    Ok(AuditReport {
        slug: REPORT_SLUG,
        title: REPORT_TITLE,
        summary,
        surfaces,
    })
}

pub fn render_prompt_chrome_consistency_markdown(report: &AuditReport) -> String {
    let mut lines = vec![
        format!("# {}", report.title),
        String::new(),
        "## Summary".to_string(),
        report.summary.clone(),
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
                "- pass \u{2014} no drift markers detected in the audited source files."
                    .to_string(),
            );
            lines.push(String::new());
            continue;
        }

        for finding in &surface.findings {
            lines.push(format!(
                "- {} \u{2014} **{}**",
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
    fn prompt_chrome_consistency_report_flags_file_search_drift() {
        let report = report();
        let file_search = surface(&report, "file_search");
        assert_eq!(file_search.status(), "warning");

        assert!(file_search
            .findings
            .iter()
            .any(|finding| finding.title == "non-universal footer hints"));

        assert!(file_search
            .findings
            .iter()
            .any(|finding| finding.title == "missing prompt hint audit"));
    }

    #[test]
    fn prompt_chrome_consistency_report_flags_clipboard_history_drift() {
        let report = report();
        let clipboard_history = surface(&report, "clipboard_history");
        assert_eq!(clipboard_history.status(), "warning");

        assert!(clipboard_history
            .findings
            .iter()
            .any(|finding| finding.title == "missing runtime chrome audit"));

        assert!(clipboard_history
            .findings
            .iter()
            .any(|finding| finding.title == "manual expanded layout"));
    }

    #[test]
    fn prompt_chrome_consistency_report_keeps_term_as_documented_exception() {
        let report = report();
        let term = surface(&report, "render_prompts::term");
        assert_eq!(term.status(), "pass");
        assert!(term
            .findings
            .iter()
            .any(|finding| finding.title == "contextual footer exception"));
    }

    #[test]
    fn prompt_chrome_consistency_markdown_contains_summary_and_findings() {
        let report = report();
        let markdown = render_prompt_chrome_consistency_markdown(&report);

        assert!(markdown.contains("# Prompt Chrome Consistency Audit"));
        assert!(markdown.contains("## Summary"));
        assert!(markdown.contains("## Surface Status"));
        assert!(markdown.contains("## Findings"));
        assert!(markdown.contains("### file_search"));
        assert!(markdown.contains("### clipboard_history"));
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
