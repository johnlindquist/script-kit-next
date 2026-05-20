use std::collections::HashMap;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use super::date::ResolvedCaptureInvocation;
use super::payload::{resolve_capture_target, ArgvInvocation, CaptureObjectRef};

pub const MENU_SYNTAX_PAYLOAD_VERSION: &str = "menu-syntax.payload.v1";
pub const MENU_SYNTAX_PAYLOAD_SCHEMA_ID: &str = "kit://schema/menu-syntax/payload-v1";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum MenuSyntaxHandlerKind {
    Script,
    Scriptlet,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MenuSyntaxHandlerRef {
    pub kind: MenuSyntaxHandlerKind,
    pub command_id: String,
    pub name: String,
    #[serde(default)]
    pub plugin_id: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MenuSyntaxPayload {
    pub version: String,
    pub family: String,
    pub target: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub raw_target: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub canonical_target: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub target_alias_of: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub operation: Option<String>,
    pub raw: String,
    pub body: String,
    pub tags: Vec<String>,
    pub priority: Option<u8>,
    pub url: Option<String>,
    pub duration: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub duration_resolved: Option<super::date::ResolvedDuration>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub recurrence: Option<super::date::ResolvedRecurrence>,
    pub kv: HashMap<String, String>,
    pub dates: Vec<super::date::ResolvedDate>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub unresolved_dates: Vec<super::date::UnresolvedDate>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub object_refs: Vec<CaptureObjectRef>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub primary_object_ref: Option<CaptureObjectRef>,
    pub handler: MenuSyntaxHandlerRef,
}

pub fn build_capture_payload(
    handler: MenuSyntaxHandlerRef,
    invocation: ResolvedCaptureInvocation,
) -> MenuSyntaxPayload {
    let mut kv_map: HashMap<String, String> = HashMap::new();
    for (k, v) in invocation.kv {
        kv_map.insert(k, v);
    }
    let target_resolution = resolve_capture_target(&invocation.target);
    MenuSyntaxPayload {
        version: MENU_SYNTAX_PAYLOAD_VERSION.to_string(),
        family: "capture.v1".to_string(),
        target: invocation.target.clone(),
        raw_target: Some(invocation.target.clone()),
        canonical_target: target_resolution
            .as_ref()
            .map(|resolution| resolution.canonical_target_str().to_string()),
        target_alias_of: target_resolution
            .as_ref()
            .and_then(|resolution| resolution.target_alias_of_str().map(str::to_string)),
        operation: target_resolution
            .as_ref()
            .map(|resolution| resolution.operation.as_str().to_string()),
        raw: invocation.raw,
        body: invocation.body,
        tags: invocation.tags,
        priority: invocation.priority,
        url: invocation.url,
        duration: invocation.duration,
        duration_resolved: invocation.duration_resolved,
        recurrence: invocation.recurrence,
        kv: kv_map,
        dates: invocation.dates,
        unresolved_dates: invocation.unresolved_dates,
        object_refs: Vec::new(),
        primary_object_ref: None,
        handler,
    }
}

pub fn payload_env(payload_path: &Path, payload: &MenuSyntaxPayload) -> Vec<(String, String)> {
    vec![
        ("KIT_MENU_SYNTAX".to_string(), "1".to_string()),
        (
            "KIT_MENU_SYNTAX_VERSION".to_string(),
            payload.version.clone(),
        ),
        (
            "KIT_MENU_SYNTAX_PAYLOAD_PATH".to_string(),
            payload_path.to_string_lossy().to_string(),
        ),
        ("KIT_MENU_SYNTAX_FAMILY".to_string(), payload.family.clone()),
        ("KIT_MENU_SYNTAX_TARGET".to_string(), payload.target.clone()),
        (
            "KIT_MENU_SYNTAX_HANDLER_KIND".to_string(),
            match payload.handler.kind {
                MenuSyntaxHandlerKind::Script => "script".to_string(),
                MenuSyntaxHandlerKind::Scriptlet => "scriptlet".to_string(),
            },
        ),
        (
            "KIT_MENU_SYNTAX_HANDLER_COMMAND_ID".to_string(),
            payload.handler.command_id.clone(),
        ),
    ]
}

pub fn command_env(invocation: &ArgvInvocation) -> Vec<(String, String)> {
    let fields_json = serde_json::to_string(&invocation.fields).unwrap_or_else(|_| "[]".into());
    let tags_json = serde_json::to_string(&invocation.tags).unwrap_or_else(|_| "[]".into());
    vec![
        ("KIT_MENU_SYNTAX".to_string(), "1".to_string()),
        (
            "KIT_MENU_SYNTAX_FAMILY".to_string(),
            "command.v1".to_string(),
        ),
        (
            "KIT_MENU_SYNTAX_COMMAND_HEAD".to_string(),
            invocation.head.clone(),
        ),
        ("KIT_MENU_SYNTAX_COMMAND_FIELDS".to_string(), fields_json),
        ("KIT_MENU_SYNTAX_COMMAND_TAGS".to_string(), tags_json),
    ]
}

pub fn write_payload_tempfile(dir: &Path, payload: &MenuSyntaxPayload) -> std::io::Result<PathBuf> {
    std::fs::create_dir_all(dir)?;
    let json = serde_json::to_string_pretty(payload).map_err(std::io::Error::other)?;
    let filename = format!(
        "{}-{}.json",
        payload.family.replace('.', "_"),
        unique_suffix()
    );
    let path = dir.join(filename);
    std::fs::write(&path, json)?;
    Ok(path)
}

fn unique_suffix() -> String {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    format!("{:x}", now)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::menu_syntax::capture::{parse_capture, CaptureParse};
    use crate::menu_syntax::date::{resolve_capture_dates, MenuSyntaxClock};
    use chrono_tz::America::Denver;

    fn clock() -> MenuSyntaxClock {
        MenuSyntaxClock::fixed("2026-04-23T12:00:00", Denver).expect("clock")
    }

    fn resolved_capture(input: &str) -> ResolvedCaptureInvocation {
        let inv = match parse_capture(input) {
            CaptureParse::Ok(inv) => inv,
            CaptureParse::Incomplete(s) => panic!("incomplete: {s:?}"),
        };
        resolve_capture_dates(&inv, &clock())
    }

    fn handler() -> MenuSyntaxHandlerRef {
        MenuSyntaxHandlerRef {
            kind: MenuSyntaxHandlerKind::Script,
            command_id: "script/main:Capture Todo Inbox".to_string(),
            name: "Capture Todo Inbox".to_string(),
            plugin_id: Some("main".to_string()),
        }
    }

    #[test]
    fn build_payload_preserves_fields() {
        let invocation = resolved_capture(";todo Renew passport tomorrow 3pm #errands p1");
        let payload = build_capture_payload(handler(), invocation);
        assert_eq!(payload.version, MENU_SYNTAX_PAYLOAD_VERSION);
        assert_eq!(payload.family, "capture.v1");
        assert_eq!(payload.target, "todo");
        assert_eq!(payload.raw_target.as_deref(), Some("todo"));
        assert_eq!(payload.canonical_target.as_deref(), Some("todo"));
        assert_eq!(payload.operation.as_deref(), Some("create"));
        assert_eq!(payload.body, "Renew passport");
        assert_eq!(payload.tags, vec!["errands".to_string()]);
        assert_eq!(payload.priority, Some(1));
        assert_eq!(payload.dates.len(), 1);
        assert_eq!(payload.handler.command_id, "script/main:Capture Todo Inbox");
    }

    #[test]
    fn build_payload_preserves_raw_alias_and_adds_canonical_target() {
        let invocation = resolved_capture(";reminder Walk dog tomorrow #home");
        let payload = build_capture_payload(handler(), invocation);
        assert_eq!(payload.target, "reminder");
        assert_eq!(payload.raw_target.as_deref(), Some("reminder"));
        assert_eq!(payload.canonical_target.as_deref(), Some("todo"));
        assert_eq!(payload.target_alias_of.as_deref(), Some("todo"));
        assert_eq!(payload.operation.as_deref(), Some("remind"));
        assert!(payload.object_refs.is_empty());
        assert!(payload.primary_object_ref.is_none());
    }

    // doc-anchor-removed: menu-syntax Execution Payload
    #[test]
    fn payload_env_contract_contains_version_family_target_and_path() {
        let invocation = resolved_capture(";todo Renew passport tomorrow 3pm #errands");
        let payload = build_capture_payload(handler(), invocation);
        let env = payload_env(Path::new("/tmp/payload.json"), &payload);
        let map: HashMap<String, String> = env.into_iter().collect();
        assert_eq!(map.get("KIT_MENU_SYNTAX").map(String::as_str), Some("1"));
        assert_eq!(
            map.get("KIT_MENU_SYNTAX_VERSION").map(String::as_str),
            Some(MENU_SYNTAX_PAYLOAD_VERSION)
        );
        assert_eq!(
            map.get("KIT_MENU_SYNTAX_PAYLOAD_PATH").map(String::as_str),
            Some("/tmp/payload.json")
        );
        assert_eq!(
            map.get("KIT_MENU_SYNTAX_FAMILY").map(String::as_str),
            Some("capture.v1")
        );
        assert_eq!(
            map.get("KIT_MENU_SYNTAX_TARGET").map(String::as_str),
            Some("todo")
        );
        assert_eq!(
            map.get("KIT_MENU_SYNTAX_HANDLER_KIND").map(String::as_str),
            Some("script")
        );
        assert_eq!(
            map.get("KIT_MENU_SYNTAX_HANDLER_COMMAND_ID")
                .map(String::as_str),
            Some("script/main:Capture Todo Inbox")
        );
    }

    #[test]
    fn command_env_contains_head_fields_and_tags() {
        let invocation = ArgvInvocation {
            head: "deploy".to_string(),
            fields: vec![("env".to_string(), "prod".to_string())],
            tags: vec!["release".to_string()],
            argv: vec!["--dry-run".to_string()],
            raw: ">deploy env:prod #release -- --dry-run".to_string(),
        };
        let env: HashMap<String, String> = command_env(&invocation).into_iter().collect();
        assert_eq!(env.get("KIT_MENU_SYNTAX").map(String::as_str), Some("1"));
        assert_eq!(
            env.get("KIT_MENU_SYNTAX_FAMILY").map(String::as_str),
            Some("command.v1")
        );
        assert_eq!(
            env.get("KIT_MENU_SYNTAX_COMMAND_HEAD").map(String::as_str),
            Some("deploy")
        );
        assert_eq!(
            env.get("KIT_MENU_SYNTAX_COMMAND_FIELDS")
                .map(String::as_str),
            Some("[[\"env\",\"prod\"]]")
        );
        assert_eq!(
            env.get("KIT_MENU_SYNTAX_COMMAND_TAGS").map(String::as_str),
            Some("[\"release\"]")
        );
    }

    #[test]
    fn writes_payload_tempfile_and_round_trips() {
        let invocation = resolved_capture(";note Decision #menu-syntax");
        let payload = build_capture_payload(handler(), invocation);
        let dir = std::env::temp_dir().join("menu-syntax-tests");
        let path = write_payload_tempfile(&dir, &payload).expect("write");
        assert!(path.exists());
        let read: MenuSyntaxPayload =
            serde_json::from_str(&std::fs::read_to_string(&path).unwrap()).unwrap();
        assert_eq!(read.body, payload.body);
        assert_eq!(read.family, payload.family);
        assert_eq!(read.handler.command_id, payload.handler.command_id);
        let _ = std::fs::remove_file(&path);
    }
}
