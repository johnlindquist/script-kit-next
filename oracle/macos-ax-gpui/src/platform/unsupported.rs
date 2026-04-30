use std::{sync::mpsc, time::Duration};

use crate::{
    AxClientOptions, AxError, AxEvent, DisplayInfo, ElementSnapshot, PlatformValue, Point, Rect, Result,
    SettableValue, TreeOptions, WindowInfo, WindowQuery,
};

#[derive(Clone, Debug)]
pub(crate) struct AxClientImpl;

#[derive(Clone, Debug)]
pub(crate) struct AxElementImpl;

#[derive(Debug)]
pub(crate) struct AxObserverImpl;

fn unsupported<T>() -> Result<T> {
    Err(AxError::UnsupportedPlatform {
        platform: std::env::consts::OS,
    })
}

impl AxClientImpl {
    pub fn new(_options: AxClientOptions) -> Result<Self> {
        unsupported()
    }

    pub fn trusted(_prompt: bool) -> Result<bool> {
        unsupported()
    }

    pub fn system_wide(&self) -> Result<AxElementImpl> {
        unsupported()
    }

    pub fn application(&self, _pid: i32) -> Result<AxElementImpl> {
        unsupported()
    }

    pub fn focused_application(&self) -> Result<AxElementImpl> {
        unsupported()
    }

    pub fn focused_element(&self) -> Result<AxElementImpl> {
        unsupported()
    }

    pub fn element_at_position(&self, _point: Point) -> Result<AxElementImpl> {
        unsupported()
    }

    pub fn mouse_location(&self) -> Result<Point> {
        unsupported()
    }

    pub fn window_list(&self, _query: WindowQuery) -> Result<Vec<WindowInfo>> {
        unsupported()
    }

    pub fn active_displays(&self) -> Result<Vec<DisplayInfo>> {
        unsupported()
    }

    pub fn observe_application(
        &self,
        _pid: i32,
        _notifications: Vec<String>,
    ) -> Result<(AxObserverImpl, mpsc::Receiver<AxEvent>)> {
        unsupported()
    }
}

impl AxElementImpl {
    pub fn ptr_eq(&self, _other: &Self) -> bool {
        false
    }

    pub fn pid(&self) -> Result<i32> {
        unsupported()
    }

    pub fn attribute_names(&self) -> Result<Vec<String>> {
        unsupported()
    }

    pub fn parameterized_attribute_names(&self) -> Result<Vec<String>> {
        unsupported()
    }

    pub fn action_names(&self) -> Result<Vec<String>> {
        unsupported()
    }

    pub fn attribute(&self, _attribute: &str) -> Result<Option<PlatformValue>> {
        unsupported()
    }

    pub fn parameterized_attribute(
        &self,
        _attribute: &str,
        _parameter: SettableValue<'_>,
    ) -> Result<Option<PlatformValue>> {
        unsupported()
    }

    pub fn string_attribute(&self, _attribute: &str) -> Result<Option<String>> {
        unsupported()
    }

    pub fn bool_attribute(&self, _attribute: &str) -> Result<Option<bool>> {
        unsupported()
    }

    pub fn element_attribute(&self, _attribute: &str) -> Result<Option<AxElementImpl>> {
        unsupported()
    }

    pub fn children(&self) -> Result<Vec<AxElementImpl>> {
        unsupported()
    }

    pub fn frame(&self) -> Result<Option<Rect>> {
        unsupported()
    }

    pub fn snapshot(&self, _options: TreeOptions) -> Result<ElementSnapshot> {
        unsupported()
    }

    pub fn is_attribute_settable(&self, _attribute: &str) -> Result<bool> {
        unsupported()
    }

    pub fn set_attribute(&self, _attribute: &str, _value: SettableValue<'_>) -> Result<()> {
        unsupported()
    }

    pub fn perform_action(&self, _action: &str) -> Result<()> {
        unsupported()
    }
}

impl AxObserverImpl {
    #[allow(dead_code)]
    pub fn noop_for_docs(_duration: Duration) {}
}
