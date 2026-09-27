#[cfg(test)]
mod launcher_catalog_snapshot_tests {
    use super::*;
    use crate::spine::catalog_subsearch::ContextSubsearchSource;
    use crate::spine::SpineListAction;

    fn script(name: &str) -> Arc<crate::scripts::Script> {
        Arc::new(crate::scripts::Script {
            name: name.to_owned(),
            path: std::path::PathBuf::from(format!("/synthetic/{name}.ts")),
            plugin_id: "main".to_owned(),
            ..crate::scripts::Script::default()
        })
    }

    fn scriptlet(name: &str) -> Arc<crate::scripts::Scriptlet> {
        Arc::new(crate::scripts::Scriptlet {
            name: name.to_owned(),
            description: Some(format!("Synthetic {name}")),
            code: "// synthetic".to_owned(),
            tool: "ts".to_owned(),
            shortcut: None,
            keyword: None,
            group: None,
            plugin_id: "main".to_owned(),
            plugin_title: None,
            file_path: Some(format!("/synthetic/scriptlets.md#{name}")),
            command: Some(name.to_owned()),
            alias: None,
            icon: None,
        })
    }

    fn skill(name: &str) -> Arc<crate::plugins::PluginSkill> {
        Arc::new(crate::plugins::PluginSkill {
            plugin_id: "synthetic".to_owned(),
            plugin_title: "Synthetic".to_owned(),
            skill_id: name.to_owned(),
            path: std::path::PathBuf::from(format!("/synthetic/skills/{name}/SKILL.md")),
            title: name.to_owned(),
            description: format!("Synthetic {name}"),
        })
    }

    #[test]
    fn inline_portals_follow_host_replacements_instead_of_process_lifetime_file_caches() {
        let store = LauncherCatalogStore::default();
        store.publish(
            &[script("old-script")],
            &[scriptlet("old-note")],
            &[skill("old-skill")],
        );
        let previous = store.snapshot();
        store.publish(
            &[script("new-script")],
            &[scriptlet("new-note")],
            &[skill("new-skill")],
        );
        let current = store.snapshot();
        for (kind, old_label, current_label) in [
            (ContextPortalKind::ScriptSearch, "old-script", "new-script"),
            (ContextPortalKind::ScriptletSearch, "old-note", "new-note"),
            (ContextPortalKind::SkillSearch, "old-skill", "new-skill"),
        ] {
            let query = InlinePortalQuery {
                kind,
                prefix: portal_prefix_for_kind(kind),
                query: String::new(),
            };
            let mut old_rows = Vec::new();
            collect_script_list_inline_items_from_snapshot(&query, &mut old_rows, &previous);
            assert_eq!(old_rows.len(), 1);
            assert_eq!(old_rows[0].label.as_ref(), old_label);
            let mut rows = Vec::new();
            collect_script_list_inline_items_from_snapshot(&query, &mut rows, &current);
            assert_eq!(rows.len(), 1);
            assert_eq!(rows[0].label.as_ref(), current_label);
        }
    }

    #[test]
    fn launcher_catalog_ranking_keeps_existing_canonical_slash_tie_break() {
        let forward =
            slash_command_rows_with_descriptions("", [("zeta", "last"), ("alpha", "first")]);
        let reverse =
            slash_command_rows_with_descriptions("", [("alpha", "first"), ("zeta", "last")]);

        let forward_ids: Vec<&str> = forward.iter().map(|row| row.id.as_ref()).collect();
        let reverse_ids: Vec<&str> = reverse.iter().map(|row| row.id.as_ref()).collect();
        assert_eq!(forward_ids, reverse_ids);
        assert_eq!(forward[0].label.as_ref(), "alpha");
        assert_eq!(forward[1].label.as_ref(), "zeta");
    }
}
