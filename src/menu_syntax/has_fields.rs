//! Shared catalog of first-class `has:<field>` predicates used by the
//! ScriptList advanced-query layer. Matcher, parser, popup, and hint
//! surfaces all read from `HAS_FIELD_SPECS` so the field list cannot drift.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HasFieldOwner {
    Script,
    Scriptlet,
}

#[derive(Debug, Clone, Copy)]
pub struct HasFieldSpec {
    pub canonical: &'static str,
    /// Pre-rendered `has:<canonical>` token used by completion rows and tests.
    pub token: &'static str,
    pub aliases: &'static [&'static str],
    pub label: &'static str,
    pub user_label: &'static str,
    pub subtitle: Option<&'static str>,
    pub detail: Option<&'static str>,
    pub owners: &'static [HasFieldOwner],
}

pub const HAS_FIELD_SPECS: &[HasFieldSpec] = &[
    HasFieldSpec {
        canonical: "shortcut",
        token: "has:shortcut",
        aliases: &[],
        label: "Has shortcut",
        user_label: "has shortcut",
        subtitle: Some("Scripts or scriptlets with a keyboard shortcut"),
        detail: None,
        owners: &[HasFieldOwner::Script, HasFieldOwner::Scriptlet],
    },
    HasFieldSpec {
        canonical: "alias",
        token: "has:alias",
        aliases: &[],
        label: "Has alias",
        user_label: "has alias",
        subtitle: Some("Scripts or scriptlets with an alias"),
        detail: None,
        owners: &[HasFieldOwner::Script, HasFieldOwner::Scriptlet],
    },
    HasFieldSpec {
        canonical: "description",
        token: "has:description",
        aliases: &["desc"],
        label: "Has description",
        user_label: "has description",
        subtitle: Some("Scripts or scriptlets with a description"),
        detail: None,
        owners: &[HasFieldOwner::Script, HasFieldOwner::Scriptlet],
    },
    HasFieldSpec {
        canonical: "icon",
        token: "has:icon",
        aliases: &[],
        label: "Has icon",
        user_label: "has icon",
        subtitle: Some("Scripts with an icon"),
        detail: None,
        owners: &[HasFieldOwner::Script],
    },
    HasFieldSpec {
        canonical: "schema",
        token: "has:schema",
        aliases: &[],
        label: "Has schema",
        user_label: "has schema",
        subtitle: Some("Scripts that declare a schema"),
        detail: None,
        owners: &[HasFieldOwner::Script],
    },
    HasFieldSpec {
        canonical: "menuSyntax",
        token: "has:menuSyntax",
        aliases: &["menusyntax", "menu_syntax", "menu-syntax", "menu syntax"],
        label: "Has menuSyntax metadata",
        user_label: "has menuSyntax metadata",
        subtitle: Some("Scripts that opt in to menu-syntax handlers"),
        detail: None,
        owners: &[HasFieldOwner::Script],
    },
    HasFieldSpec {
        canonical: "keyword",
        token: "has:keyword",
        aliases: &[],
        label: "Has keyword",
        user_label: "has keyword",
        subtitle: Some("Scriptlets with a keyword"),
        detail: None,
        owners: &[HasFieldOwner::Scriptlet],
    },
    HasFieldSpec {
        canonical: "group",
        token: "has:group",
        aliases: &[],
        label: "Has group",
        user_label: "has group",
        subtitle: Some("Scriptlets with a group"),
        detail: None,
        owners: &[HasFieldOwner::Scriptlet],
    },
    HasFieldSpec {
        canonical: "command",
        token: "has:command",
        aliases: &[],
        label: "Has command",
        user_label: "has command",
        subtitle: Some("Scriptlets with a command"),
        detail: None,
        owners: &[HasFieldOwner::Scriptlet],
    },
];

/// Look up a `HasFieldSpec` by canonical name or alias, case-insensitively.
/// Whitespace-only and empty probes return `None`.
pub fn lookup_has_field(probe: &str) -> Option<&'static HasFieldSpec> {
    let trimmed = probe.trim();
    if trimmed.is_empty() {
        return None;
    }
    let lower = trimmed.to_ascii_lowercase();
    HAS_FIELD_SPECS.iter().find(|spec| {
        spec.canonical.eq_ignore_ascii_case(&lower)
            || spec
                .aliases
                .iter()
                .any(|alias| alias.eq_ignore_ascii_case(&lower))
    })
}

/// Canonicalize an incoming `has:<field>` value, returning the catalog
/// canonical spelling when one matches. Unknown but syntactically valid
/// probes are returned trimmed and unchanged so the matcher can still
/// fall back to script `metadata.extra` keys.
///
/// Returns `None` when the probe is empty or, when `reject_unknown_whitespace`
/// is set, when the probe contains internal whitespace and does not match a
/// known alias.
pub fn canonical_has_field_value(probe: &str, reject_unknown_whitespace: bool) -> Option<String> {
    let trimmed = probe.trim();
    if trimmed.is_empty() {
        return None;
    }
    if let Some(spec) = lookup_has_field(trimmed) {
        return Some(spec.canonical.to_string());
    }
    if reject_unknown_whitespace && trimmed.chars().any(|c| c.is_whitespace()) {
        return None;
    }
    Some(trimmed.to_string())
}

pub fn iter_has_fields_for_owner(
    owner: HasFieldOwner,
) -> impl Iterator<Item = &'static HasFieldSpec> {
    HAS_FIELD_SPECS
        .iter()
        .filter(move |spec| spec.owners.contains(&owner))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lookup_resolves_aliases_case_insensitively() {
        for probe in [
            "menuSyntax",
            "menusyntax",
            "MENU_SYNTAX",
            "menu-syntax",
            "menu syntax",
        ] {
            let spec = lookup_has_field(probe).expect(probe);
            assert_eq!(spec.canonical, "menuSyntax");
        }
    }

    #[test]
    fn description_alias_resolves_to_canonical() {
        assert_eq!(
            lookup_has_field("desc").map(|s| s.canonical),
            Some("description")
        );
        assert_eq!(
            lookup_has_field("Description").map(|s| s.canonical),
            Some("description"),
        );
    }

    #[test]
    fn canonical_has_field_value_rejects_empty() {
        assert!(canonical_has_field_value("", true).is_none());
        assert!(canonical_has_field_value("   ", true).is_none());
    }

    #[test]
    fn canonical_has_field_value_rejects_unknown_whitespace() {
        assert!(canonical_has_field_value("not a field", true).is_none());
        assert_eq!(
            canonical_has_field_value("not a field", false),
            Some("not a field".to_string()),
        );
    }

    #[test]
    fn canonical_has_field_value_canonicalizes_known_aliases() {
        assert_eq!(
            canonical_has_field_value("menu syntax", true),
            Some("menuSyntax".to_string()),
        );
        assert_eq!(
            canonical_has_field_value("DESC", true),
            Some("description".to_string()),
        );
    }

    #[test]
    fn canonical_has_field_value_preserves_unknown_extra_keys() {
        assert_eq!(
            canonical_has_field_value("fooBar", true),
            Some("fooBar".to_string()),
        );
    }
}
