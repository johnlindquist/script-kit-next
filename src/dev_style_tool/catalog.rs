use crate::designs::MainMenuThemeDef;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct StyleKnobId(&'static str);

impl StyleKnobId {
    pub const fn new(value: &'static str) -> Self {
        Self(value)
    }

    pub const fn as_str(self) -> &'static str {
        self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum StyleValue {
    Number(f32),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StyleKnobGroup {
    Search,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StyleUnit {
    Px,
}

#[derive(Debug, Clone, Copy)]
pub struct StyleKnob {
    pub id: StyleKnobId,
    pub label: &'static str,
    pub group: StyleKnobGroup,
    pub unit: StyleUnit,
    pub min: f32,
    pub max: f32,
    pub step: f32,
    pub get: fn(&MainMenuThemeDef) -> StyleValue,
    pub apply: fn(&mut MainMenuThemeDef, StyleValue),
}

impl StyleKnob {
    pub fn clamp_value(self, value: StyleValue) -> StyleValue {
        match value {
            StyleValue::Number(number) => StyleValue::Number(number.clamp(self.min, self.max)),
        }
    }
}

pub const SEARCH_HEIGHT_KNOB_ID: StyleKnobId = StyleKnobId::new("search.height");

pub const STYLE_KNOBS: &[StyleKnob] = &[StyleKnob {
    id: SEARCH_HEIGHT_KNOB_ID,
    label: "Main input height",
    group: StyleKnobGroup::Search,
    unit: StyleUnit::Px,
    min: 20.0,
    max: 96.0,
    step: 1.0,
    get: get_search_height,
    apply: apply_search_height,
}];

pub fn knob_by_id(id: StyleKnobId) -> Option<&'static StyleKnob> {
    STYLE_KNOBS.iter().find(|knob| knob.id == id)
}

pub fn knob_id_from_str(value: &str) -> Option<StyleKnobId> {
    let normalized = value
        .strip_prefix("control:dev-style-tool:")
        .unwrap_or(value);
    STYLE_KNOBS
        .iter()
        .find(|knob| knob.id.as_str() == normalized)
        .map(|knob| knob.id)
}

fn get_search_height(def: &MainMenuThemeDef) -> StyleValue {
    StyleValue::Number(def.search.height)
}

fn apply_search_height(def: &mut MainMenuThemeDef, value: StyleValue) {
    let StyleValue::Number(height) = value;
    def.search.height = height;
}
