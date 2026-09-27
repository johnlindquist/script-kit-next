use super::fixture_ids::DAY_PAGE_FIXTURE_IDS;
use anyhow::{Context as _, Result};
use gpui::{AnyWindowHandle, App, AppContext, Entity, Window};

use crate::runtime_policy::{owned_evaluation, WindowHostPolicy};

fn fixture_now() -> Result<chrono::DateTime<chrono::Utc>> {
    Ok(chrono::DateTime::parse_from_rfc3339("2026-08-28T12:00:00Z")?.with_timezone(&chrono::Utc))
}

/// A note longer than the main window's list area, so reading it scrolls under the input.
const LAUNCH_PLAN: &str = "# Launch plan\n\n- Freeze the beta branch on Monday\n- Write the release notes draft\n- Ask Sarah to review the pricing page\n- Record the one-minute demo video\n- Update the changelog with every fix\n- Check the onboarding emails render in dark mode\n- Book the launch review with Mo and Priya\n- Prepare the support macros for common questions\n- Test the installer on a clean Mac\n- Translate the landing page headline\n- Schedule the social posts for Thursday\n- Confirm the status page is ready\n- Send the press kit to the three outlets\n- Turn on the download counter\n- Celebrate with the team on Friday\n";

/// Seeds canonical markdown + the real SQLite index, never an in-memory stand-in.
/// Existing documents survive reopen so save/reload remains observable.
pub(crate) fn prepare_notes_storage() -> Result<()> {
    let policy = owned_evaluation().context("Notes fixtures require an owned evaluation")?;
    policy.require_owned_path(&policy.root().join("notes"))?;
    crate::notes::init_notes_db()?;
    let now = fixture_now()?;
    for (id, title, content) in [
        ("d0197594-1111-4000-8000-000000000001", "Fixture Alpha", "# Fixture Alpha\n\n- [ ] Review the document\n- [x] Keep the completed task\n\n[Reference](https://example.invalid/reference)\n"),
        ("d0197594-1111-4000-8000-000000000002", "Fixture Beta", "# Fixture Beta\n\nSecond document for search and switcher.\n"),
        ("d0197594-1111-4000-8000-000000000003", "Launch plan", LAUNCH_PLAN),
    ] {
        let id = crate::notes::NoteId::parse(id).context("invalid fixture note identity")?;
        if crate::notes::get_note(id)?.is_none() {
            crate::notes::save_note(&crate::notes::Note {
                id, title: title.into(), content: content.into(), created_at: now,
                updated_at: now, deleted_at: None, is_pinned: title == "Fixture Alpha", sort_order: 0,
            })?;
            crate::runtime_policy::record_completed_fixture_effect();
        }
    }
    // Both recent switchers search the canonical Notes/day corpus. Seed the
    // same actual Day document used by the Day root before either host opens.
    let substrate = script_kit_gpui::brain::substrate::BrainSubstrate::default_kit();
    policy.require_owned_path(substrate.paths().base())?;
    let mut session = script_kit_gpui::day_page::DayPageDocumentSession::new(substrate.clone());
    session.bind_today(now)?;
    if session.disk_content().is_empty() {
        session.apply_editor_content("# Today\n\n- [ ] Review the document\n\n09:15 [Fixture fragment](../fragments/fixture-fragment.md)\n09:20 [Clipboard entry](kit://clipboard-history?id=fixture-entry)\n");
        session.save(now)?;
    }
    let fragment = substrate
        .paths()
        .fragments_dir()
        .join("fixture-fragment.md");
    policy.require_owned_path(&fragment)?;
    if !fragment.exists() {
        crate::atomic_file::ensure_private_directory(&substrate.paths().fragments_dir())?;
        crate::brain::substrate::io::atomic_write(
            &fragment,
            "# Fixture fragment\n\nA source-line return target.\n",
        )?;
        crate::runtime_policy::record_completed_fixture_effect();
    }
    Ok(())
}

/// DayPageView remains a child of the actual ScriptListApp; Main installs the
/// returned entity in AppView::DayPage, retaining the real Day footer owner.
pub(crate) fn create_day_page_fixture(
    fixture_id: &str,
    app: Entity<crate::ScriptListApp>,
    window: &mut Window,
    cx: &mut App,
) -> Result<Entity<crate::DayPageView>> {
    anyhow::ensure!(
        DAY_PAGE_FIXTURE_IDS.contains(&fixture_id),
        "unknown_day_page_fixture"
    );
    prepare_notes_storage()?;
    let substrate = script_kit_gpui::brain::substrate::BrainSubstrate::default_kit();
    let now = fixture_now()?;
    let mut session = script_kit_gpui::day_page::DayPageDocumentSession::new(substrate.clone());
    session.bind_today(now)?;
    let fragment = substrate
        .paths()
        .fragments_dir()
        .join("fixture-fragment.md");
    if fixture_id == "day-page.fragment" {
        session.bind_fragment(fragment, now)?;
    }
    let initial = crate::DayPageInitialData {
        session,
        now: Some(now),
        shelf_previews: Some(std::collections::HashMap::from([(
            "fixture-entry".into(),
            "Owned clipboard shelf fixture".into(),
        )])),
        host_policy: WindowHostPolicy::OwnedHidden,
    };
    Ok(cx.new(|cx| {
        let mut view = crate::DayPageView::from_initial_data(app, initial, window, cx);
        view.clipboard_shelf_expanded = fixture_id == "day-page.shelf";
        view
    }))
}

fn require_fixture_parent(
    parent: &crate::protocol::AutomationWindowInfo,
    handle: AnyWindowHandle,
) -> Result<u64> {
    WindowHostPolicy::OwnedHidden.validate()?;
    let generation = parent
        .generation
        .filter(|generation| *generation != 0)
        .context("fixture_parent_generation_missing")?;
    anyhow::ensure!(
        crate::windows::get_runtime_window_handle_for_generation(&parent.id, generation)
            == Some(handle),
        "fixture_parent_stale"
    );
    anyhow::ensure!(
        crate::windows::runtime_window_host_policy(&parent.id, generation)?
            == WindowHostPolicy::OwnedHidden,
        "fixture_parent_not_owned"
    );
    Ok(generation)
}

pub(crate) fn mount_day_page_fixture_presentation(
    fixture_id: &str,
    entity: &Entity<crate::DayPageView>,
    parent: &crate::protocol::AutomationWindowInfo,
    parent_handle: AnyWindowHandle,
    cx: &mut App,
) -> Result<crate::protocol::AutomationWindowInfo> {
    anyhow::ensure!(
        fixture_id == "day-page.switcher",
        "unknown_day_page_auxiliary_fixture"
    );
    require_fixture_parent(parent, parent_handle)?;
    let app = entity
        .read(cx)
        .app
        .upgrade()
        .context("day_fixture_app_released")?;
    anyhow::ensure!(
        parent_handle.read(cx, |root: Entity<gpui_component::Root>, cx| root
            .read(cx)
            .view()
            .entity_id()
            == app.entity_id())?,
        "day_fixture_root_mismatch"
    );
    anyhow::ensure!(
        matches!(&app.read(cx).current_view, crate::AppView::DayPage { entity: current } if current.entity_id() == entity.entity_id()),
        "day_fixture_owner_not_installed"
    );
    parent_handle.update(cx, |_, window, cx| {
        entity.update(cx, |day, cx| day.open_day_switcher(window, cx))
    })?;
    anyhow::ensure!(
        entity.read(cx).note_switcher.dialog().is_some(),
        "day_fixture_switcher_not_open"
    );
    Ok(parent.clone())
}
