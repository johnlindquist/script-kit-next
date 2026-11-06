//! What still needs setting up leads the empty list under its own header, never a banner above the
//! line: the input stays where it is. It leads above everything else on the empty list, the
//! launcher's subject tabs included, until it's done or the user says to go on without it.

use crate::contract::ledger_words;
use crate::row::Row;

/// Every setup row's key starts with this.
pub const PREFIX: &str = "setup:";

/// The key of the row that installs what's missing.
pub const BUN_KEY: &str = "setup:bun";

/// A permission row's key: this, then the permission's id.
pub const PERMISSION_PREFIX: &str = "setup:permission/";

/// The row that goes on without the permissions still missing.
pub const CONTINUE_KEY: &str = "setup:permissions-continue";

/// How much the app depends on a permission.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Need {
    Required,
    Recommended,
    Optional,
}

impl Need {
    fn words(self) -> &'static str {
        match self {
            Need::Required => "Required",
            Need::Recommended => "Recommended",
            Need::Optional => "Optional",
        }
    }
}

/// A macOS permission as a setup row shows it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Permission {
    /// Stable id, the end of the row's key.
    pub id: &'static str,
    pub name: &'static str,
    pub icon: &'static str,
    pub need: Need,
    /// None means the OS did not report a status. Never display it as granted.
    pub granted: Option<bool>,
    /// What it's for, or what stops working while it's off.
    pub about: String,
}

/// Something the app needs before everything can run.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Missing {
    Bun,
    Permission(Permission),
}

/// Whether a row is one of the setup rows.
pub fn is_setup(key: &str) -> bool {
    key.starts_with(PREFIX)
}

/// The permission a row grants, by id.
pub fn permission_id(key: &str) -> Option<&str> {
    key.strip_prefix(PERMISSION_PREFIX)
}

/// Rows for what's missing, the first under the "Needs setup" header. When a permission is
/// still missing, the last row goes on without it.
pub fn needs_setup_rows(missing: &[Missing]) -> Vec<Row> {
    let mut rows: Vec<Row> = missing
        .iter()
        .map(|m| match m {
            Missing::Bun => Row::new(BUN_KEY, ledger_words::BUN_MISSING)
                .with_sub(ledger_words::BUN_INSTALL)
                .with_enter_label(ledger_words::BUN_INSTALL)
                .with_icon("download"),
            Missing::Permission(p) if p.granted.is_none() => {
                Row::new(format!("{PERMISSION_PREFIX}{}", p.id), p.name)
                    .with_sub(format!("Status unavailable · {}", p.about))
                    .with_enter_label("Check status")
                    .with_icon(p.icon)
            }
            Missing::Permission(p) if p.granted == Some(true) => {
                Row::new(format!("{PERMISSION_PREFIX}{}", p.id), p.name)
                    .with_sub(format!("Granted · {}", p.about))
                    .with_enter_label(format!("{} is on", p.name))
                    .with_icon(p.icon)
            }
            Missing::Permission(p) => Row::new(format!("{PERMISSION_PREFIX}{}", p.id), p.name)
                .with_sub(format!("{} · {}", p.need.words(), p.about))
                .with_enter_label(format!("Allow {}", p.name))
                .with_icon(p.icon),
        })
        .collect();
    if missing
        .iter()
        .any(|m| matches!(m, Missing::Permission(p) if p.granted != Some(true)))
    {
        rows.push(
            Row::new(CONTINUE_KEY, ledger_words::PERMISSIONS_CONTINUE)
                .with_sub(ledger_words::PERMISSIONS_CONTINUE_SUB)
                .with_enter_label(ledger_words::PERMISSIONS_CONTINUE)
                .with_icon("arrow-right"),
        );
    }
    if let Some(first) = rows.first_mut() {
        first.head = Some(ledger_words::NEEDS_SETUP.to_string());
    }
    rows
}

#[cfg(test)]
mod tests {
    use super::*;

    fn accessibility(granted: bool) -> Missing {
        Missing::Permission(Permission {
            id: "accessibility",
            name: "Accessibility",
            icon: "accessibility",
            need: Need::Required,
            granted: Some(granted),
            about: "Snippets and pasting".into(),
        })
    }

    #[test]
    fn a_missing_bun_leads_with_needs_setup() {
        let rows = needs_setup_rows(&[Missing::Bun]);
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].head.as_deref(), Some(ledger_words::NEEDS_SETUP));
        assert_eq!(rows[0].title, "bun is not installed");
        assert_eq!(rows[0].enter_words(), "Install from bun.sh");
        assert!(needs_setup_rows(&[]).is_empty());
    }

    #[test]
    fn a_missing_permission_offers_to_allow_it_and_to_go_on_without_it() {
        let rows = needs_setup_rows(&[Missing::Bun, accessibility(false)]);
        let keys: Vec<&str> = rows.iter().map(|r| r.key.as_str()).collect();
        assert_eq!(
            keys,
            [BUN_KEY, "setup:permission/accessibility", CONTINUE_KEY]
        );
        assert_eq!(rows[0].head.as_deref(), Some(ledger_words::NEEDS_SETUP));
        assert!(rows[1..].iter().all(|r| r.head.is_none()));
        assert_eq!(rows[1].enter_words(), "Allow Accessibility");
        assert_eq!(
            rows[1].sub.as_deref(),
            Some("Required · Snippets and pasting")
        );
        assert_eq!(permission_id(&rows[1].key), Some("accessibility"));
        assert!(rows.iter().all(|r| is_setup(&r.key)));
    }

    #[test]
    fn with_every_permission_on_there_is_nothing_to_go_on_without() {
        let rows = needs_setup_rows(&[accessibility(true)]);
        assert_eq!(rows.len(), 1);
        assert!(rows[0].sub.as_deref().unwrap().starts_with("Granted"));
    }
}
