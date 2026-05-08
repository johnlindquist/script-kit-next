use std::hint::black_box;
use std::time::Instant;

use gpui::{px, ListAlignment, ListState};

const SAMPLES: usize = 240;
const WARMUP: usize = 30;
const VISIBLE_ROWS: usize = 22;
const HISTORY_FILTERS: [&str; 17] = [
    "git",
    "github",
    "gh issue",
    "open",
    "deploy",
    "todo",
    "note",
    "gr",
    "grep",
    "settings",
    "window",
    "clipboard",
    "zzz-no-match-001",
    ":type:script git",
    ":shortcut:cmd+k",
    ";todo ",
    "2 + 2",
];

#[derive(Debug, Default, Clone)]
pub(crate) struct MainMenuHistoryRenderBenchSample {
    pub total_ms: f64,
    pub list_sync_ms: f64,
    pub visible_rows_ms: f64,
    pub grouped_item_count: usize,
}

#[derive(Debug, Default)]
pub(crate) struct MainMenuHistoryRenderBenchReport {
    pub samples: usize,
    pub total_p50_ms: f64,
    pub total_p95_ms: f64,
    pub total_max_ms: f64,
    pub list_sync_p95_ms: f64,
    pub visible_rows_p95_ms: f64,
    pub list_state_replacement_count: usize,
    pub list_state_measure_all_count: usize,
}

pub(crate) fn run_main_menu_history_render_prep_benchmark() -> MainMenuHistoryRenderBenchReport {
    let mut samples = Vec::with_capacity(SAMPLES);
    let mut list_state_replacement_count = 0;

    for ix in 0..(SAMPLES + WARMUP) {
        let filter = HISTORY_FILTERS[ix % HISTORY_FILTERS.len()];
        let item_count = synthetic_grouped_item_count(ix, filter);
        let row_generation = ix as u64 + 1;

        let total_start = Instant::now();

        let list_sync_start = Instant::now();
        let list_state = ListState::new(
            item_count,
            ListAlignment::Top,
            px(crate::list_item::effective_average_item_height_for_scroll()),
        );
        black_box(list_state.item_count());
        list_state_replacement_count += 1;
        let list_sync_ms = elapsed_ms(list_sync_start);

        let visible_rows_start = Instant::now();
        for row_ix in 0..VISIBLE_ROWS.min(item_count) {
            let row_id = if row_ix % 7 == 0 {
                format!("section-header-gen-{row_generation}:{row_ix}")
            } else {
                format!("script-item-gen-{row_generation}:{row_ix}")
            };
            black_box(row_id);
        }
        let visible_rows_ms = elapsed_ms(visible_rows_start);

        let total_ms = elapsed_ms(total_start);

        if ix >= WARMUP {
            samples.push(MainMenuHistoryRenderBenchSample {
                total_ms,
                list_sync_ms,
                visible_rows_ms,
                grouped_item_count: item_count,
            });
        }
    }

    let list_state_measure_all_count = include_str!("../app_navigation/impl_scroll.rs")
        .matches(".measure_all()")
        .count();

    MainMenuHistoryRenderBenchReport {
        samples: samples.len(),
        total_p50_ms: percentile(&samples, |sample| sample.total_ms, 0.50),
        total_p95_ms: percentile(&samples, |sample| sample.total_ms, 0.95),
        total_max_ms: samples
            .iter()
            .map(|sample| sample.total_ms)
            .fold(0.0, f64::max),
        list_sync_p95_ms: percentile(&samples, |sample| sample.list_sync_ms, 0.95),
        visible_rows_p95_ms: percentile(&samples, |sample| sample.visible_rows_ms, 0.95),
        list_state_replacement_count,
        list_state_measure_all_count,
    }
}

fn synthetic_grouped_item_count(ix: usize, filter: &str) -> usize {
    let base = match filter {
        "zzz-no-match-001" => 0,
        "2 + 2" => 2,
        filter if filter.starts_with(';') => 6,
        filter if filter.starts_with(':') => 48,
        _ => 140 + ((ix * 37) % 540),
    };

    black_box(base)
}

fn elapsed_ms(start: Instant) -> f64 {
    start.elapsed().as_secs_f64() * 1000.0
}

fn percentile(
    samples: &[MainMenuHistoryRenderBenchSample],
    value: impl Fn(&MainMenuHistoryRenderBenchSample) -> f64,
    quantile: f64,
) -> f64 {
    if samples.is_empty() {
        return 0.0;
    }

    let mut values: Vec<f64> = samples.iter().map(value).collect();
    values.sort_by(|a, b| a.total_cmp(b));
    let index = ((values.len() - 1) as f64 * quantile).round() as usize;
    values[index]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[ignore = "performance benchmark: run with cargo test --release main_menu_history_render_prep_benchmark -- --ignored --nocapture"]
    fn main_menu_history_render_prep_benchmark() {
        let report = run_main_menu_history_render_prep_benchmark();
        eprintln!("{report:#?}");

        assert!(
            report.total_p95_ms <= 8.0,
            "history recall render-prep p95 regressed: {report:#?}"
        );
        assert!(
            report.visible_rows_p95_ms <= 2.5,
            "visible row identity prep p95 regressed: {report:#?}"
        );
        assert_eq!(
            report.list_state_measure_all_count, 0,
            "history recall must not use ListState::measure_all(): {report:#?}"
        );
    }
}
