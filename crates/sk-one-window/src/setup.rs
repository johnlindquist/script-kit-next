//! What still needs setting up leads the empty list under its own header, never a banner above the
//! line: the input stays where it is.

use crate::contract::ledger_words;
use crate::row::Row;

/// Something the app needs before scripts can run.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Missing {
    Bun,
}

/// The key of the row that installs what's missing.
pub const BUN_KEY: &str = "setup:bun";

/// Rows for what's missing, the first under the "Needs setup" header.
pub fn needs_setup_rows(missing: &[Missing]) -> Vec<Row> {
    missing
        .iter()
        .enumerate()
        .map(|(i, m)| {
            let mut row = match m {
                Missing::Bun => Row::new(BUN_KEY, ledger_words::BUN_MISSING)
                    .with_sub(ledger_words::BUN_INSTALL)
                    .with_enter_label(ledger_words::BUN_INSTALL)
                    .with_icon("download"),
            };
            if i == 0 {
                row.head = Some(ledger_words::NEEDS_SETUP.to_string());
            }
            row
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_missing_bun_leads_with_needs_setup() {
        let rows = needs_setup_rows(&[Missing::Bun]);
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].head.as_deref(), Some(ledger_words::NEEDS_SETUP));
        assert_eq!(rows[0].title, "bun is not installed");
        assert_eq!(rows[0].enter_words(), "Install from bun.sh");
        assert!(needs_setup_rows(&[]).is_empty());
    }
}
