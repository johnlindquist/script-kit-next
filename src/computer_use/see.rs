use crate::computer_use::types::{
    ComputerUseObservationEnvelope, ComputerUseSeeArgs, COMPUTER_USE_SCHEMA_VERSION,
};
use crate::protocol::{AutomationInspectSnapshot, AutomationWindowTarget, Message};

pub fn build_inspect_message(request_id: String, args: ComputerUseSeeArgs) -> Message {
    Message::InspectAutomationWindow {
        request_id,
        target: args.target,
        hi_dpi: args.hi_dpi,
        probes: args.probes,
    }
}

pub fn envelope_from_snapshot(
    target: Option<AutomationWindowTarget>,
    snapshot: AutomationInspectSnapshot,
) -> ComputerUseObservationEnvelope {
    ComputerUseObservationEnvelope {
        schema_version: COMPUTER_USE_SCHEMA_VERSION,
        action: "see".to_string(),
        target,
        observation: snapshot,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::protocol::{AutomationWindowTarget, PixelProbe};

    #[test]
    fn build_inspect_message_uses_existing_protocol_variant() {
        let message = build_inspect_message(
            "cu-see-1".to_string(),
            ComputerUseSeeArgs {
                target: Some(AutomationWindowTarget::Focused),
                hi_dpi: Some(false),
                probes: vec![PixelProbe { x: 10, y: 20 }],
                max_elements: None,
            },
        );

        match message {
            Message::InspectAutomationWindow {
                request_id,
                target,
                hi_dpi,
                probes,
            } => {
                assert_eq!(request_id, "cu-see-1");
                assert_eq!(target, Some(AutomationWindowTarget::Focused));
                assert_eq!(hi_dpi, Some(false));
                assert_eq!(probes, vec![PixelProbe { x: 10, y: 20 }]);
            }
            other => panic!("expected InspectAutomationWindow, got {other:?}"),
        }
    }
}
