#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{FixedOffset, TimeZone, Timelike};
    use std::time::Instant;

    #[test]
    fn test_parse_cron_valid() {
        // Every minute
        let cron = parse_cron("* * * * *");
        assert!(cron.is_ok());

        // Every 5 minutes
        let cron = parse_cron("*/5 * * * *");
        assert!(cron.is_ok());

        // Every hour at minute 0
        let cron = parse_cron("0 * * * *");
        assert!(cron.is_ok());

        // Every day at 9:00 AM
        let cron = parse_cron("0 9 * * *");
        assert!(cron.is_ok());

        // Every Monday at 2:30 PM
        let cron = parse_cron("30 14 * * 1");
        assert!(cron.is_ok());
    }

    #[test]
    fn test_parse_cron_supports_standard_variants() {
        for expr in [
            "*/15 9-17 * * MON-FRI",
            "0 0 1 JAN *",
            "0 0 * * 7",     // 7 == Sunday
            "0 30 14 * * *", // seconds precision
        ] {
            assert!(
                parse_cron(expr).is_ok(),
                "expected cron variant to parse: {expr}"
            );
        }
    }

    #[test]
    fn test_parse_cron_invalid() {
        // Invalid: not enough fields
        let cron = parse_cron("* * *");
        assert!(cron.is_err());

        // Invalid: bad range
        let cron = parse_cron("60 * * * *");
        assert!(cron.is_err());
    }

    #[test]
    fn test_natural_to_cron_basic() {
        // Test basic conversions
        let result = natural_to_cron("every minute");
        assert!(
            result.is_ok(),
            "Failed to parse 'every minute': {:?}",
            result.err()
        );

        let result = natural_to_cron("every hour");
        assert!(
            result.is_ok(),
            "Failed to parse 'every hour': {:?}",
            result.err()
        );
    }

    #[test]
    fn test_natural_to_cron_specific_time() {
        // Test specific time parsing
        let result = natural_to_cron("every day at 9am");
        assert!(
            result.is_ok(),
            "Failed to parse 'every day at 9am': {:?}",
            result.err()
        );

        if let Ok(cron_str) = result {
            // Should contain hour=9
            assert!(
                cron_str.contains("9"),
                "Expected hour 9 in cron: {}",
                cron_str
            );
        }
    }

    #[test]
    fn test_natural_to_cron_weekday() {
        // Test weekday parsing
        let result = natural_to_cron("every tuesday at 2pm");
        assert!(
            result.is_ok(),
            "Failed to parse 'every tuesday at 2pm': {:?}",
            result.err()
        );

        if let Ok(cron_str) = result {
            // Should contain hour=14 (2pm)
            assert!(
                cron_str.contains("14") || cron_str.contains("2"),
                "Expected hour 14 or 2 in cron: {}",
                cron_str
            );
        }
    }

    #[test]
    fn test_natural_to_cron_output_is_parseable_by_croner() {
        let cron = natural_to_cron("every day at 9am").expect("natural language parse");
        assert!(
            parse_cron(&cron).is_ok(),
            "english-to-cron output should parse in croner: {cron}"
        );
    }

    #[test]
    fn test_scheduler_creation() {
        let (scheduler, _rx) = Scheduler::new();
        assert!(scheduler.list_scripts().is_empty());
    }

    #[test]
    fn test_scheduler_add_script_with_cron() {
        let (scheduler, _rx) = Scheduler::new();

        let result = scheduler.add_script(
            PathBuf::from("/test/script.ts"),
            Some("*/5 * * * *".to_string()),
            None,
        );

        assert!(result.is_ok(), "Failed to add script: {:?}", result.err());

        let scripts = scheduler.list_scripts();
        assert_eq!(scripts.len(), 1);
        assert_eq!(scripts[0].path, PathBuf::from("/test/script.ts"));
        assert_eq!(scripts[0].source, ScheduleSource::Cron);
    }

    #[test]
    fn test_scheduler_add_script_with_natural_language() {
        let (scheduler, _rx) = Scheduler::new();

        let result = scheduler.add_script(
            PathBuf::from("/test/script.ts"),
            None,
            Some("every hour".to_string()),
        );

        assert!(result.is_ok(), "Failed to add script: {:?}", result.err());

        let scripts = scheduler.list_scripts();
        assert_eq!(scripts.len(), 1);
        assert_eq!(scripts[0].source, ScheduleSource::NaturalLanguage);
    }

    #[test]
    fn test_scheduler_add_script_cron_takes_precedence() {
        let (scheduler, _rx) = Scheduler::new();

        let result = scheduler.add_script(
            PathBuf::from("/test/script.ts"),
            Some("0 9 * * *".to_string()),
            Some("every hour".to_string()), // Should be ignored
        );

        assert!(result.is_ok());

        let scripts = scheduler.list_scripts();
        assert_eq!(scripts.len(), 1);
        assert_eq!(scripts[0].source, ScheduleSource::Cron);
        assert_eq!(scripts[0].cron_expr, "0 9 * * *");
    }

    #[test]
    fn test_scheduler_add_script_no_schedule() {
        let (scheduler, _rx) = Scheduler::new();

        let result = scheduler.add_script(PathBuf::from("/test/script.ts"), None, None);

        assert!(result.is_err(), "Should fail when no schedule provided");
    }

    #[test]
    fn test_scheduler_remove_script() {
        let (scheduler, _rx) = Scheduler::new();

        scheduler
            .add_script(
                PathBuf::from("/test/script.ts"),
                Some("* * * * *".to_string()),
                None,
            )
            .unwrap();

        assert_eq!(scheduler.list_scripts().len(), 1);

        let removed = scheduler.remove_script(&PathBuf::from("/test/script.ts"));
        assert!(removed);
        assert!(scheduler.list_scripts().is_empty());
    }

    #[test]
    fn test_scheduler_update_existing_script() {
        let (scheduler, _rx) = Scheduler::new();
        let path = PathBuf::from("/test/script.ts");

        // Add initial script
        scheduler
            .add_script(path.clone(), Some("* * * * *".to_string()), None)
            .unwrap();

        // Update with new schedule
        scheduler
            .add_script(path.clone(), Some("0 9 * * *".to_string()), None)
            .unwrap();

        let scripts = scheduler.list_scripts();
        assert_eq!(scripts.len(), 1); // Should still be 1, not 2
        assert_eq!(scripts[0].cron_expr, "0 9 * * *");
    }

    #[test]
    fn test_scheduler_list_scripts_returns_paths_in_sorted_order() {
        let (scheduler, _rx) = Scheduler::new();

        scheduler
            .add_script(
                PathBuf::from("/test/z-last.ts"),
                Some("* * * * *".to_string()),
                None,
            )
            .unwrap();
        scheduler
            .add_script(
                PathBuf::from("/test/a-first.ts"),
                Some("* * * * *".to_string()),
                None,
            )
            .unwrap();

        let scripts = scheduler.list_scripts();
        let paths: Vec<_> = scripts
            .iter()
            .map(|script| script.path.to_string_lossy().into_owned())
            .collect();

        assert_eq!(paths, vec!["/test/a-first.ts", "/test/z-last.ts"]);
    }

    #[test]
    fn test_scheduler_event_clone() {
        let event = SchedulerEvent::RunScript(PathBuf::from("/test.ts"));
        let _cloned = event.clone();

        let error_event = SchedulerEvent::Error("test error".to_string());
        let _cloned = error_event.clone();
    }

    #[test]
    fn test_schedule_source_equality() {
        assert_eq!(ScheduleSource::Cron, ScheduleSource::Cron);
        assert_eq!(
            ScheduleSource::NaturalLanguage,
            ScheduleSource::NaturalLanguage
        );
        assert_ne!(ScheduleSource::Cron, ScheduleSource::NaturalLanguage);
    }

    #[test]
    fn test_find_next_occurrence() {
        let cron = parse_cron("0 9 * * *").unwrap(); // Every day at 9 AM
        let now = Utc::now();

        let next = find_next_occurrence(&cron, &now);
        assert!(
            next.is_ok(),
            "Failed to find next occurrence: {:?}",
            next.err()
        );

        let next = next.unwrap();
        assert!(next > now, "Next occurrence should be in the future");
    }

    #[test]
    fn test_find_next_occurrence_utc_in_timezone_keeps_local_hour() {
        let cron = parse_cron("0 9 * * *").unwrap();
        let tz = FixedOffset::west_opt(8 * 3600).expect("valid timezone");
        // 2025-01-15 16:00:00Z = 08:00 local in UTC-8.
        let after_utc = Utc
            .with_ymd_and_hms(2025, 1, 15, 16, 0, 0)
            .single()
            .expect("valid timestamp");

        let next_utc =
            find_next_occurrence_utc_in_timezone(&cron, &after_utc, &tz).expect("next run");
        let local_next = next_utc.with_timezone(&tz);

        assert_eq!(local_next.hour(), 9, "should run at 9am local time");
        assert_eq!(local_next.minute(), 0, "should run at minute 0");
    }

    #[test]
    fn test_scheduler_stop_returns_quickly_when_idle() {
        let (mut scheduler, _rx) = Scheduler::new();
        scheduler.start().expect("start scheduler");

        // Give the background thread a moment to enter the wait state.
        thread::sleep(Duration::from_millis(20));

        let start = Instant::now();
        scheduler.stop();
        let elapsed = start.elapsed();
        assert!(
            elapsed < Duration::from_millis(500),
            "scheduler stop took too long: {elapsed:?}"
        );
    }
}
