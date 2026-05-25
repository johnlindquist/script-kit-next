//! Canonical InfoState guidance language for Storybook.
//!
//! These fixtures are intentionally presenter-only. They let designers compare
//! empty/help/form/setup/permission/recovery/About-style guidance before live
//! product surfaces migrate to the shared InfoState renderer.

use gpui::{div, prelude::*, px, AnyElement};

use crate::components::{
    render_info_state, InfoGuidanceItem, InfoSection, InfoStateDensity, InfoStateLayout,
    InfoStateSpec, InfoStateTone,
};
use crate::storybook::StoryVariant;
use crate::theme::get_cached_theme;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NonListStateShowcaseId {
    Empty,
    Help,
    Form,
    Setup,
    Permission,
    Recovery,
    About,
    Density,
}

impl NonListStateShowcaseId {
    pub const ALL: [Self; 8] = [
        Self::Empty,
        Self::Help,
        Self::Form,
        Self::Setup,
        Self::Permission,
        Self::Recovery,
        Self::About,
        Self::Density,
    ];

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Empty => "empty",
            Self::Help => "help",
            Self::Form => "form",
            Self::Setup => "setup",
            Self::Permission => "permission",
            Self::Recovery => "recovery",
            Self::About => "about",
            Self::Density => "density",
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            Self::Empty => "Empty",
            Self::Help => "Help",
            Self::Form => "Form",
            Self::Setup => "Setup",
            Self::Permission => "Permission",
            Self::Recovery => "Recovery",
            Self::About => "About",
            Self::Density => "Density",
        }
    }

    pub fn description(self) -> &'static str {
        match self {
            Self::Empty => "Centered one-message state with one next move.",
            Self::Help => "Anchored guidance with concise examples and truthful footer hints.",
            Self::Form => "Field-first information hierarchy for prompt and power-user forms.",
            Self::Setup => "Requirement checklist for agent or first-run readiness.",
            Self::Permission => "Plain-language scope and two-action decision card.",
            Self::Recovery => "Calm failure state with retry and escape hatch.",
            Self::About => "Branded product identity without dashboard chrome.",
            Self::Density => "Compact and comfortable density comparison.",
        }
    }

    pub fn from_stable_id(value: &str) -> Option<Self> {
        match value {
            "empty" => Some(Self::Empty),
            "help" => Some(Self::Help),
            "form" => Some(Self::Form),
            "setup" => Some(Self::Setup),
            "permission" => Some(Self::Permission),
            "recovery" => Some(Self::Recovery),
            "about" => Some(Self::About),
            "density" => Some(Self::Density),
            _ => None,
        }
    }
}

pub fn non_list_state_showcase_story_variants() -> Vec<StoryVariant> {
    NonListStateShowcaseId::ALL
        .into_iter()
        .map(|id| {
            StoryVariant::default_named(id.as_str(), id.name())
                .description(id.description())
                .with_prop("surface", "nonListState")
                .with_prop("representation", "presenterFixture")
                .with_prop("layout", id.as_str())
        })
        .collect()
}

pub fn render_non_list_state_showcase_preview(stable_id: &str) -> AnyElement {
    let id =
        NonListStateShowcaseId::from_stable_id(stable_id).unwrap_or(NonListStateShowcaseId::Empty);
    render_showcase_state(id, false)
}

pub fn render_non_list_state_showcase_compare_thumbnail(stable_id: &str) -> AnyElement {
    let id =
        NonListStateShowcaseId::from_stable_id(stable_id).unwrap_or(NonListStateShowcaseId::Empty);
    render_showcase_state(id, true)
}

fn render_showcase_state(id: NonListStateShowcaseId, compact: bool) -> AnyElement {
    if matches!(id, NonListStateShowcaseId::Density) {
        return render_density_state(compact);
    }

    stage(
        id.stage_id(),
        render_info_state(showcase_spec(id), &get_cached_theme()),
        compact,
    )
}

fn stage(id: &'static str, content: impl IntoElement, compact: bool) -> AnyElement {
    let theme = get_cached_theme();
    let palette = crate::components::info_palette(&theme);
    let width = if compact { 440.0 } else { 680.0 };
    let height = if compact { 300.0 } else { 430.0 };

    let panel = div()
        .id(id)
        .w(px(width))
        .h(px(height))
        .rounded(px(10.0))
        .overflow_hidden()
        .border_1()
        .border_color(palette.border)
        .bg(palette.panel)
        .flex()
        .flex_col()
        .child(
            div()
                .flex_1()
                .min_h(px(0.0))
                .w_full()
                .overflow_hidden()
                .child(content),
        );

    div()
        .w_full()
        .min_h(px(if compact { 320.0 } else { 470.0 }))
        .flex()
        .items_center()
        .justify_center()
        .child(panel)
        .into_any_element()
}

impl NonListStateShowcaseId {
    fn stage_id(self) -> &'static str {
        match self {
            Self::Empty => "info-state-empty-stage",
            Self::Help => "info-state-help-stage",
            Self::Form => "info-state-form-stage",
            Self::Setup => "info-state-setup-stage",
            Self::Permission => "info-state-permission-stage",
            Self::Recovery => "info-state-recovery-stage",
            Self::About => "info-state-about-stage",
            Self::Density => "info-state-density-stage",
        }
    }
}

fn showcase_spec(id: NonListStateShowcaseId) -> InfoStateSpec {
    match id {
        NonListStateShowcaseId::Empty => InfoStateSpec::new("info-state-empty")
            .layout(InfoStateLayout::Centered)
            .density(InfoStateDensity::Compact)
            .tone(InfoStateTone::Neutral)
            .title("No scripts yet")
            .body("Create your first script, install one from Kit, or ask Agent Chat to draft the workflow you need.")
            .section(InfoSection::new(vec![
                InfoGuidanceItem::new(Some("⌘N"), "Create a script"),
                InfoGuidanceItem::new(Some("⇥"), "Ask Agent Chat"),
                InfoGuidanceItem::new(Some("⌘K"), "More actions"),
            ])),
        NonListStateShowcaseId::Help => InfoStateSpec::new("info-state-help")
            .layout(InfoStateLayout::AnchoredTop)
            .density(InfoStateDensity::Compact)
            .tone(InfoStateTone::Help)
            .title("Use structured capture")
            .body("Start with ;todo or ;note, then add the details naturally. Script Kit will show fields when the target supports them.")
            .section(InfoSection::titled(
                "Examples",
                vec![
                    InfoGuidanceItem::new(Some(";todo"), "Buy milk #errands tomorrow"),
                    InfoGuidanceItem::new(Some(";note"), "Standup summary #work"),
                    InfoGuidanceItem::new(Some(":#"), "Filter by tag"),
                ],
            ))
            .footer_note("Long documentation belongs in Actions; this state only teaches the next move."),
        NonListStateShowcaseId::Form => InfoStateSpec::new("info-state-form")
            .layout(InfoStateLayout::AnchoredTop)
            .density(InfoStateDensity::Compact)
            .tone(InfoStateTone::Neutral)
            .title("Capture a task")
            .body("Forms keep field intent visible and explain only the current choice.")
            .section(InfoSection::titled(
                "Field rhythm",
                vec![
                    InfoGuidanceItem::new(Some("Title"), "Prepare release checklist")
                        .detail("Required"),
                    InfoGuidanceItem::new(Some("Project"), "Script Kit GPUI")
                        .detail("Autocomplete accepts known projects"),
                    InfoGuidanceItem::new(Some("Tab"), "Move to the next field"),
                ],
            )),
        NonListStateShowcaseId::Setup => InfoStateSpec::new("info-state-setup")
            .layout(InfoStateLayout::AnchoredTop)
            .density(InfoStateDensity::Comfortable)
            .tone(InfoStateTone::Setup)
            .title("Connect an agent")
            .body("Script Kit needs an Agent Chat backend before this window can send messages.")
            .section(InfoSection::titled(
                "Readiness",
                vec![
                    InfoGuidanceItem::new(Some("Agent"), "Binary found"),
                    InfoGuidanceItem::new(Some("Auth"), "Needs sign in"),
                    InfoGuidanceItem::new(Some("Trust"), "Workspace ready"),
                ],
            ))
            .section(InfoSection::new(vec![
                InfoGuidanceItem::new(Some("↵"), "Continue"),
                InfoGuidanceItem::new(Some("⌘K"), "Open logs or repair actions"),
            ])),
        NonListStateShowcaseId::Permission => InfoStateSpec::new("info-state-permission")
            .layout(InfoStateLayout::Centered)
            .density(InfoStateDensity::Comfortable)
            .tone(InfoStateTone::Permission)
            .title("Allow this change?")
            .body("The agent wants to edit one file in this workspace. Review the scope, then allow or deny.")
            .section(InfoSection::titled(
                "Scope",
                vec![
                    InfoGuidanceItem::new(Some("File"), "src/components/info_state.rs"),
                    InfoGuidanceItem::new(Some("Shell"), "No command will run"),
                ],
            ))
            .section(InfoSection::new(vec![
                InfoGuidanceItem::new(Some("↵"), "Allow once"),
                InfoGuidanceItem::new(Some("Esc"), "Deny"),
            ])),
        NonListStateShowcaseId::Recovery => InfoStateSpec::new("info-state-recovery")
            .layout(InfoStateLayout::Centered)
            .density(InfoStateDensity::Compact)
            .tone(InfoStateTone::Recovery)
            .title("Update check failed")
            .body("Try again. If it keeps failing, open logs and copy the last error.")
            .section(InfoSection::new(vec![
                InfoGuidanceItem::new(Some("↵"), "Try again"),
                InfoGuidanceItem::new(Some("⌘K"), "Open logs"),
                InfoGuidanceItem::new(Some("Esc"), "Dismiss"),
            ])),
        NonListStateShowcaseId::About => InfoStateSpec::new("info-state-about")
            .layout(InfoStateLayout::Centered)
            .density(InfoStateDensity::Hero)
            .tone(InfoStateTone::About)
            .eyebrow("Script Kit")
            .title("Keyboard-first automation")
            .body("A launcher for scripts, agents, and everyday workflows.")
            .section(InfoSection::titled(
                "Product",
                vec![
                    InfoGuidanceItem::new(Some("Version"), "0.1.8"),
                    InfoGuidanceItem::new(Some("⌘1"), "GitHub"),
                    InfoGuidanceItem::new(Some("⌘2"), "Discord"),
                    InfoGuidanceItem::new(Some("⌘3"), "Updates"),
                ],
            )),
        NonListStateShowcaseId::Density => unreachable!("density renders a comparison stage"),
    }
}

fn render_density_state(compact: bool) -> AnyElement {
    let theme = get_cached_theme();
    let content = div()
        .size_full()
        .px(px(16.0))
        .py(px(16.0))
        .flex()
        .flex_row()
        .gap(px(12.0))
        .child(
            div().flex_1().min_w(px(0.0)).child(render_info_state(
                InfoStateSpec::new("info-state-density-compact")
                    .layout(InfoStateLayout::InlinePanel)
                    .density(InfoStateDensity::Compact)
                    .title("Compact")
                    .body("Inline help, empty composers, and launcher recovery.")
                    .section(InfoSection::new(vec![
                        InfoGuidanceItem::new(Some("Width"), "380px max"),
                        InfoGuidanceItem::new(Some("Title"), "14 / 20"),
                        InfoGuidanceItem::new(Some("Body"), "13 / 18"),
                    ])),
                &theme,
            )),
        )
        .child(
            div().flex_1().min_w(px(0.0)).child(render_info_state(
                InfoStateSpec::new("info-state-density-comfortable")
                    .layout(InfoStateLayout::InlinePanel)
                    .density(InfoStateDensity::Comfortable)
                    .title("Comfortable")
                    .body("Setup, permission, recovery, and explanatory surfaces.")
                    .section(InfoSection::new(vec![
                        InfoGuidanceItem::new(Some("Width"), "500px max"),
                        InfoGuidanceItem::new(Some("Title"), "16 / 22"),
                        InfoGuidanceItem::new(Some("Body"), "13 / 18"),
                    ])),
                &theme,
            )),
        );

    stage("info-state-density-stage", content, compact)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn non_list_showcase_covers_required_layouts() {
        let ids: Vec<_> = non_list_state_showcase_story_variants()
            .into_iter()
            .map(|variant| variant.stable_id())
            .collect();

        for required in [
            "empty",
            "help",
            "form",
            "setup",
            "permission",
            "recovery",
            "about",
            "density",
        ] {
            assert!(
                ids.iter().any(|id| id == required),
                "missing non-list showcase layout {required}"
            );
        }
    }

    #[test]
    fn non_list_showcase_variants_are_presenter_fixtures() {
        for variant in non_list_state_showcase_story_variants() {
            assert_eq!(
                variant.props.get("surface").map(String::as_str),
                Some("nonListState")
            );
            assert_eq!(
                variant.props.get("representation").map(String::as_str),
                Some("presenterFixture")
            );
            assert!(variant.props.contains_key("layout"));
        }
    }
}
