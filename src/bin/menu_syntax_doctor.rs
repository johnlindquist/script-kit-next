//! `menu-syntax-doctor` — CLI subcommand wiring for the
//! [[script_kit_gpui::menu_syntax::doctor::validate]] engine (Pass 11
//! shipped the engine; Pass 37 wires this CLI). Reads a JSON or YAML
//! `menuSyntax` value from a file path or stdin, runs the validator,
//! prints the report, and exits non-zero when any `Error`-severity
//! issue is found.
//!
//! Usage:
//!   menu-syntax-doctor [--path <file>] [--json]
//!
//! `--path` argument is optional; if omitted the validator reads from
//! stdin so the CLI plays nicely with shell pipelines
//! (`cat foo.json | menu-syntax-doctor --json`).
//!
//! Exit code:
//!   0 — no errors (warnings allowed)
//!   1 — at least one error-severity issue
//!   2 — argument or input parse failure (the input was unreadable, not
//!        merely invalid by spec — argv error stays distinct from spec error)
//!
//! Receipt: `cargo test --features menu-syntax-doctor --test menu_syntax_doctor_cli`

use std::fs;
use std::io::{self, Read};
use std::process::ExitCode;

use script_kit_gpui::menu_syntax::{doctor_validate_at_path, DoctorReport, DoctorSeverity};
use serde_json::Value;

#[derive(Debug)]
struct CliArgs {
    path: Option<String>,
    json: bool,
}

fn parse_args(argv: &[String]) -> Result<CliArgs, String> {
    let mut path: Option<String> = None;
    let mut json = false;
    let mut iter = argv.iter().skip(1);
    while let Some(arg) = iter.next() {
        match arg.as_str() {
            "--json" => json = true,
            "--path" => {
                path = Some(
                    iter.next()
                        .ok_or_else(|| "--path requires a file path argument".to_string())?
                        .clone(),
                );
            }
            other if other.starts_with("--path=") => {
                path = Some(other.trim_start_matches("--path=").to_string());
            }
            "-h" | "--help" => {
                return Err("usage: menu-syntax-doctor [--path <file>] [--json]".to_string());
            }
            other => return Err(format!("unknown argument: {other}")),
        }
    }
    Ok(CliArgs { path, json })
}

fn read_input(path: Option<&str>) -> Result<String, String> {
    match path {
        Some(p) => fs::read_to_string(p).map_err(|e| format!("read {p}: {e}")),
        None => {
            let mut buf = String::new();
            io::stdin()
                .read_to_string(&mut buf)
                .map_err(|e| format!("read stdin: {e}"))?;
            Ok(buf)
        }
    }
}

/// Try JSON first, fall back to YAML so authors can pipe either format.
fn parse_value(raw: &str) -> Result<Value, String> {
    if let Ok(v) = serde_json::from_str::<Value>(raw) {
        return Ok(v);
    }
    if let Ok(v) = serde_yaml::from_str::<Value>(raw) {
        return Ok(v);
    }
    if let Some(frontmatter) = extract_yaml_frontmatter(raw) {
        return serde_yaml::from_str::<Value>(frontmatter)
            .map_err(|e| format!("parse YAML frontmatter: {e}"));
    }
    serde_yaml::from_str::<Value>(raw).map_err(|e| format!("parse input as JSON or YAML: {e}"))
}

fn extract_yaml_frontmatter(raw: &str) -> Option<&str> {
    let trimmed = raw.trim_start();
    let rest = trimmed.strip_prefix("---")?;
    let rest = rest
        .strip_prefix("\r\n")
        .or_else(|| rest.strip_prefix('\n'))
        .unwrap_or(rest);
    let mut offset = 0;
    for line in rest.split_inclusive('\n') {
        let line_without_newline = line.trim_end_matches('\n').trim_end_matches('\r');
        if line_without_newline == "---" {
            return Some(&rest[..offset]);
        }
        offset += line.len();
    }
    None
}

struct DoctorInput<'a> {
    value: &'a Value,
    path: &'static str,
}

fn select_menu_syntax_value(root: &Value) -> DoctorInput<'_> {
    if let Some(value) = root.get("menuSyntax") {
        return DoctorInput {
            value,
            path: "$.menuSyntax",
        };
    }
    if let Some(value) = root
        .get("metadata")
        .and_then(|metadata| metadata.get("menuSyntax"))
    {
        return DoctorInput {
            value,
            path: "$.metadata.menuSyntax",
        };
    }
    DoctorInput {
        value: root,
        path: "$",
    }
}

fn render_human(report: &DoctorReport) -> String {
    if report.issues.is_empty() {
        return "menu-syntax-doctor: OK (no issues)\n".to_string();
    }
    let mut out = String::new();
    let err_count = report
        .issues
        .iter()
        .filter(|i| i.severity == DoctorSeverity::Error)
        .count();
    let warn_count = report.issues.len() - err_count;
    out.push_str(&format!(
        "menu-syntax-doctor: {err_count} error(s), {warn_count} warning(s)\n"
    ));
    for issue in &report.issues {
        let tag = match issue.severity {
            DoctorSeverity::Error => "error",
            DoctorSeverity::Warning => "warn",
        };
        out.push_str(&format!("  [{tag}] {}: {}\n", issue.path, issue.message));
    }
    out
}

fn render_json(report: &DoctorReport) -> String {
    serde_json::to_string_pretty(report).expect("DoctorReport is serializable")
}

fn run(argv: &[String]) -> Result<u8, String> {
    let args = parse_args(argv)?;
    let raw = read_input(args.path.as_deref())?;
    let value = parse_value(&raw)?;
    let input = select_menu_syntax_value(&value);
    let report = doctor_validate_at_path(input.value, input.path);
    let rendered = if args.json {
        render_json(&report)
    } else {
        render_human(&report)
    };
    println!("{rendered}");
    Ok(if report.has_errors() { 1 } else { 0 })
}

fn main() -> ExitCode {
    let argv: Vec<String> = std::env::args().collect();
    match run(&argv) {
        Ok(code) => ExitCode::from(code),
        Err(msg) => {
            eprintln!("menu-syntax-doctor: {msg}");
            ExitCode::from(2)
        }
    }
}
