//! Personal lists are declarations for Kit's existing row renderer, not programs. A source
//! determines its legal filters, row detail and action. Authoring one never opens a source,
//! reads a note/pane, installs a connector or grants the right to perform its action.
use schemars::{JsonSchema, generate::SchemaSettings};
use serde::{Deserialize, Serialize};
use serde_json::Value;

pub const MAX_VIEWS: usize = 8;
pub const MAX_ROWS: u8 = 48;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ViewRecipe {
    /// A local name, never a path or an action id.
    pub id: String,
    /// Name shown when the person opens their list.
    pub title: String,
    pub limit: u8,
    pub list: ListSpec,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "source", deny_unknown_fields)]
pub enum ListSpec {
    #[serde(rename = "kit.noteTitles")]
    Notes {
        filter: NoteFilter,
        sort: NoteSort,
        detail: NoteDetail,
        enter: NoteAction,
    },
    #[serde(rename = "herdr.paneMetadata")]
    Panes {
        filter: PaneFilter,
        sort: PaneSort,
        detail: PaneDetail,
        enter: PaneAction,
    },
}
macro_rules! choices {
    ($name:ident: $($item:ident),+) => {
        #[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
        #[serde(rename_all = "camelCase")]
        pub enum $name { $($item),+ }
    };
}
choices!(NoteFilter: All, Pinned);
choices!(NoteSort: Title, LastChanged);
choices!(NoteDetail: None, LastChanged, Pinned);
choices!(NoteAction: OpenNote);
choices!(PaneFilter: All, NeedsAttention);
choices!(PaneSort: Title, AttentionFirst);
choices!(PaneDetail: None, Space, Status);
choices!(PaneAction: None, Focus, FocusAndZoom);

pub fn valid_id(id: &str) -> bool {
    let b = id.as_bytes();
    (1..=40).contains(&b.len())
        && b[0].is_ascii_lowercase()
        && !id.ends_with('-')
        && !id.contains("--")
        && b.iter()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || *c == b'-')
}
impl ViewRecipe {
    pub fn validate(&self) -> super::Result<()> {
        let title = &self.title;
        if !valid_id(&self.id)
            || title.trim() != title
            || title.is_empty()
            || title.chars().count() > 64
            || title
                .chars()
                .any(|c| c.is_control() || matches!(c, '\u{61c}' | '\u{200b}'..='\u{200f}' | '\u{2028}'..='\u{202e}' | '\u{2060}'..='\u{206f}' | '\u{feff}'))
            || !(1..=MAX_ROWS).contains(&self.limit)
        {
            return Err(super::Error::InvalidView);
        }
        Ok(())
    }
    /// Runtime requirements, not permissions inherited by an author or minted by a recipe.
    pub fn requirements(&self) -> &'static [&'static str] {
        match self.list {
            ListSpec::Notes { .. } => &["notes.titles", "notes.open"],
            ListSpec::Panes {
                enter: PaneAction::None,
                ..
            } => &["herdr.panes.metadata"],
            ListSpec::Panes {
                enter: PaneAction::Focus,
                ..
            } => &["herdr.panes.metadata", "herdr.panes.focus"],
            ListSpec::Panes {
                enter: PaneAction::FocusAndZoom,
                ..
            } => &[
                "herdr.panes.metadata",
                "herdr.panes.focus",
                "herdr.panes.zoom",
            ],
        }
    }
}
pub(crate) fn validate_all(views: &[ViewRecipe]) -> super::Result<()> {
    if views.len() > MAX_VIEWS {
        return Err(super::Error::ViewLimit);
    }
    let mut ids = std::collections::BTreeSet::new();
    for v in views {
        v.validate()?;
        if !ids.insert(&v.id) {
            return Err(super::Error::InvalidView);
        }
    }
    Ok(())
}
/// Inline schema so it can be embedded under either MCP tool without dangling $refs.
pub fn schema() -> Value {
    let mut generator = SchemaSettings::default()
        .with(|s| s.inline_subschemas = true)
        .into_generator();
    let mut schema =
        serde_json::to_value(generator.subschema_for::<ViewRecipe>()).expect("view schema");
    let props = &mut schema["properties"];
    props["id"]["pattern"] = "^[a-z]([a-z0-9]|-[a-z0-9])*$".into();
    props["id"]["maxLength"] = 40.into();
    props["title"]["minLength"] = 1.into();
    props["title"]["maxLength"] = 64.into();
    props["limit"]["minimum"] = 1.into();
    props["limit"]["maximum"] = MAX_ROWS.into();
    schema
}
