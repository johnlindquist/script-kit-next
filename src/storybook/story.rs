use gpui::*;
use std::collections::HashMap;

/// A story renders a component in various states for preview
///
/// Stories are stateless previews of components. They render static elements
/// and don't require app state or window mutations.
pub trait Story: Send + Sync {
    fn id(&self) -> &'static str;
    fn name(&self) -> &'static str;
    fn category(&self) -> &'static str;
    fn surface(&self) -> StorySurface {
        StorySurface::Component
    }
    /// Render the story preview.
    /// Note: Window and App are provided for compatibility but stories should
    /// render stateless elements that don't depend on app state.
    fn render(&self) -> AnyElement;
    fn render_variant(&self, _variant: &StoryVariant) -> AnyElement {
        self.render()
    }
    fn variants(&self) -> Vec<StoryVariant> {
        vec![StoryVariant::default_named("default", "Default")]
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum StorySurface {
    #[default]
    Component,
    Footer,
    Input,
    Header,
    Shell,
    ActionDialog,
    TurnCard,
    FullPrompt,
}

impl StorySurface {
    pub fn label(&self) -> &'static str {
        match self {
            Self::Component => "Component",
            Self::Footer => "Footer",
            Self::Input => "Input",
            Self::Header => "Header",
            Self::Shell => "Shell",
            Self::ActionDialog => "Action Dialog",
            Self::TurnCard => "Turn Card",
            Self::FullPrompt => "Full Prompt",
        }
    }
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct StoryVariant {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub props: HashMap<String, String>,
}

impl StoryVariant {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            id: String::new(),
            name: name.into(),
            description: None,
            props: HashMap::new(),
        }
    }

    pub fn default_named(id: impl Into<String>, name: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            description: None,
            props: HashMap::new(),
        }
    }

    pub fn description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    pub fn with_prop(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.props.insert(key.into(), value.into());
        self
    }

    pub fn stable_id(&self) -> String {
        if !self.id.trim().is_empty() {
            return self.id.clone();
        }

        let mut slug = String::with_capacity(self.name.len());
        let mut last_was_dash = false;

        for ch in self.name.chars().flat_map(char::to_lowercase) {
            if ch.is_alphanumeric() {
                slug.push(ch);
                last_was_dash = false;
            } else if !last_was_dash {
                slug.push('-');
                last_was_dash = true;
            }
        }

        let slug = slug.trim_matches('-').to_string();
        if slug.is_empty() {
            "default".to_string()
        } else {
            slug
        }
    }
}

#[cfg(test)]
mod tests {
    use super::StoryVariant;

    #[test]
    fn stable_id_falls_back_to_slugified_name() {
        let variant = StoryVariant::new("Script Kit Branded");
        assert_eq!(variant.stable_id(), "script-kit-branded");
    }
}
