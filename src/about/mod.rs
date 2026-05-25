//! About screen state and rendering.

pub mod render;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AboutState {
    pub acks_open: bool,
}

impl AboutState {
    pub fn new() -> Self {
        Self { acks_open: false }
    }
}

impl Default for AboutState {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::AboutState;

    #[test]
    fn about_state_starts_with_acknowledgements_collapsed() {
        assert!(!AboutState::new().acks_open);
    }
}
