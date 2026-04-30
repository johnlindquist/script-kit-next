//! Menu-syntax payload retention policy.
//!
//! Oracle iter 004 numbers (preserved verbatim in iter 007):
//! - Always keep the newest [`KEEP_NEWEST_DEFAULT`] = 250 payload tempfiles,
//!   regardless of age.
//! - Hard cap total payloads at [`HARD_CAP_DEFAULT`] = 1000 — files ranked
//!   beyond that by age get deleted even if they are recent.
//! - Outside the newest-250 window, files older than
//!   [`AGE_CUTOFF_DAYS_DEFAULT`] = 30 days get deleted.
//! - Touch ONLY payload tempfiles (`capture_v1-*.json`). Never touch
//!   `todos.jsonl`, `events.jsonl`, `notes.jsonl`, `drafts.jsonl`,
//!   `bookmarks.jsonl`, per-day markdown notes, `.ics` calendar files, or
//!   social drafts — those are user data, not launcher byproducts.
//!
//! The planning function [`plan_retention`] is pure: it operates on a
//! caller-provided listing (mapping path → creation time) and returns the
//! exact set of paths to keep and delete. Filesystem-touching helpers are
//! intentionally separate so the policy itself is deterministic and
//! unit-testable without a sandbox directory.

use std::path::PathBuf;

/// Oracle iter 004 defaults. Exported as `pub const` so callers can reference
/// the numbers in log lines or HUD copy without re-deriving them.
pub const KEEP_NEWEST_DEFAULT: usize = 250;
pub const HARD_CAP_DEFAULT: usize = 1000;
pub const AGE_CUTOFF_DAYS_DEFAULT: u64 = 30;
pub const SECONDS_PER_DAY: u64 = 24 * 60 * 60;

/// One payload tempfile the caller has already identified as a
/// `capture_v1-*.json` under the payload dir. `created_at_unix` is expected
/// to be the file's birth time (or mtime fallback) in seconds since the
/// Unix epoch.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PayloadListing {
    pub path: PathBuf,
    pub created_at_unix: u64,
}

/// Retention configuration. Defaults match Oracle iter 004 numbers; callers
/// can override per-run but the retention module itself never mutates these.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RetentionConfig {
    pub keep_newest: usize,
    pub hard_cap: usize,
    pub age_cutoff_days: u64,
}

impl Default for RetentionConfig {
    fn default() -> Self {
        Self {
            keep_newest: KEEP_NEWEST_DEFAULT,
            hard_cap: HARD_CAP_DEFAULT,
            age_cutoff_days: AGE_CUTOFF_DAYS_DEFAULT,
        }
    }
}

/// Output of [`plan_retention`]. Both vectors are ordered newest-first so
/// callers can stream writes (or log previews) without re-sorting.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct RetentionPlan {
    pub keep: Vec<PathBuf>,
    pub delete: Vec<PathBuf>,
}

impl RetentionPlan {
    pub fn is_empty(&self) -> bool {
        self.keep.is_empty() && self.delete.is_empty()
    }
}

/// Plan the retention pass for a payload-tempfile listing.
///
/// The listing does not need to be sorted — the planner sorts a local copy
/// by `created_at_unix` descending (ties break by path ascending so the
/// plan is deterministic for identical inputs). The planner then:
///
/// 1. Keeps every file at rank `< cfg.keep_newest`, regardless of age.
/// 2. For files at rank `>= cfg.keep_newest`:
///    a. Deletes if rank `>= cfg.hard_cap`.
///    b. Otherwise deletes if `now_unix - created_at_unix` exceeds
///    `cfg.age_cutoff_days * SECONDS_PER_DAY`.
///    c. Keeps the file otherwise.
///
/// `now_unix` is supplied by the caller so tests can run with a fixed clock.
pub fn plan_retention(
    listing: &[PayloadListing],
    now_unix: u64,
    cfg: &RetentionConfig,
) -> RetentionPlan {
    let mut ordered: Vec<&PayloadListing> = listing.iter().collect();
    ordered.sort_by(|a, b| {
        b.created_at_unix
            .cmp(&a.created_at_unix)
            .then_with(|| a.path.cmp(&b.path))
    });

    let age_cutoff_secs = cfg.age_cutoff_days.saturating_mul(SECONDS_PER_DAY);
    let mut plan = RetentionPlan::default();

    for (rank, entry) in ordered.into_iter().enumerate() {
        let mut should_delete = false;
        if rank < cfg.keep_newest {
            // Newest-keep floor: always preserved, regardless of age or cap.
        } else if rank >= cfg.hard_cap {
            should_delete = true;
        } else {
            let age_secs = now_unix.saturating_sub(entry.created_at_unix);
            if age_secs > age_cutoff_secs {
                should_delete = true;
            }
        }

        if should_delete {
            plan.delete.push(entry.path.clone());
        } else {
            plan.keep.push(entry.path.clone());
        }
    }

    plan
}

/// Result of applying a plan. The filesystem helper returns per-path success
/// so callers can surface warnings without failing the whole pass.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct AppliedRetention {
    pub deleted: Vec<PathBuf>,
    pub failed: Vec<(PathBuf, String)>,
}

/// Apply a retention plan to the filesystem. This helper is intentionally
/// thin — the planning function above is where the policy lives. Unknown
/// paths (missing because something else already deleted them) are treated
/// as a successful no-op so concurrent passes don't spuriously fail.
pub fn apply_retention_plan(plan: &RetentionPlan) -> AppliedRetention {
    let mut applied = AppliedRetention::default();
    for path in &plan.delete {
        match std::fs::remove_file(path) {
            Ok(()) => applied.deleted.push(path.clone()),
            Err(err) if err.kind() == std::io::ErrorKind::NotFound => {
                applied.deleted.push(path.clone());
            }
            Err(err) => applied.failed.push((path.clone(), err.to_string())),
        }
    }
    applied
}

#[cfg(test)]
mod tests {
    use super::*;

    fn listing(entries: &[(&str, u64)]) -> Vec<PayloadListing> {
        entries
            .iter()
            .map(|(name, ts)| PayloadListing {
                path: PathBuf::from(format!("/tmp/payloads/{name}")),
                created_at_unix: *ts,
            })
            .collect()
    }

    fn cfg() -> RetentionConfig {
        RetentionConfig::default()
    }

    const NOW: u64 = 10_000_000;
    const ONE_DAY: u64 = SECONDS_PER_DAY;

    #[test]
    fn empty_listing_yields_empty_plan() {
        let plan = plan_retention(&[], NOW, &cfg());
        assert!(plan.keep.is_empty());
        assert!(plan.delete.is_empty());
        assert!(plan.is_empty());
    }

    #[test]
    fn listing_under_keep_newest_keeps_everything_regardless_of_age() {
        // 50 files, all 200 days old. All under the newest-250 floor → keep.
        let ancient = NOW.saturating_sub(200 * ONE_DAY);
        let entries: Vec<(String, u64)> = (0..50)
            .map(|i| (format!("capture_v1-{i:03}.json"), ancient))
            .collect();
        let entries_ref: Vec<(&str, u64)> = entries.iter().map(|(n, t)| (n.as_str(), *t)).collect();
        let list = listing(&entries_ref);
        let plan = plan_retention(&list, NOW, &cfg());
        assert_eq!(plan.keep.len(), 50);
        assert!(
            plan.delete.is_empty(),
            "newest-{} floor must preserve ancient files too",
            cfg().keep_newest
        );
    }

    #[test]
    fn age_rule_only_triggers_outside_newest_floor() {
        // Mix: 250 young + 10 old. All 260 must be kept (young ones by age,
        // old ones by floor? no — old ones are RANKED after young ones, so
        // they sit at rank >= 250. That IS outside the floor, so age rule
        // applies. Expect 10 deleted.).
        let young = NOW.saturating_sub(1 * ONE_DAY);
        let old = NOW.saturating_sub(90 * ONE_DAY);
        let mut entries: Vec<(String, u64)> = (0..250)
            .map(|i| (format!("young-{i:03}.json"), young))
            .collect();
        entries.extend((0..10).map(|i| (format!("old-{i:03}.json"), old)));
        let entries_ref: Vec<(&str, u64)> = entries.iter().map(|(n, t)| (n.as_str(), *t)).collect();
        let list = listing(&entries_ref);
        let plan = plan_retention(&list, NOW, &cfg());
        assert_eq!(plan.keep.len(), 250, "newest-250 survive");
        assert_eq!(plan.delete.len(), 10, "aged entries outside floor deleted");
        for path in &plan.delete {
            assert!(
                path.to_string_lossy().contains("old-"),
                "deletion targeted the old entries, got {path:?}"
            );
        }
    }

    #[test]
    fn old_entries_inside_newest_floor_are_still_kept() {
        // Only 50 files total, all 100 days old. Under the newest-250 floor,
        // the age rule must NOT fire. This is the invariant regression guard.
        let ancient = NOW.saturating_sub(100 * ONE_DAY);
        let entries: Vec<(String, u64)> = (0..50)
            .map(|i| (format!("ancient-{i:03}.json"), ancient))
            .collect();
        let entries_ref: Vec<(&str, u64)> = entries.iter().map(|(n, t)| (n.as_str(), *t)).collect();
        let list = listing(&entries_ref);
        let plan = plan_retention(&list, NOW, &cfg());
        assert_eq!(plan.keep.len(), 50);
        assert!(plan.delete.is_empty());
    }

    #[test]
    fn hard_cap_trims_excess_even_when_young() {
        // 1050 files, all young (1 day old). 1000 kept, 50 deleted strictly
        // by hard-cap ranking — age rule doesn't apply.
        let young = NOW.saturating_sub(ONE_DAY);
        let entries: Vec<(String, u64)> = (0..1050)
            .map(|i| {
                // Stagger timestamps by 1 sec so rank is deterministic.
                let ts = young.saturating_add((1050 - i) as u64);
                (format!("young-{i:04}.json"), ts)
            })
            .collect();
        let entries_ref: Vec<(&str, u64)> = entries.iter().map(|(n, t)| (n.as_str(), *t)).collect();
        let list = listing(&entries_ref);
        let plan = plan_retention(&list, NOW, &cfg());
        assert_eq!(plan.keep.len(), 1000);
        assert_eq!(plan.delete.len(), 50);
    }

    #[test]
    fn newest_250_invariant_never_violated_even_when_all_old() {
        // 300 files, all 90 days old. Newest-250 floor forces 250 kept;
        // age rule deletes the remaining 50 outside the floor.
        let old = NOW.saturating_sub(90 * ONE_DAY);
        let entries: Vec<(String, u64)> = (0..300)
            .map(|i| {
                // Stagger by 1s so rank is deterministic.
                let ts = old.saturating_add((300 - i) as u64);
                (format!("old-{i:03}.json"), ts)
            })
            .collect();
        let entries_ref: Vec<(&str, u64)> = entries.iter().map(|(n, t)| (n.as_str(), *t)).collect();
        let list = listing(&entries_ref);
        let plan = plan_retention(&list, NOW, &cfg());
        assert_eq!(plan.keep.len(), 250, "floor is inviolable");
        assert_eq!(plan.delete.len(), 50);
    }

    #[test]
    fn age_rule_cutoff_is_strictly_greater_than() {
        // File exactly at the 30-day boundary should be KEPT (age not >).
        let at_cutoff = NOW.saturating_sub(30 * ONE_DAY);
        // Place it outside the newest floor so the age rule actually runs.
        let mut entries: Vec<(String, u64)> = (0..KEEP_NEWEST_DEFAULT)
            .map(|i| (format!("young-{i:03}.json"), NOW - 60))
            .collect();
        entries.push(("boundary.json".into(), at_cutoff));
        let entries_ref: Vec<(&str, u64)> = entries.iter().map(|(n, t)| (n.as_str(), *t)).collect();
        let list = listing(&entries_ref);
        let plan = plan_retention(&list, NOW, &cfg());
        assert!(
            plan.keep.iter().any(|p| p.ends_with("boundary.json")),
            "boundary file kept when age == cutoff"
        );
        assert!(plan.delete.is_empty());
    }

    #[test]
    fn plan_is_deterministic_for_identical_inputs() {
        // Two listings with identical (ts, path) pairs but in different orders
        // must produce identical plans.
        let young = NOW.saturating_sub(ONE_DAY);
        let old = NOW.saturating_sub(100 * ONE_DAY);
        let a = vec![
            PayloadListing {
                path: "/tmp/payloads/a.json".into(),
                created_at_unix: young,
            },
            PayloadListing {
                path: "/tmp/payloads/b.json".into(),
                created_at_unix: old,
            },
        ];
        let b = vec![a[1].clone(), a[0].clone()];
        let plan_a = plan_retention(&a, NOW, &cfg());
        let plan_b = plan_retention(&b, NOW, &cfg());
        assert_eq!(plan_a, plan_b);
    }

    #[test]
    fn tie_break_on_timestamp_uses_path_ordering() {
        // Same timestamp, different names — rank must be deterministic.
        let ts = NOW.saturating_sub(100 * ONE_DAY);
        let entries = vec![
            PayloadListing {
                path: "/tmp/payloads/z.json".into(),
                created_at_unix: ts,
            },
            PayloadListing {
                path: "/tmp/payloads/a.json".into(),
                created_at_unix: ts,
            },
            PayloadListing {
                path: "/tmp/payloads/m.json".into(),
                created_at_unix: ts,
            },
        ];
        let cfg_tight = RetentionConfig {
            keep_newest: 1,
            hard_cap: 2,
            age_cutoff_days: 30,
        };
        let plan = plan_retention(&entries, NOW, &cfg_tight);
        // Newest-1 kept → path "a.json" (sorts ascending). Next comes m.json,
        // inside hard cap but age > 30d → deleted. z.json is at rank >=
        // hard_cap → deleted.
        assert_eq!(plan.keep.len(), 1);
        assert!(plan.keep[0].ends_with("a.json"));
        assert_eq!(plan.delete.len(), 2);
    }

    #[test]
    fn disable_hard_cap_by_setting_config_enormous() {
        // Regression guard against drift: the policy only deletes when a
        // rule fires. A config with huge caps + huge age cutoff MUST keep
        // everything.
        let ancient = NOW.saturating_sub(5000 * ONE_DAY);
        let entries: Vec<(String, u64)> = (0..2000)
            .map(|i| (format!("x-{i:04}.json"), ancient + i as u64))
            .collect();
        let entries_ref: Vec<(&str, u64)> = entries.iter().map(|(n, t)| (n.as_str(), *t)).collect();
        let list = listing(&entries_ref);
        let cfg_huge = RetentionConfig {
            keep_newest: usize::MAX,
            hard_cap: usize::MAX,
            age_cutoff_days: 100_000,
        };
        let plan = plan_retention(&list, NOW, &cfg_huge);
        assert_eq!(plan.keep.len(), 2000);
        assert!(plan.delete.is_empty());
    }

    #[test]
    fn keep_order_is_newest_first() {
        let t = NOW;
        let entries = vec![
            PayloadListing {
                path: "/tmp/payloads/old.json".into(),
                created_at_unix: t - 10,
            },
            PayloadListing {
                path: "/tmp/payloads/new.json".into(),
                created_at_unix: t,
            },
            PayloadListing {
                path: "/tmp/payloads/mid.json".into(),
                created_at_unix: t - 5,
            },
        ];
        let plan = plan_retention(&entries, NOW, &cfg());
        assert_eq!(plan.keep.len(), 3);
        assert!(plan.keep[0].ends_with("new.json"));
        assert!(plan.keep[1].ends_with("mid.json"));
        assert!(plan.keep[2].ends_with("old.json"));
    }

    #[test]
    fn default_constants_match_oracle_iter_004_numbers() {
        assert_eq!(KEEP_NEWEST_DEFAULT, 250);
        assert_eq!(HARD_CAP_DEFAULT, 1000);
        assert_eq!(AGE_CUTOFF_DAYS_DEFAULT, 30);
        let default = RetentionConfig::default();
        assert_eq!(default.keep_newest, 250);
        assert_eq!(default.hard_cap, 1000);
        assert_eq!(default.age_cutoff_days, 30);
    }

    #[test]
    fn apply_retention_plan_treats_missing_paths_as_success() {
        let tmp = tempfile::TempDir::new().expect("tempdir");
        let present = tmp.path().join("present.json");
        std::fs::write(&present, "{}").unwrap();
        let missing = tmp.path().join("already-gone.json");
        let plan = RetentionPlan {
            keep: Vec::new(),
            delete: vec![present.clone(), missing.clone()],
        };
        let applied = apply_retention_plan(&plan);
        assert_eq!(applied.deleted.len(), 2);
        assert!(applied.failed.is_empty());
        assert!(!present.exists(), "present file was removed");
    }
}
