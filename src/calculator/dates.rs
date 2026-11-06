use super::CalculatorInlineResult;
use chrono::NaiveDate;
/// Calendar-date arithmetic, not 24-hour intervals across DST. The clock stays at the caller.
pub(super) fn until(input: &str, today: NaiveDate) -> Option<CalculatorInlineResult> {
    let lower = input.trim().to_ascii_lowercase();
    let date = lower.strip_prefix("days until ")?.trim();
    if date.len() != 10 {
        return None;
    }
    let to = NaiveDate::parse_from_str(date, "%Y-%m-%d").ok()?;
    if to.to_string() != date {
        return None;
    }
    let days = to.signed_duration_since(today).num_days();
    let unit = if days.abs() == 1 { "day" } else { "days" };
    let formatted = if days < 0 {
        format!("{} {unit} ago", -days)
    } else {
        format!("{days} {unit}")
    };
    Some(CalculatorInlineResult {
        raw_input: input.to_owned(),
        normalized_expr: format!("{to} − {today}"),
        operation_name: "Calendar days".into(),
        value: days as f64,
        formatted: formatted.clone(),
        words: format!("{formatted} · {today} to {to}"),
    })
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn dates_cross_leap_days_and_reject_partial_or_impossible_input() {
        let day = NaiveDate::from_ymd_opt(2024, 2, 28).unwrap();
        assert_eq!(
            until("days until 2024-03-01", day).unwrap().formatted,
            "2 days"
        );
        assert_eq!(
            until("days until 2024-02-27", day).unwrap().formatted,
            "1 day ago"
        );
        assert!(until("days until 2024-02-30", day).is_none());
        assert!(until("days until 2024-3-1", day).is_none());
    }
}
