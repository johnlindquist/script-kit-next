use chrono::{DateTime, Duration, NaiveDateTime, TimeZone, Utc};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::menu_syntax::execute::MenuSyntaxPayload;

pub type CapturePayload = MenuSyntaxPayload;

const DEFAULT_PRODID: &str = "-//Script Kit//Menu Syntax//EN";

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IcsBuildOpts {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub prodid: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub dtstamp_utc: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ValidatedIcs {
    pub uid: String,
    pub summary: String,
    pub dtstart_iso: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub dtend_iso: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub duration: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rrule: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum IcsError {
    MissingEnvelope,
    MissingVeventProp { prop: String },
    NonCrlfLineEnding,
    OverlongLine { octets: usize },
    InvalidRrule { reason: String },
    ConflictingTimeFields,
}

pub fn build_ics_event(payload: &CapturePayload, opts: &IcsBuildOpts) -> String {
    let Some(start) = payload.dates.first() else {
        return String::new();
    };
    let Some(start_dt) = parse_rfc3339_utc(&start.iso) else {
        return String::new();
    };
    let end_dt = start
        .end_iso
        .as_deref()
        .and_then(parse_rfc3339_utc)
        .or_else(|| {
            payload
                .duration_resolved
                .as_ref()
                .map(|duration| start_dt + Duration::seconds(duration.seconds as i64))
        })
        .unwrap_or_else(|| start_dt + Duration::minutes(30));

    let summary = if payload.body.trim().is_empty() {
        "Untitled event"
    } else {
        payload.body.trim()
    };
    let description = build_description(payload);
    let uid = stable_uid(payload, &start.iso);
    let dtstamp = opts
        .dtstamp_utc
        .clone()
        .unwrap_or_else(|| format_ics_utc(Utc::now()));
    let prodid = opts.prodid.as_deref().unwrap_or(DEFAULT_PRODID);

    let mut lines = vec![
        "BEGIN:VCALENDAR".to_string(),
        "VERSION:2.0".to_string(),
        format!("PRODID:{prodid}"),
        "BEGIN:VEVENT".to_string(),
        format!("UID:{uid}"),
        format!("DTSTAMP:{dtstamp}"),
        format!("DTSTART:{}", format_ics_utc(start_dt)),
        format!("DTEND:{}", format_ics_utc(end_dt)),
        format!("SUMMARY:{}", escape_ical_text(summary)),
    ];

    if let Some(recurrence) = &payload.recurrence {
        lines.push(format!("RRULE:{}", recurrence.rrule));
    }

    lines.extend([
        format!("DESCRIPTION:{}", escape_ical_text(&description)),
        "END:VEVENT".to_string(),
        "END:VCALENDAR".to_string(),
    ]);

    let mut out = String::new();
    for line in lines {
        out.push_str(&fold_ical_line(&line));
        out.push_str("\r\n");
    }
    out
}

pub fn validate_ics(input: &str) -> Result<ValidatedIcs, IcsError> {
    validate_crlf(input)?;
    for physical in input.split("\r\n") {
        if !physical.is_empty() {
            let octets = physical.len();
            if octets > 75 {
                return Err(IcsError::OverlongLine { octets });
            }
        }
    }

    let lines = unfold_ical_lines(input)?;
    if !has_line(&lines, "BEGIN:VCALENDAR") || !has_line(&lines, "END:VCALENDAR") {
        return Err(IcsError::MissingEnvelope);
    }
    if !has_line(&lines, "VERSION:2.0") {
        return Err(IcsError::MissingEnvelope);
    }
    let vevent_begins = lines
        .iter()
        .filter(|line| line.as_str() == "BEGIN:VEVENT")
        .count();
    let vevent_ends = lines
        .iter()
        .filter(|line| line.as_str() == "END:VEVENT")
        .count();
    if vevent_begins != 1 || vevent_ends != 1 {
        return Err(IcsError::MissingEnvelope);
    }

    let start_idx = lines
        .iter()
        .position(|line| line == "BEGIN:VEVENT")
        .ok_or(IcsError::MissingEnvelope)?;
    let end_idx = lines
        .iter()
        .position(|line| line == "END:VEVENT")
        .ok_or(IcsError::MissingEnvelope)?;
    if start_idx >= end_idx {
        return Err(IcsError::MissingEnvelope);
    }
    let event = &lines[start_idx + 1..end_idx];

    let uid = required_prop(event, "UID")?.to_string();
    let _dtstamp = required_prop(event, "DTSTAMP")?;
    let summary = unescape_ical_text(required_prop(event, "SUMMARY")?);
    let dtstart_raw = prop_line(event, "DTSTART").ok_or_else(|| IcsError::MissingVeventProp {
        prop: "DTSTART".to_string(),
    })?;
    let dtstart_iso = parse_ics_datetime_prop(dtstart_raw)?;
    let dtend = prop_value(event, "DTEND");
    let duration = prop_value(event, "DURATION");
    if dtend.is_some() && duration.is_some() {
        return Err(IcsError::ConflictingTimeFields);
    }
    if dtend.is_none() && duration.is_none() {
        return Err(IcsError::MissingVeventProp {
            prop: "DTEND_OR_DURATION".to_string(),
        });
    }
    let dtend_iso = if dtend.is_some() {
        let line = prop_line(event, "DTEND").ok_or_else(|| IcsError::MissingVeventProp {
            prop: "DTEND".to_string(),
        })?;
        Some(parse_ics_datetime_prop(line)?)
    } else {
        None
    };
    let duration = duration.map(ToString::to_string);
    let rrule = prop_value(event, "RRULE").map(ToString::to_string);
    if let Some(rrule) = &rrule {
        validate_rrule(rrule)?;
    }

    Ok(ValidatedIcs {
        uid,
        summary,
        dtstart_iso,
        dtend_iso,
        duration,
        rrule,
    })
}

fn validate_crlf(input: &str) -> Result<(), IcsError> {
    let bytes = input.as_bytes();
    for (idx, byte) in bytes.iter().enumerate() {
        if *byte == b'\n' && idx.checked_sub(1).and_then(|i| bytes.get(i)) != Some(&b'\r') {
            return Err(IcsError::NonCrlfLineEnding);
        }
    }
    if input.contains('\r') && !input.contains("\r\n") {
        return Err(IcsError::NonCrlfLineEnding);
    }
    Ok(())
}

fn unfold_ical_lines(input: &str) -> Result<Vec<String>, IcsError> {
    let mut lines: Vec<String> = Vec::new();
    for raw in input.split("\r\n") {
        if raw.is_empty() {
            continue;
        }
        if raw.starts_with(' ') || raw.starts_with('\t') {
            let Some(last) = lines.last_mut() else {
                return Err(IcsError::MissingEnvelope);
            };
            last.push_str(&raw[1..]);
        } else {
            lines.push(raw.to_string());
        }
    }
    Ok(lines)
}

fn required_prop<'a>(lines: &'a [String], prop: &str) -> Result<&'a str, IcsError> {
    prop_value(lines, prop).ok_or_else(|| IcsError::MissingVeventProp {
        prop: prop.to_string(),
    })
}

fn prop_value<'a>(lines: &'a [String], prop: &str) -> Option<&'a str> {
    let prefix = format!("{prop}:");
    let param_prefix = format!("{prop};");
    lines.iter().find_map(|line| {
        if line.starts_with(&prefix) {
            line.split_once(':').map(|(_, value)| value)
        } else if line.starts_with(&param_prefix) {
            line.split_once(':').map(|(_, value)| value)
        } else {
            None
        }
    })
}

fn prop_line<'a>(lines: &'a [String], prop: &str) -> Option<&'a str> {
    let prefix = format!("{prop}:");
    let param_prefix = format!("{prop};");
    lines
        .iter()
        .find(|line| line.starts_with(&prefix) || line.starts_with(&param_prefix))
        .map(String::as_str)
}

fn parse_ics_datetime_prop(value_or_line: &str) -> Result<String, IcsError> {
    let (params, value) = if let Some((left, right)) = value_or_line.split_once(':') {
        (left, right)
    } else {
        ("", value_or_line)
    };
    if let Some(stripped) = value.strip_suffix('Z') {
        let naive = NaiveDateTime::parse_from_str(stripped, "%Y%m%dT%H%M%S").map_err(|_| {
            IcsError::MissingVeventProp {
                prop: "DATE_TIME".to_string(),
            }
        })?;
        return Ok(DateTime::<Utc>::from_naive_utc_and_offset(naive, Utc).to_rfc3339());
    }

    let naive = NaiveDateTime::parse_from_str(value, "%Y%m%dT%H%M%S").map_err(|_| {
        IcsError::MissingVeventProp {
            prop: "DATE_TIME".to_string(),
        }
    })?;
    let timezone = params
        .split(';')
        .find_map(|part| part.strip_prefix("TZID="))
        .and_then(|name| name.parse::<chrono_tz::Tz>().ok());
    if let Some(tz) = timezone {
        let local = tz
            .from_local_datetime(&naive)
            .single()
            .or_else(|| tz.from_local_datetime(&naive).earliest())
            .ok_or_else(|| IcsError::MissingVeventProp {
                prop: "DATE_TIME".to_string(),
            })?;
        Ok(local.to_rfc3339())
    } else {
        Ok(DateTime::<Utc>::from_naive_utc_and_offset(naive, Utc).to_rfc3339())
    }
}

fn validate_rrule(rrule: &str) -> Result<(), IcsError> {
    let mut freq: Option<&str> = None;
    let mut byday: Option<&str> = None;
    for part in rrule.split(';') {
        let Some((key, value)) = part.split_once('=') else {
            return Err(IcsError::InvalidRrule {
                reason: format!("malformed part {part}"),
            });
        };
        match key {
            "FREQ" => freq = Some(value),
            "BYDAY" => byday = Some(value),
            "INTERVAL" => validate_rrule_int("INTERVAL", value)?,
            "COUNT" => validate_rrule_int("COUNT", value)?,
            "UNTIL" => validate_rrule_until(value)?,
            _ => {}
        }
    }
    let Some(freq) = freq else {
        return Err(IcsError::InvalidRrule {
            reason: "missing FREQ".to_string(),
        });
    };
    if !matches!(freq, "DAILY" | "WEEKLY" | "MONTHLY" | "YEARLY") {
        return Err(IcsError::InvalidRrule {
            reason: format!("unsupported FREQ {freq}"),
        });
    }
    if let Some(byday) = byday {
        let valid = byday
            .split(',')
            .all(|day| matches!(day, "MO" | "TU" | "WE" | "TH" | "FR" | "SA" | "SU"));
        if !valid {
            return Err(IcsError::InvalidRrule {
                reason: format!("invalid BYDAY {byday}"),
            });
        }
    }
    Ok(())
}

fn validate_rrule_int(key: &str, value: &str) -> Result<(), IcsError> {
    if value.is_empty() || !value.chars().all(|ch| ch.is_ascii_digit()) {
        return Err(IcsError::InvalidRrule {
            reason: format!("invalid {key} {value}"),
        });
    }
    let parsed = value.parse::<u16>().map_err(|_| IcsError::InvalidRrule {
        reason: format!("invalid {key} {value}"),
    })?;
    if !(1..=999).contains(&parsed) {
        return Err(IcsError::InvalidRrule {
            reason: format!("invalid {key} {value}"),
        });
    }
    Ok(())
}

fn validate_rrule_until(value: &str) -> Result<(), IcsError> {
    if value.len() != 16 || !value.ends_with('Z') {
        return Err(IcsError::InvalidRrule {
            reason: format!("invalid UNTIL {value}"),
        });
    }
    NaiveDateTime::parse_from_str(value, "%Y%m%dT%H%M%SZ")
        .map(|_| ())
        .map_err(|_| IcsError::InvalidRrule {
            reason: format!("invalid UNTIL {value}"),
        })
}

fn has_line(lines: &[String], needle: &str) -> bool {
    lines.iter().any(|line| line == needle)
}

fn parse_rfc3339_utc(value: &str) -> Option<DateTime<Utc>> {
    DateTime::parse_from_rfc3339(value)
        .ok()
        .map(|dt| dt.with_timezone(&Utc))
}

fn format_ics_utc(dt: DateTime<Utc>) -> String {
    dt.format("%Y%m%dT%H%M%SZ").to_string()
}

fn build_description(payload: &CapturePayload) -> String {
    let tag_line = payload
        .tags
        .iter()
        .map(|tag| format!("#{tag}"))
        .collect::<Vec<_>>()
        .join(" ");
    let calendar = payload
        .kv
        .get("calendar")
        .map(|value| format!("calendar: {value}"));
    [Some(payload.raw.clone()), Some(tag_line), calendar]
        .into_iter()
        .flatten()
        .filter(|line| !line.is_empty())
        .collect::<Vec<_>>()
        .join("\n")
}

fn stable_uid(payload: &CapturePayload, start_iso: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(payload.raw.as_bytes());
    hasher.update(b"|");
    hasher.update(start_iso.as_bytes());
    let digest = hasher.finalize();
    let hex = digest
        .iter()
        .take(16)
        .map(|byte| format!("{byte:02x}"))
        .collect::<String>();
    format!("menu-syntax-{hex}@scriptkit")
}

fn escape_ical_text(value: &str) -> String {
    let mut out = String::with_capacity(value.len());
    for ch in value.chars() {
        match ch {
            '\\' => out.push_str("\\\\"),
            ',' => out.push_str("\\,"),
            ';' => out.push_str("\\;"),
            '\n' => out.push_str("\\n"),
            '\r' => {}
            _ => out.push(ch),
        }
    }
    out
}

fn unescape_ical_text(value: &str) -> String {
    let mut out = String::with_capacity(value.len());
    let mut chars = value.chars();
    while let Some(ch) = chars.next() {
        if ch == '\\' {
            match chars.next() {
                Some('n') | Some('N') => out.push('\n'),
                Some(next @ ('\\' | ',' | ';')) => out.push(next),
                Some(next) => out.push(next),
                None => out.push(ch),
            }
        } else {
            out.push(ch);
        }
    }
    out
}

fn fold_ical_line(line: &str) -> String {
    let mut out = String::new();
    let mut current_octets = 0usize;
    for ch in line.chars() {
        let len = ch.len_utf8();
        let limit = if out.is_empty() { 75 } else { 74 };
        if current_octets + len > limit {
            out.push_str("\r\n ");
            current_octets = 1;
        }
        out.push(ch);
        current_octets += len;
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::menu_syntax::capture::{parse_capture, CaptureParse};
    use crate::menu_syntax::date::{
        resolve_capture_dates_with_accepts, MenuSyntaxClock, ResolvedCaptureInvocation,
    };
    use crate::menu_syntax::execute::{
        build_capture_payload, MenuSyntaxHandlerKind, MenuSyntaxHandlerRef,
    };
    use chrono_tz::America::Denver;

    fn clock() -> MenuSyntaxClock {
        MenuSyntaxClock::fixed("2026-04-26T09:00:00", Denver).expect("clock")
    }

    fn handler() -> MenuSyntaxHandlerRef {
        MenuSyntaxHandlerRef {
            kind: MenuSyntaxHandlerKind::Script,
            command_id: "script/main:Create macOS Calendar Event".to_string(),
            name: "Create macOS Calendar Event".to_string(),
            plugin_id: Some("main".to_string()),
        }
    }

    fn resolved(input: &str) -> ResolvedCaptureInvocation {
        let invocation = match parse_capture(input) {
            CaptureParse::Ok(invocation) => invocation,
            CaptureParse::Incomplete(state) => panic!("incomplete: {state:?}"),
        };
        resolve_capture_dates_with_accepts(&invocation, &clock(), &[])
    }

    fn payload(input: &str) -> CapturePayload {
        build_capture_payload(handler(), resolved(input))
    }

    fn opts() -> IcsBuildOpts {
        IcsBuildOpts {
            prodid: None,
            dtstamp_utc: Some("20260426T150000Z".to_string()),
        }
    }

    #[test]
    fn build_ics_event_for_mcal_til_range_round_trips_through_validate() {
        let payload = payload(";mcal Lunch with Ryan tomorrow at 12pm til 1pm");
        let ics = build_ics_event(&payload, &opts());
        let validated = validate_ics(&ics).expect("valid ics");
        assert_eq!(validated.summary, "Lunch with Ryan");
        assert_eq!(validated.dtstart_iso, "2026-04-27T18:00:00+00:00");
        assert_eq!(
            validated.dtend_iso.as_deref(),
            Some("2026-04-27T19:00:00+00:00")
        );
    }

    #[test]
    fn build_ics_event_for_mcal_for_duration_uses_dtend() {
        let payload = payload(";mcal Lunch with Ryan tom 12pm for 30mins");
        let ics = build_ics_event(&payload, &opts());
        assert!(ics.contains("DTEND:20260427T183000Z\r\n"));
        assert!(!ics.contains("\r\nDURATION:"));
    }

    #[test]
    fn build_ics_event_for_mcal_every_mon_includes_rrule_freq_weekly_byday_mo() {
        let payload = payload(";mcal Lunch w/ Ryan every mon from 1 til 2");
        let ics = build_ics_event(&payload, &opts());
        assert!(ics.contains("RRULE:FREQ=WEEKLY;BYDAY=MO\r\n"));
        assert_eq!(
            validate_ics(&ics).unwrap().rrule.as_deref(),
            Some("FREQ=WEEKLY;BYDAY=MO")
        );
    }

    #[test]
    fn validate_rrule_accepts_weekly_without_byday_and_new_fields() {
        for rrule in [
            "FREQ=WEEKLY",
            "FREQ=WEEKLY;INTERVAL=2",
            "FREQ=WEEKLY;INTERVAL=2;COUNT=4",
            "FREQ=WEEKLY;UNTIL=20260531T060000Z",
        ] {
            validate_rrule(rrule).unwrap_or_else(|err| panic!("{rrule}: {err:?}"));
        }
    }

    #[test]
    fn validate_rrule_rejects_invalid_interval_count_until_and_byday() {
        for rrule in [
            "FREQ=WEEKLY;INTERVAL=0",
            "FREQ=WEEKLY;INTERVAL=1000",
            "FREQ=WEEKLY;COUNT=0",
            "FREQ=WEEKLY;COUNT=1000",
            "FREQ=WEEKLY;UNTIL=20260531",
            "FREQ=WEEKLY;UNTIL=20260531T060000",
            "FREQ=WEEKLY;BYDAY=XX",
        ] {
            assert!(
                matches!(validate_rrule(rrule), Err(IcsError::InvalidRrule { .. })),
                "{rrule} should be rejected"
            );
        }
    }

    #[test]
    fn build_ics_event_emits_crlf_line_endings() {
        let payload = payload(";mcal Lunch with Ryan tomorrow at 12pm til 1pm");
        let ics = build_ics_event(&payload, &opts());
        assert!(ics.contains("\r\n"));
        assert!(!ics.replace("\r\n", "").contains('\n'));
    }

    #[test]
    fn build_ics_event_folds_long_summary_at_75_octets() {
        let mut payload = payload(";mcal Lunch with Ryan tomorrow at 12pm til 1pm");
        payload.body = "A very long calendar summary that needs folding because it crosses the seventy five octet physical line limit".to_string();
        let ics = build_ics_event(&payload, &opts());
        let summary_physical_lines = ics
            .split("\r\n")
            .filter(|line| line.starts_with("SUMMARY:") || line.starts_with(' '))
            .collect::<Vec<_>>();
        assert!(summary_physical_lines.len() > 1);
        assert!(summary_physical_lines
            .iter()
            .all(|line| line.as_bytes().len() <= 75));
        assert_eq!(validate_ics(&ics).unwrap().summary, payload.body);
    }

    #[test]
    fn validate_ics_rejects_missing_dtstart() {
        let input = "BEGIN:VCALENDAR\r\nVERSION:2.0\r\nBEGIN:VEVENT\r\nUID:1\r\nDTSTAMP:20260426T150000Z\r\nSUMMARY:Missing\r\nDTEND:20260427T190000Z\r\nEND:VEVENT\r\nEND:VCALENDAR\r\n";
        assert_eq!(
            validate_ics(input),
            Err(IcsError::MissingVeventProp {
                prop: "DTSTART".to_string()
            })
        );
    }

    #[test]
    fn validate_ics_rejects_lf_only_line_endings() {
        let input = "BEGIN:VCALENDAR\nVERSION:2.0\nBEGIN:VEVENT\nUID:1\nDTSTAMP:20260426T150000Z\nDTSTART:20260427T180000Z\nDTEND:20260427T190000Z\nSUMMARY:Bad\nEND:VEVENT\nEND:VCALENDAR\n";
        assert_eq!(validate_ics(input), Err(IcsError::NonCrlfLineEnding));
    }

    #[test]
    fn validate_ics_rejects_dtend_and_duration_both_present() {
        let input = "BEGIN:VCALENDAR\r\nVERSION:2.0\r\nBEGIN:VEVENT\r\nUID:1\r\nDTSTAMP:20260426T150000Z\r\nDTSTART:20260427T180000Z\r\nDTEND:20260427T190000Z\r\nDURATION:PT1H\r\nSUMMARY:Bad\r\nEND:VEVENT\r\nEND:VCALENDAR\r\n";
        assert_eq!(validate_ics(input), Err(IcsError::ConflictingTimeFields));
    }

    #[test]
    fn build_ics_event_escapes_summary_commas_semicolons_backslashes() {
        let mut payload = payload(";mcal Lunch with Ryan tomorrow at 12pm til 1pm");
        payload.body = r"Budget, roadmap; C:\tmp".to_string();
        let ics = build_ics_event(&payload, &opts());
        assert!(ics.contains(r"SUMMARY:Budget\, roadmap\; C:\\tmp"));
        assert_eq!(validate_ics(&ics).unwrap().summary, payload.body);
    }

    #[test]
    fn build_ics_event_uid_is_deterministic_for_same_payload() {
        let payload = payload(";mcal Lunch with Ryan tomorrow at 12pm til 1pm");
        let first = build_ics_event(&payload, &opts());
        let second = build_ics_event(&payload, &opts());
        assert_eq!(
            validate_ics(&first).unwrap().uid,
            validate_ics(&second).unwrap().uid
        );
    }
}
