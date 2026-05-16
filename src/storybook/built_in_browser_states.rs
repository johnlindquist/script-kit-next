//! Presenter-backed built-in browser states for Storybook.
//!
//! Built-in browser renderers are owned by `ScriptListApp` because they carry
//! focus, input, storage, and platform side effects. These fixtures keep
//! Storybook deterministic while reusing the shared list, scaffold, and footer
//! chrome those renderers depend on.

use gpui::{div, prelude::*, px, rgba, AnyElement, FontWeight, SharedString};
use gpui_component::scroll::ScrollableElement;

use crate::list_item::{ListItem, ListItemColors};
use crate::storybook::StoryVariant;
use crate::theme::get_cached_theme;
use crate::ui_foundation::HexColorExt;

const ENTER: &str = "\u{21b5}";
const CMD_ENTER: &str = "\u{2318}\u{21b5}";
const CMD_C: &str = "\u{2318}C";
const CMD_K: &str = "\u{2318}K";
const ESC: &str = "Esc";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BuiltInBrowserStateId {
    FileSearch,
    FileSearchLoading,
    ClipboardHistory,
    BrowserTabs,
    BrowserHistoryPortal,
    NotesBrowsePortal,
    WindowSwitcher,
    DictationHistory,
    AcpHistory,
    SdkReference,
    EmptyResults,
}

impl BuiltInBrowserStateId {
    pub const ALL: [Self; 11] = [
        Self::FileSearch,
        Self::FileSearchLoading,
        Self::ClipboardHistory,
        Self::BrowserTabs,
        Self::BrowserHistoryPortal,
        Self::NotesBrowsePortal,
        Self::WindowSwitcher,
        Self::DictationHistory,
        Self::AcpHistory,
        Self::SdkReference,
        Self::EmptyResults,
    ];

    pub fn as_str(self) -> &'static str {
        match self {
            Self::FileSearch => "file-search",
            Self::FileSearchLoading => "file-search-loading",
            Self::ClipboardHistory => "clipboard-history",
            Self::BrowserTabs => "browser-tabs",
            Self::BrowserHistoryPortal => "browser-history-portal",
            Self::NotesBrowsePortal => "notes-browse-portal",
            Self::WindowSwitcher => "window-switcher",
            Self::DictationHistory => "dictation-history",
            Self::AcpHistory => "acp-history",
            Self::SdkReference => "sdk-reference",
            Self::EmptyResults => "empty-results",
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            Self::FileSearch => "File Search",
            Self::FileSearchLoading => "File Search Loading",
            Self::ClipboardHistory => "Clipboard History",
            Self::BrowserTabs => "Browser Tabs",
            Self::BrowserHistoryPortal => "Browser History Portal",
            Self::NotesBrowsePortal => "Notes Browse Portal",
            Self::WindowSwitcher => "Window Switcher",
            Self::DictationHistory => "Dictation History",
            Self::AcpHistory => "Agent Chat History",
            Self::SdkReference => "SDK Reference",
            Self::EmptyResults => "Empty Results",
        }
    }

    pub fn description(self) -> &'static str {
        match self {
            Self::FileSearch => "Expanded file search with list, preview, and footer actions.",
            Self::FileSearchLoading => {
                "File search while choices are still loading, with stable skeleton rows."
            }
            Self::ClipboardHistory => "Expanded clipboard history with selected-entry preview.",
            Self::BrowserTabs => "Compact browser tab switcher list without preview pane.",
            Self::BrowserHistoryPortal => {
                "Attachment portal for browser-history rows and page preview."
            }
            Self::NotesBrowsePortal => "Attachment portal for searchable notes and note preview.",
            Self::WindowSwitcher => "Window list paired with the live actions/details panel.",
            Self::DictationHistory => "Saved dictation browser with transcript preview.",
            Self::AcpHistory => "ACP conversation history browser with thread preview.",
            Self::SdkReference => "SDK reference browser with list, support state, and snippet.",
            Self::EmptyResults => "Shared no-results state for filtered built-in browsers.",
        }
    }

    pub fn from_stable_id(value: &str) -> Option<Self> {
        match value {
            "file-search" => Some(Self::FileSearch),
            "file-search-loading" => Some(Self::FileSearchLoading),
            "clipboard-history" => Some(Self::ClipboardHistory),
            "browser-tabs" => Some(Self::BrowserTabs),
            "browser-history-portal" => Some(Self::BrowserHistoryPortal),
            "notes-browse-portal" => Some(Self::NotesBrowsePortal),
            "window-switcher" => Some(Self::WindowSwitcher),
            "dictation-history" => Some(Self::DictationHistory),
            "acp-history" => Some(Self::AcpHistory),
            "sdk-reference" => Some(Self::SdkReference),
            "empty-results" => Some(Self::EmptyResults),
            _ => None,
        }
    }
}

pub fn built_in_browser_state_story_variants() -> Vec<StoryVariant> {
    BuiltInBrowserStateId::ALL
        .into_iter()
        .map(|id| {
            StoryVariant::default_named(id.as_str(), id.name())
                .description(id.description())
                .with_prop("surface", "builtInBrowser")
                .with_prop("representation", "presenterFixture")
                .with_prop("state", id.as_str())
        })
        .collect()
}

pub fn render_built_in_browser_state_preview(stable_id: &str) -> AnyElement {
    let id = BuiltInBrowserStateId::from_stable_id(stable_id)
        .unwrap_or(BuiltInBrowserStateId::FileSearch);
    render_built_in_browser_state(id, false)
}

pub fn render_built_in_browser_state_compare_thumbnail(stable_id: &str) -> AnyElement {
    let id = BuiltInBrowserStateId::from_stable_id(stable_id)
        .unwrap_or(BuiltInBrowserStateId::FileSearch);
    render_built_in_browser_state(id, true)
}

fn render_built_in_browser_state(id: BuiltInBrowserStateId, compact: bool) -> AnyElement {
    match id {
        BuiltInBrowserStateId::BrowserTabs => {
            render_single_list_fixture(browser_tabs_fixture(), compact)
        }
        _ => render_expanded_fixture(fixture_for(id), compact),
    }
}

#[derive(Clone)]
struct BrowserFixture {
    title: &'static str,
    filter_text: &'static str,
    placeholder: &'static str,
    count_label: &'static str,
    rows: Vec<BrowserRow>,
    selected_index: usize,
    preview: PreviewPane,
    hints: Vec<SharedString>,
}

#[derive(Clone)]
struct BrowserRow {
    title: &'static str,
    description: Option<&'static str>,
    shortcut: Option<&'static str>,
    badge: Option<&'static str>,
    hovered: bool,
}

#[derive(Clone)]
struct PreviewPane {
    eyebrow: &'static str,
    title: &'static str,
    subtitle: Option<&'static str>,
    body: Vec<&'static str>,
    badges: Vec<&'static str>,
    code_lines: Vec<&'static str>,
}

fn fixture_for(id: BuiltInBrowserStateId) -> BrowserFixture {
    match id {
        BuiltInBrowserStateId::FileSearch => BrowserFixture {
            title: "File Search",
            filter_text: "story",
            placeholder: "Search files...",
            count_label: "18 files",
            rows: vec![
                row("src/storybook/main_menu_variations/mod.rs", "Rust source - 46 KB", "rs"),
                row("src/storybook/dictation_states.rs", "Rust source - state fixtures", "rs"),
                row("removed-docs", "Architecture notes - markdown", "md"),
                row(".claude/skills/storybook/SKILL.md", "Agent workflow notes", "md"),
                row("src/bin/storybook.rs", "Standalone Storybook binary", "rs"),
            ],
            selected_index: 1,
            preview: PreviewPane {
                eyebrow: "RUST SOURCE",
                title: "src/storybook/dictation_states.rs",
                subtitle: Some("Canonical dictation capsule states"),
                body: vec![
                    "Defines deterministic state fixtures for the compact dictation overlay.",
                    "The renderer uses shared capsule tokens so Storybook stays aligned with the runtime surface.",
                ],
                badges: vec!["rust", "storybook", "canonical"],
                code_lines: vec![
                    "pub enum DictationStateId {",
                    "    IdleHidden,",
                    "    RecordingSpeech,",
                    "    StopConfirmation,",
                    "}",
                ],
            },
            hints: hints(&[
                (ENTER, "Open"),
                (CMD_ENTER, "Attach"),
                (CMD_K, "Actions"),
            ]),
        },
        BuiltInBrowserStateId::FileSearchLoading => BrowserFixture {
            title: "File Search",
            filter_text: "story",
            placeholder: "Search files...",
            count_label: "Indexing files",
            rows: Vec::new(),
            selected_index: 0,
            preview: PreviewPane {
                eyebrow: "FILE SEARCH",
                title: "Loading choices",
                subtitle: Some("The list holds its row rhythm while file results stream in."),
                body: vec![
                    "Skeleton rows reserve the same icon, path, and metadata columns as real file results.",
                    "The preview pane stays quiet until a result can be selected.",
                ],
                badges: vec!["loading", "skeleton"],
                code_lines: Vec::new(),
            },
            hints: hints(&[(CMD_ENTER, "Explain with AI"), (CMD_K, "Actions"), (ESC, "Back")]),
        },
        BuiltInBrowserStateId::ClipboardHistory => BrowserFixture {
            title: "Clipboard History",
            filter_text: "",
            placeholder: "Search clipboard...",
            count_label: "42 entries",
            rows: vec![
                row("Storybook cleanup plan", "Markdown - copied 2m ago", "md"),
                row("cargo build --features storybook --bin storybook", "Shell - copied 8m ago", "sh"),
                row("https://github.com/script-kit/script-kit-gpui", "URL - copied yesterday", "url"),
                row("Main menu state matrix", "Plain text - copied yesterday", "txt"),
                row("source checks", "Shell - copied Mon", "sh"),
            ],
            selected_index: 0,
            preview: PreviewPane {
                eyebrow: "CLIPBOARD ENTRY",
                title: "Storybook cleanup plan",
                subtitle: Some("Markdown text - 1,248 characters"),
                body: vec![
                    "Keep main menu and dictation as anchors, remove runtime image fixtures, then add canonical state coverage for each app window.",
                    "Every new story should be deterministic and browseable without reading user data.",
                ],
                badges: vec!["text", "markdown"],
                code_lines: Vec::new(),
            },
            hints: hints(&[(ENTER, "Paste"), (CMD_K, "Actions"), (ESC, "Back")]),
        },
        BuiltInBrowserStateId::BrowserHistoryPortal => BrowserFixture {
            title: "Browser History",
            filter_text: "storybook",
            placeholder: "Search browser history...",
            count_label: "7 pages",
            rows: vec![
                row("Storybook catalog JSON", "Chrome - today 10:42", "web"),
                row("GPUI component docs", "Safari - today 09:16", "web"),
                row("Script Kit GPUI pull request", "Chrome - yesterday", "web"),
                row("Agentic testing receipts", "Arc - yesterday", "web"),
            ],
            selected_index: 0,
            preview: PreviewPane {
                eyebrow: "BROWSER HISTORY",
                title: "Storybook catalog JSON",
                subtitle: Some("http://localhost:7337/catalog-json"),
                body: vec![
                    "Local Storybook catalog endpoint used to confirm registered stories, roles, surfaces, variants, and fixture representation types.",
                    "Attachment portals preserve the page title and URL as structured ACP context.",
                ],
                badges: vec!["chrome", "portal", "context"],
                code_lines: Vec::new(),
            },
            hints: hints(&[(ENTER, "Attach Page"), (CMD_K, "Actions"), (ESC, "Cancel")]),
        },
        BuiltInBrowserStateId::NotesBrowsePortal => BrowserFixture {
            title: "Notes Browse",
            filter_text: "menu",
            placeholder: "Search notes...",
            count_label: "4 notes",
            rows: vec![
                row("Daily launch notes", "2026-04-22 09:41 - 928 chars - pinned", "note"),
                row("Main menu polish", "2026-04-21 18:14 - 642 chars", "note"),
                row("Dictation regressions", "2026-04-20 15:02 - 388 chars", "note"),
                row("ACP attachment ideas", "2026-04-19 11:27 - 1,040 chars", "note"),
            ],
            selected_index: 1,
            preview: PreviewPane {
                eyebrow: "NOTE",
                title: "Main menu polish",
                subtitle: Some("Updated 2026-04-21 18:14"),
                body: vec![
                    "The menu needs to feel like the production launcher, not a gallery card.",
                    "State coverage should make row density, footer behavior, selection, and empty states easy to compare.",
                ],
                badges: vec!["note", "portal"],
                code_lines: Vec::new(),
            },
            hints: hints(&[(ENTER, "Attach Note"), (CMD_K, "Actions"), (ESC, "Cancel")]),
        },
        BuiltInBrowserStateId::WindowSwitcher => BrowserFixture {
            title: "Window Switcher",
            filter_text: "",
            placeholder: "Search windows...",
            count_label: "5 windows",
            rows: vec![
                row("Arc: Script Kit GPUI", "1512x982 at (48, 72)", "app"),
                row("Ghostty: cargo build", "1180x760 at (120, 96)", "app"),
                row("Code: script-kit-gpui", "1440x900 at (96, 54)", "app"),
                row("Finder: Downloads", "1024x720 at (210, 160)", "app"),
            ],
            selected_index: 2,
            preview: PreviewPane {
                eyebrow: "WINDOW",
                title: "Code: script-kit-gpui",
                subtitle: Some("Visual Studio Code"),
                body: vec![
                    "Bounds: 1440x900 at 96, 54",
                    "The right pane mirrors the live actions/details panel rather than a document preview.",
                ],
                badges: vec!["switch", "details"],
                code_lines: vec!["Enter  Switch", "Esc    Back"],
            },
            hints: hints(&[(ENTER, "Switch"), (ESC, "Back")]),
        },
        BuiltInBrowserStateId::DictationHistory => BrowserFixture {
            title: "Dictation History",
            filter_text: "storybook",
            placeholder: "Search dictations...",
            count_label: "6 transcripts",
            rows: vec![
                row("Storybook cleanup summary", "Today 10:18 - 00:42", "voice"),
                row("Main menu row spacing notes", "Yesterday 17:22 - 01:14", "voice"),
                row("ACP portal test case", "Apr 20 - 00:33", "voice"),
            ],
            selected_index: 0,
            preview: PreviewPane {
                eyebrow: "TRANSCRIPT",
                title: "Storybook cleanup summary",
                subtitle: Some("Duration 00:42"),
                body: vec![
                    "Remove the screenshot fixture experiment and make Storybook show deterministic runtime states.",
                    "The dictation overlay should stay compact and prove target app, Agent Chat target, confirmation, transcribing, and error states.",
                ],
                badges: vec!["dictation", "transcript"],
                code_lines: Vec::new(),
            },
            hints: hints(&[(ENTER, "Attach Transcript"), (CMD_K, "Actions"), (ESC, "Cancel")]),
        },
        BuiltInBrowserStateId::AcpHistory => BrowserFixture {
            title: "Agent Chat History",
            filter_text: "storybook",
            placeholder: "Search chats...",
            count_label: "9 chats",
            rows: vec![
                row("Storybook state coverage", "Sonnet - 14 messages", "chat"),
                row("Popup positioning regression", "GPT-5 - 8 messages", "chat"),
                row("Notes window redesign", "Sonnet - 22 messages", "chat"),
            ],
            selected_index: 0,
            preview: PreviewPane {
                eyebrow: "ACP THREAD",
                title: "Storybook state coverage",
                subtitle: Some("14 messages - last updated today"),
                body: vec![
                    "Thread includes the cleanup plan, fixture migration rules, and confirmation that primary stories should be presenter-backed.",
                    "History rows keep model and message count visible so the preview is useful before attachment.",
                ],
                badges: vec!["acp", "history"],
                code_lines: Vec::new(),
            },
            hints: hints(&[(ENTER, "Attach Chat"), (CMD_K, "Actions"), (ESC, "Cancel")]),
        },
        BuiltInBrowserStateId::SdkReference => BrowserFixture {
            title: "SDK Reference",
            filter_text: "notify",
            placeholder: "Search SDK...",
            count_label: "3 functions",
            rows: vec![
                row("notify(message)", "Supported - System notification", "api"),
                row("hud(message)", "Supported - Launcher-local feedback", "api"),
                row("menu(items)", "Unsupported - use arg/select flow", "api"),
            ],
            selected_index: 0,
            preview: PreviewPane {
                eyebrow: "SUPPORTED API",
                title: "notify(message)",
                subtitle: Some("Deliver an OS-level Notification Center message."),
                body: vec![
                    "Use notify when the script result should leave the launcher and appear as a system notification.",
                    "For launcher-local transient feedback, prefer hud(message).",
                ],
                badges: vec!["supported", "notification"],
                code_lines: vec![
                    "await notify({",
                    "  title: \"Build complete\",",
                    "  body: \"Storybook catalog is current\"",
                    "})",
                ],
            },
            hints: hints(&[(ENTER, "Copy"), (CMD_C, "Copy Markdown"), (ESC, "Back")]),
        },
        BuiltInBrowserStateId::EmptyResults => BrowserFixture {
            title: "Filtered Built-in Browser",
            filter_text: "zz-no-match",
            placeholder: "Search...",
            count_label: "0 results",
            rows: Vec::new(),
            selected_index: 0,
            preview: PreviewPane {
                eyebrow: "EMPTY",
                title: "No item selected",
                subtitle: None,
                body: vec![
                    "Filtered built-in browsers keep the header, split layout, and footer stable while the list reports an empty state.",
                ],
                badges: vec!["empty", "filter"],
                code_lines: Vec::new(),
            },
            hints: hints(&[(ESC, "Clear Filter"), (CMD_K, "Actions")]),
        },
        BuiltInBrowserStateId::BrowserTabs => browser_tabs_fixture(),
    }
}

fn browser_tabs_fixture() -> BrowserFixture {
    BrowserFixture {
        title: "Browser Tabs",
        filter_text: "script",
        placeholder: "Search tabs...",
        count_label: "5 tabs",
        rows: vec![
            row("Script Kit GPUI - Pull Request", "Arc - github.com", "tab"),
            row("Storybook catalog JSON", "Chrome - localhost:7337", "tab"),
            row("GPUI input component notes", "Safari - docs.rs", "tab"),
            row("Agentic testing receipt", "Arc - local file", "tab"),
            row("Script Kit documentation", "Chrome - scriptkit.com", "tab"),
        ],
        selected_index: 1,
        preview: PreviewPane {
            eyebrow: "BROWSER TAB",
            title: "Storybook catalog JSON",
            subtitle: Some("Chrome - localhost:7337"),
            body: Vec::new(),
            badges: vec!["tab"],
            code_lines: Vec::new(),
        },
        hints: hints(&[(ENTER, "Activate"), (CMD_K, "Actions"), (ESC, "Back")]),
    }
}

fn row(title: &'static str, description: &'static str, badge: &'static str) -> BrowserRow {
    BrowserRow {
        title,
        description: Some(description),
        shortcut: None,
        badge: Some(badge),
        hovered: false,
    }
}

fn hints(items: &[(&'static str, &'static str)]) -> Vec<SharedString> {
    items
        .iter()
        .map(|(key, label)| SharedString::from(format!("{key} {label}")))
        .collect()
}

fn render_expanded_fixture(fixture: BrowserFixture, compact: bool) -> AnyElement {
    let theme = get_cached_theme();
    let header = render_header(&fixture, compact);
    let list_pane = render_list_pane(&fixture, compact);
    let preview_pane = render_preview_pane(&fixture.preview, compact);
    let scaffold = crate::components::render_expanded_view_scaffold_with_hints(
        header,
        list_pane,
        preview_pane,
        fixture.hints,
        None,
    )
    .text_color(theme.colors.text.primary.to_rgb())
    .font_family("Geist");

    render_shell(scaffold, compact)
}

fn render_single_list_fixture(fixture: BrowserFixture, compact: bool) -> AnyElement {
    let theme = get_cached_theme();
    let body = div()
        .w_full()
        .h_full()
        .flex()
        .flex_col()
        .child(
            div()
                .w_full()
                .px(px(crate::ui::chrome::HEADER_PADDING_X))
                .py(px(crate::ui::chrome::HEADER_PADDING_Y))
                .child(render_header(&fixture, compact)),
        )
        .child(
            div()
                .flex_1()
                .min_h(px(0.0))
                .w_full()
                .overflow_hidden()
                .child(render_list_pane(&fixture, compact)),
        )
        .child(crate::components::render_simple_hint_strip(
            fixture.hints,
            None,
        ))
        .text_color(theme.colors.text.primary.to_rgb())
        .font_family("Geist");

    render_shell(body, compact)
}

fn render_shell(content: impl IntoElement, compact: bool) -> AnyElement {
    let theme = get_cached_theme();
    let width = if compact { 500.0 } else { 820.0 };
    let height = if compact { 320.0 } else { 500.0 };

    div()
        .w_full()
        .min_h(px(if compact { 340.0 } else { 540.0 }))
        .flex()
        .items_center()
        .justify_center()
        .child(
            div()
                .w(px(width))
                .h(px(height))
                .rounded(px(10.0))
                .overflow_hidden()
                .border_1()
                .border_color(rgba((theme.colors.ui.border << 8) | 0x66))
                .bg(theme.colors.background.main.to_rgb())
                .child(content),
        )
        .into_any_element()
}

fn render_header(fixture: &BrowserFixture, compact: bool) -> AnyElement {
    let theme = get_cached_theme();
    let input_text = if fixture.filter_text.is_empty() {
        fixture.placeholder
    } else {
        fixture.filter_text
    };
    let text_color = if fixture.filter_text.is_empty() {
        theme.colors.text.dimmed.to_rgb()
    } else {
        theme.colors.text.primary.to_rgb()
    };

    div()
        .flex_1()
        .flex()
        .flex_row()
        .items_center()
        .gap(px(12.0))
        .child(
            div()
                .flex_1()
                .h(px(28.0))
                .flex()
                .flex_row()
                .items_center()
                .gap(px(6.0))
                .text_size(px(if compact { 14.0 } else { 16.0 }))
                .text_color(text_color)
                .child(input_text)
                .when(!fixture.filter_text.is_empty(), |d| {
                    d.child(
                        div().w(px(1.5)).h(px(17.0)).rounded(px(1.0)).bg(theme
                            .colors
                            .accent
                            .selected
                            .to_rgb()),
                    )
                }),
        )
        .child(
            div()
                .text_xs()
                .font_weight(FontWeight::SEMIBOLD)
                .text_color(theme.colors.text.dimmed.to_rgb())
                .child(fixture.title),
        )
        .child(
            div()
                .text_sm()
                .text_color(theme.colors.text.dimmed.to_rgb())
                .child(fixture.count_label),
        )
        .into_any_element()
}

fn render_list_pane(fixture: &BrowserFixture, compact: bool) -> AnyElement {
    let theme = get_cached_theme();
    let colors = ListItemColors::from_theme(&theme);
    let selected_index = fixture.selected_index;
    let rows: Vec<AnyElement> = if fixture.rows.is_empty() {
        if fixture.count_label == "Indexing files" {
            return div()
                .w_full()
                .h_full()
                .min_h(px(0.0))
                .overflow_hidden()
                .flex()
                .flex_col()
                .child(
                    div()
                        .w_full()
                        .px(px(12.0))
                        .pt(px(8.0))
                        .pb(px(2.0))
                        .flex()
                        .justify_end()
                        .child(render_loading_badge()),
                )
                .child(render_loading_skeleton_rows(compact))
                .into_any_element();
        }
        vec![render_empty_row(fixture.filter_text, compact)]
    } else {
        fixture
            .rows
            .iter()
            .enumerate()
            .map(|(ix, row)| render_browser_row(row, ix == selected_index, colors))
            .collect()
    };

    div()
        .w_full()
        .h_full()
        .min_h(px(0.0))
        .py(px(4.0))
        .overflow_hidden()
        .flex()
        .flex_col()
        .children(rows)
        .into_any_element()
}

fn render_loading_badge() -> AnyElement {
    let theme = get_cached_theme();

    div()
        .px(px(9.0))
        .py(px(4.0))
        .rounded(px(999.0))
        .border_1()
        .border_color(rgba((theme.colors.accent.selected << 8) | 0x24))
        .bg(rgba((theme.colors.accent.selected << 8) | 0x10))
        .text_xs()
        .font_weight(FontWeight::MEDIUM)
        .text_color(theme.colors.text.dimmed.to_rgb())
        .child("Indexing files")
        .into_any_element()
}

fn render_loading_skeleton_rows(compact: bool) -> AnyElement {
    let theme = get_cached_theme();
    let rail_bg = rgba((theme.colors.accent.selected << 8) | 0x22);
    let skeleton_bg = rgba((theme.colors.ui.border << 8) | 0x12);
    let skeleton_strong = rgba((theme.colors.ui.border << 8) | 0x26);
    let muted_bg = rgba((theme.colors.text.dimmed << 8) | 0x10);
    let row_height = if compact { 46.0 } else { 52.0 };
    let row_specs = [
        (156.0, 246.0, 52.0, 70.0),
        (214.0, 302.0, 44.0, 62.0),
        (182.0, 274.0, 58.0, 78.0),
        (238.0, 326.0, 48.0, 66.0),
        (168.0, 256.0, 56.0, 72.0),
        (206.0, 288.0, 42.0, 60.0),
    ];

    div()
        .w_full()
        .h_full()
        .flex()
        .flex_col()
        .py(px(6.0))
        .children(row_specs.into_iter().enumerate().map(
            |(ix, (title_w, path_w, size_w, age_w))| {
                div()
                    .id(ix)
                    .w_full()
                    .h(px(row_height))
                    .flex()
                    .flex_row()
                    .items_center()
                    .px(px(12.0))
                    .gap(px(12.0))
                    .when(ix == 0, |row| {
                        row.bg(rgba((theme.colors.ui.border << 8) | 0x08))
                    })
                    .child(div().w(px(3.0)).h(px(28.0)).rounded(px(2.0)).bg(rail_bg))
                    .child(
                        div()
                            .w(px(26.0))
                            .h(px(26.0))
                            .rounded(px(6.0))
                            .border_1()
                            .border_color(skeleton_strong)
                            .bg(skeleton_bg),
                    )
                    .child(
                        div()
                            .flex_1()
                            .min_w(px(0.0))
                            .flex()
                            .flex_col()
                            .gap(px(7.0))
                            .child(
                                div()
                                    .w(px(title_w))
                                    .max_w_full()
                                    .h(px(11.0))
                                    .rounded(px(5.5))
                                    .bg(skeleton_strong),
                            )
                            .child(
                                div()
                                    .w(px(path_w))
                                    .max_w_full()
                                    .h(px(8.0))
                                    .rounded(px(4.0))
                                    .bg(skeleton_bg),
                            ),
                    )
                    .child(
                        div()
                            .w(px(if compact { 76.0 } else { 104.0 }))
                            .flex()
                            .flex_col()
                            .items_end()
                            .gap(px(7.0))
                            .child(
                                div()
                                    .w(px(size_w))
                                    .h(px(8.0))
                                    .rounded(px(4.0))
                                    .bg(skeleton_bg),
                            )
                            .child(div().w(px(age_w)).h(px(8.0)).rounded(px(4.0)).bg(muted_bg)),
                    )
            },
        ))
        .into_any_element()
}

fn render_browser_row(row: &BrowserRow, selected: bool, colors: ListItemColors) -> AnyElement {
    let mut item = ListItem::new(row.title, colors)
        .description_opt(row.description.map(str::to_string))
        .shortcut_opt(row.shortcut.map(str::to_string))
        .tool_badge_opt(row.badge.map(str::to_string))
        .selected(selected)
        .hovered(row.hovered)
        .with_accent_bar(true);

    if selected {
        item = item.semantic_id(format!("choice:0:{}", row.title.replace(' ', "-")));
    }

    div().w_full().child(item).into_any_element()
}

fn render_empty_row(filter_text: &str, compact: bool) -> AnyElement {
    let theme = get_cached_theme();
    div()
        .w_full()
        .py(px(if compact { 28.0 } else { 42.0 }))
        .px(px(14.0))
        .text_center()
        .text_color(theme.colors.text.muted.to_rgb())
        .text_size(px(if compact { 13.0 } else { 14.0 }))
        .child(if filter_text.is_empty() {
            "No items available".to_string()
        } else {
            format!("No results for \"{filter_text}\"")
        })
        .into_any_element()
}

fn render_preview_pane(preview: &PreviewPane, compact: bool) -> AnyElement {
    let theme = get_cached_theme();
    let padding = if compact { 14.0 } else { 22.0 };
    let body_size = if compact { 12.0 } else { 13.0 };

    let mut pane = div()
        .w_full()
        .h_full()
        .min_h(px(0.0))
        .overflow_y_scrollbar()
        .px(px(padding))
        .py(px(padding))
        .flex()
        .flex_col()
        .gap(px(if compact { 8.0 } else { 12.0 }))
        .child(
            div()
                .text_xs()
                .font_weight(FontWeight::SEMIBOLD)
                .text_color(theme.colors.text.muted.to_rgb())
                .child(preview.eyebrow),
        )
        .child(
            div()
                .text_lg()
                .font_weight(FontWeight::SEMIBOLD)
                .text_color(theme.colors.text.primary.to_rgb())
                .child(preview.title),
        );

    if let Some(subtitle) = preview.subtitle {
        pane = pane.child(
            div()
                .text_sm()
                .text_color(theme.colors.text.dimmed.to_rgb())
                .child(subtitle),
        );
    }

    if !preview.badges.is_empty() {
        pane = pane.child(
            div()
                .flex()
                .flex_row()
                .flex_wrap()
                .gap(px(6.0))
                .children(preview.badges.iter().map(|badge| render_badge(*badge))),
        );
    }

    pane = pane.children(preview.body.iter().map(|paragraph| {
        div()
            .text_size(px(body_size))
            .line_height(px(if compact { 18.0 } else { 20.0 }))
            .text_color(theme.colors.text.secondary.to_rgb())
            .child(*paragraph)
    }));

    if !preview.code_lines.is_empty() {
        pane = pane.child(render_code_block(&preview.code_lines, compact));
    }

    pane.into_any_element()
}

fn render_badge(label: &'static str) -> AnyElement {
    let theme = get_cached_theme();
    div()
        .px(px(7.0))
        .py(px(3.0))
        .rounded(px(4.0))
        .border_1()
        .border_color(rgba((theme.colors.ui.border << 8) | 0x70))
        .bg(rgba((theme.colors.background.main << 8) | 0xb3))
        .text_xs()
        .text_color(theme.colors.text.secondary.to_rgb())
        .child(label)
        .into_any_element()
}

fn render_code_block(lines: &[&'static str], compact: bool) -> AnyElement {
    let theme = get_cached_theme();
    div()
        .w_full()
        .rounded(px(6.0))
        .border_1()
        .border_color(rgba((theme.colors.ui.border << 8) | 0x66))
        .bg(rgba((theme.colors.background.main << 8) | 0xcc))
        .px(px(if compact { 9.0 } else { 12.0 }))
        .py(px(if compact { 8.0 } else { 10.0 }))
        .flex()
        .flex_col()
        .gap(px(3.0))
        .children(lines.iter().map(|line| {
            div()
                .text_xs()
                .font_family(crate::list_item::FONT_MONO)
                .text_color(theme.colors.text.secondary.to_rgb())
                .child(*line)
        }))
        .into_any_element()
}
