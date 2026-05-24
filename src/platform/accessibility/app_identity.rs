#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ActiveAppIdentity {
    pub name: String,
    pub bundle_id: Option<String>,
    pub process_id: Option<i32>,
}

impl ActiveAppIdentity {
    pub fn unknown() -> Self {
        Self {
            name: "Unknown".to_string(),
            bundle_id: None,
            process_id: None,
        }
    }
}
