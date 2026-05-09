use crate::protocol::{AutomationWindowTarget, PixelProbe};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ComputerUseSeeArgs {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub target: Option<AutomationWindowTarget>,
    #[serde(rename = "hiDpi", default, skip_serializing_if = "Option::is_none")]
    pub hi_dpi: Option<bool>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub probes: Vec<PixelProbe>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::protocol::{AutomationWindowTarget, PixelProbe};

    #[test]
    fn see_args_serde_roundtrip_preserves_target_and_probes() {
        let args = ComputerUseSeeArgs {
            target: Some(AutomationWindowTarget::Focused),
            hi_dpi: Some(false),
            probes: vec![PixelProbe { x: 10, y: 20 }],
        };

        let json = serde_json::to_string(&args).expect("serialize");
        assert!(json.contains("hiDpi"));

        let parsed: ComputerUseSeeArgs = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(parsed, args);
    }

    #[test]
    fn see_args_reject_unknown_fields() {
        let error = serde_json::from_value::<ComputerUseSeeArgs>(serde_json::json!({
            "unexpected": true
        }))
        .expect_err("unknown fields should be rejected");

        assert!(error.to_string().contains("unknown field"));
    }
}
