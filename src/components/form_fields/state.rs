use std::sync::{Arc, Mutex};

/// Shared state for form field values
///
/// This allows parent components to access field values for form submission
#[derive(Clone)]
pub struct FormFieldState {
    value: Arc<Mutex<String>>,
}

impl FormFieldState {
    /// Create a new form field state with an initial value
    pub fn new(initial_value: String) -> Self {
        Self {
            value: Arc::new(Mutex::new(initial_value)),
        }
    }

    /// Get the current value
    pub fn get_value(&self) -> String {
        self.value.lock().unwrap_or_else(|e| e.into_inner()).clone()
    }

    /// Set the value
    pub fn set_value(&self, value: String) {
        *self.value.lock().unwrap_or_else(|e| e.into_inner()) = value;
    }
}
