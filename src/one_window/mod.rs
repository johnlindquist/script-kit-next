//! The main window as one window: `sk_one_window::OneWindow` owns its rules, and these modules
//! connect it to `ScriptListApp`.

pub(crate) mod automations;
pub(crate) mod capture;
pub(crate) mod chat_entry;
pub(crate) mod chat_host;
mod chat_presentation;
pub(crate) mod chips;
pub(crate) mod connections;
mod desktop_fill;
mod handoff;
pub(crate) mod height_host;
pub(crate) mod host;
pub(crate) mod learns;
pub(crate) mod ledger_view;
pub(crate) mod lessons_host;
pub(crate) mod marks;
pub(crate) mod notes;
pub(crate) mod owner_pill;
pub(crate) mod past_chats;
pub(crate) mod reading;
pub(crate) mod saved_scripts;
pub(crate) mod share;
pub(crate) mod shortcut_view;
pub(crate) mod slot_line;
pub(crate) mod staging;
pub(crate) mod subject_tabs;
pub(crate) mod talk_host;
pub(crate) mod view_staging;
pub(crate) mod views;
pub(crate) mod voice_lists;
pub(crate) mod work;
pub(crate) mod work_view;
