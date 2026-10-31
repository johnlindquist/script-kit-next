//! A connector must verify its authenticated account before reporting `Connected`.
//! None of the advertised launcher services currently has such a connector.
use sk_one_window::work::WorkEvent;

pub(crate) fn unavailable(integration: String) -> WorkEvent {
    WorkEvent::ConnectFailed {
        integration,
        message: "This service isn't available in the launcher yet. Nothing was connected.".into(),
    }
}
