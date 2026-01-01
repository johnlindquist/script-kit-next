//! Notes Window
//!
//! A separate floating window for notes, built with gpui-component.
//! This is completely independent from the main Script Kit launcher window.

use anyhow::Result;
use gpui::{
    div, prelude::*, px, size, App, Context, Entity, FocusHandle, Focusable, IntoElement,
    ParentElement, Render, SharedString, Styled, Subscription, Window, WindowBounds, WindowOptions,
};
use gpui_component::{
    button::{Button, ButtonVariants},
    input::{Input, InputEvent, InputState},
    sidebar::{Sidebar, SidebarGroup, SidebarMenu, SidebarMenuItem},
    theme::ActiveTheme,
    Root, Sizable,
};
use tracing::info;

use super::model::{Note, NoteId};
use super::storage;

/// Global handle to the notes window
static NOTES_WINDOW: std::sync::OnceLock<std::sync::Mutex<Option<gpui::WindowHandle<Root>>>> =
    std::sync::OnceLock::new();

/// The main notes application view
pub struct NotesApp {
    /// All notes (cached from storage)
    notes: Vec<Note>,

    /// Currently selected note ID
    selected_note_id: Option<NoteId>,

    /// Editor input state (using gpui-component's Input)
    editor_state: Entity<InputState>,

    /// Search input state
    #[allow(dead_code)]
    search_state: Entity<InputState>,

    /// Whether the sidebar is collapsed
    sidebar_collapsed: bool,

    /// Focus handle for keyboard navigation
    focus_handle: FocusHandle,

    /// Subscriptions to keep alive
    _subscriptions: Vec<Subscription>,
}

impl NotesApp {
    /// Create a new NotesApp
    pub fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        // Initialize storage
        if let Err(e) = storage::init_notes_db() {
            tracing::error!(error = %e, "Failed to initialize notes database");
        }

        // Load notes from storage
        let notes = storage::get_all_notes().unwrap_or_default();
        let selected_note_id = notes.first().map(|n| n.id);

        // Get initial content if we have a selected note
        let initial_content = selected_note_id
            .and_then(|id| notes.iter().find(|n| n.id == id))
            .map(|n| n.content.clone())
            .unwrap_or_default();

        // Create input states - use multi_line for the editor
        let editor_state = cx.new(|cx| {
            InputState::new(window, cx)
                .multi_line(true)
                .rows(20)
                .placeholder("Start typing your note...")
                .default_value(initial_content)
        });

        let search_state = cx.new(|cx| InputState::new(window, cx).placeholder("Search notes..."));

        let focus_handle = cx.focus_handle();

        // Subscribe to editor changes
        let subscriptions = vec![cx.subscribe_in(&editor_state, window, {
            move |this, _, ev: &InputEvent, _window, cx| {
                if matches!(ev, InputEvent::Change) {
                    this.on_editor_change(cx);
                }
            }
        })];

        info!(note_count = notes.len(), "Notes app initialized");

        Self {
            notes,
            selected_note_id,
            editor_state,
            search_state,
            sidebar_collapsed: false,
            focus_handle,
            _subscriptions: subscriptions,
        }
    }

    /// Handle editor content changes
    fn on_editor_change(&mut self, cx: &mut Context<Self>) {
        if let Some(id) = self.selected_note_id {
            let content = self.editor_state.read(cx).value();

            // Update the note in our cache
            if let Some(note) = self.notes.iter_mut().find(|n| n.id == id) {
                note.set_content(content.to_string());

                // Save to storage (debounced in a real implementation)
                if let Err(e) = storage::save_note(note) {
                    tracing::error!(error = %e, "Failed to save note");
                }
            }

            cx.notify();
        }
    }

    /// Create a new note
    fn create_note(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let note = Note::new();
        let id = note.id;

        // Save to storage
        if let Err(e) = storage::save_note(&note) {
            tracing::error!(error = %e, "Failed to create note");
            return;
        }

        // Add to cache and select it
        self.notes.insert(0, note);
        self.select_note(id, window, cx);

        info!(note_id = %id, "New note created");
    }

    /// Select a note for editing
    fn select_note(&mut self, id: NoteId, window: &mut Window, cx: &mut Context<Self>) {
        self.selected_note_id = Some(id);

        // Load content into editor
        if let Some(note) = self.notes.iter().find(|n| n.id == id) {
            self.editor_state.update(cx, |state, cx| {
                state.set_value(&note.content, window, cx);
            });
        }

        cx.notify();
    }

    /// Delete the currently selected note (soft delete)
    fn delete_selected_note(&mut self, cx: &mut Context<Self>) {
        if let Some(id) = self.selected_note_id {
            if let Some(note) = self.notes.iter_mut().find(|n| n.id == id) {
                note.soft_delete();

                if let Err(e) = storage::save_note(note) {
                    tracing::error!(error = %e, "Failed to delete note");
                }
            }

            // Remove from visible list and select next
            self.notes.retain(|n| n.id != id);
            self.selected_note_id = self.notes.first().map(|n| n.id);

            cx.notify();
        }
    }

    /// Render the notes sidebar
    fn render_sidebar(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let notes = &self.notes;
        let selected_id = self.selected_note_id;

        Sidebar::left()
            .collapsed(self.sidebar_collapsed)
            .header(
                div()
                    .flex()
                    .items_center()
                    .justify_between()
                    .w_full()
                    .child("Notes")
                    .child(
                        Button::new("new-note")
                            .ghost()
                            .small()
                            .icon(gpui_component::IconName::Plus)
                            .on_click(cx.listener(|this, _, window, cx| {
                                this.create_note(window, cx);
                            })),
                    ),
            )
            .child(
                SidebarGroup::new("notes-list").child(SidebarMenu::new().children(
                    notes.iter().map(|note| {
                        let note_id = note.id;
                        let is_selected = selected_id == Some(note_id);
                        let title: SharedString = if note.title.is_empty() {
                            "Untitled Note".into()
                        } else {
                            note.title.clone().into()
                        };

                        // SidebarMenuItem::new takes the label as its argument
                        SidebarMenuItem::new(title)
                            .active(is_selected)
                            .on_click(cx.listener(move |this, _, window, cx| {
                                this.select_note(note_id, window, cx);
                            }))
                    }),
                )),
            )
    }

    /// Render the main editor area
    fn render_editor(&self, cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .flex_1()
            .flex()
            .flex_col()
            .h_full()
            .p_4()
            .child(
                // Editor header with title
                div()
                    .flex()
                    .items_center()
                    .justify_between()
                    .pb_2()
                    .border_b_1()
                    .border_color(cx.theme().border)
                    .child(
                        div()
                            .text_lg()
                            .font_weight(gpui::FontWeight::SEMIBOLD)
                            .child(
                                self.selected_note_id
                                    .and_then(|id| self.notes.iter().find(|n| n.id == id))
                                    .map(|n| {
                                        if n.title.is_empty() {
                                            "Untitled Note".to_string()
                                        } else {
                                            n.title.clone()
                                        }
                                    })
                                    .unwrap_or_else(|| "No note selected".to_string()),
                            ),
                    )
                    .child(
                        div().flex().gap_2().child(
                            Button::new("delete")
                                .ghost()
                                .small()
                                .label("Delete")
                                .on_click(cx.listener(|this, _, _, cx| {
                                    this.delete_selected_note(cx);
                                })),
                        ),
                    ),
            )
            .child(
                // Editor content - full height multi-line input
                div()
                    .flex_1()
                    .pt_4()
                    .child(Input::new(&self.editor_state).h_full()),
            )
    }
}

impl Focusable for NotesApp {
    fn focus_handle(&self, _cx: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl Render for NotesApp {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .flex()
            .flex_row()
            .size_full()
            .bg(cx.theme().background)
            .text_color(cx.theme().foreground)
            .child(self.render_sidebar(cx))
            .child(self.render_editor(cx))
    }
}

/// Open the notes window (or focus it if already open)
pub fn open_notes_window(cx: &mut App) -> Result<()> {
    let window_handle = NOTES_WINDOW.get_or_init(|| std::sync::Mutex::new(None));
    let mut guard = window_handle.lock().unwrap();

    // Check if window already exists and is valid
    if let Some(ref handle) = *guard {
        // Try to focus the existing window
        if handle.update(cx, |_, _, cx| cx.notify()).is_ok() {
            info!("Focusing existing notes window");
            return Ok(());
        }
    }

    // Create new window
    info!("Opening new notes window");

    let window_options = WindowOptions {
        window_bounds: Some(WindowBounds::Windowed(gpui::Bounds::centered(
            None,
            size(px(900.), px(700.)),
            cx,
        ))),
        titlebar: Some(gpui::TitlebarOptions {
            title: Some("Script Kit Notes".into()),
            appears_transparent: true,
            ..Default::default()
        }),
        focus: true,
        show: true,
        kind: gpui::WindowKind::Normal,
        ..Default::default()
    };

    let handle = cx.open_window(window_options, |window, cx| {
        let view = cx.new(|cx| NotesApp::new(window, cx));
        cx.new(|cx| Root::new(view, window, cx))
    })?;

    *guard = Some(handle);

    Ok(())
}

/// Quick capture - open notes with a new note ready for input
pub fn quick_capture(cx: &mut App) -> Result<()> {
    open_notes_window(cx)?;

    // TODO: Focus the editor and optionally create a new note
    // This requires accessing the NotesApp through the Root wrapper

    Ok(())
}

/// Close the notes window
pub fn close_notes_window(cx: &mut App) {
    let window_handle = NOTES_WINDOW.get_or_init(|| std::sync::Mutex::new(None));
    let mut guard = window_handle.lock().unwrap();

    if let Some(handle) = guard.take() {
        let _ = handle.update(cx, |_, window, _| {
            window.remove_window();
        });
    }
}
