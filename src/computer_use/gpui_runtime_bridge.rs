use crate::computer_use::runtime_bridge::{
    ComputerUseInspectRequest, ComputerUseRuntimeBridge, ComputerUseRuntimeError,
};
use crate::protocol::AutomationInspectSnapshot;
use std::sync::mpsc::{self, SyncSender};
use std::sync::RwLock;
use std::time::Duration;

pub struct GpuiComputerUseRuntimeBridge {
    sender: RwLock<Option<async_channel::Sender<GpuiComputerUseRequest>>>,
    timeout: Duration,
}

pub struct GpuiComputerUseRequest {
    pub request_id: String,
    pub request: ComputerUseInspectRequest,
    response_tx: SyncSender<Result<AutomationInspectSnapshot, ComputerUseRuntimeError>>,
}

impl GpuiComputerUseRuntimeBridge {
    pub fn new(timeout: Duration) -> Self {
        Self {
            sender: RwLock::new(None),
            timeout,
        }
    }

    pub fn install(&self, sender: async_channel::Sender<GpuiComputerUseRequest>) {
        if let Ok(mut guard) = self.sender.write() {
            *guard = Some(sender);
        }
    }

    pub fn clear(&self) {
        if let Ok(mut guard) = self.sender.write() {
            *guard = None;
        }
    }
}

impl GpuiComputerUseRequest {
    pub fn respond(self, result: Result<AutomationInspectSnapshot, ComputerUseRuntimeError>) {
        let _ = self.response_tx.send(result);
    }
}

impl ComputerUseRuntimeBridge for GpuiComputerUseRuntimeBridge {
    fn inspect_automation_window(
        &self,
        request: ComputerUseInspectRequest,
    ) -> Result<AutomationInspectSnapshot, ComputerUseRuntimeError> {
        let sender = self
            .sender
            .read()
            .ok()
            .and_then(|guard| guard.clone())
            .ok_or(ComputerUseRuntimeError::Unavailable)?;

        let request_id = format!("mcp-computer-see:{}", uuid::Uuid::new_v4());
        let (response_tx, response_rx) = mpsc::sync_channel(1);
        sender
            .try_send(GpuiComputerUseRequest {
                request_id,
                request,
                response_tx,
            })
            .map_err(|_| ComputerUseRuntimeError::Unavailable)?;

        response_rx
            .recv_timeout(self.timeout)
            .map_err(|_| ComputerUseRuntimeError::Timeout)?
    }
}
