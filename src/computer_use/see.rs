use crate::computer_use::types::ComputerUseSeeArgs;
use crate::protocol::Message;

pub fn build_inspect_message(request_id: String, args: ComputerUseSeeArgs) -> Message {
    Message::InspectAutomationWindow {
        request_id,
        target: args.target,
        hi_dpi: args.hi_dpi,
        probes: args.probes,
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
