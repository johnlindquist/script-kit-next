//! A dedicated writer's profile grant is not a person's approval of a particular edit.
//! Goose may skip ACP permission requests (e.g. a cached grant). The actual MCP write
//! therefore consumes the identical, once-approved arguments, or refuses before storage.
use super::REGISTRY;
use parking_lot::Mutex;
use serde_json::Value;
use std::sync::Arc;

#[derive(Debug, Default)]
enum Slot {
    #[default]
    Idle,
    Pending(u64),
    Approved(Value),
    Closed,
}
#[derive(Debug, Default)]
pub(crate) struct LauncherReviewTurn(Mutex<Slot>);
impl LauncherReviewTurn {
    pub(crate) fn present(&self, id: u64) -> bool {
        let mut slot = self.0.lock();
        if matches!(*slot, Slot::Closed) {
            return false;
        }
        *slot = Slot::Pending(id);
        true
    }
    pub(crate) fn resolve(&self, id: u64, approved: Option<Value>) -> bool {
        let mut slot = self.0.lock();
        if !matches!(*slot, Slot::Pending(current) if current == id) {
            return false;
        }
        *slot = approved.map_or(Slot::Idle, Slot::Approved);
        true
    }
    pub(crate) fn close(&self) {
        *self.0.lock() = Slot::Closed;
    }
    fn take(&self, args: &Value) -> bool {
        let mut slot = self.0.lock();
        if !matches!(&*slot, Slot::Approved(_)) {
            return false;
        }
        matches!(std::mem::take(&mut *slot), Slot::Approved(reviewed) if reviewed == *args)
    }
}

pub(crate) fn start_launcher_review(policy_id: &str) -> Option<Arc<LauncherReviewTurn>> {
    let mut registry = REGISTRY.lock();
    let previous = registry.get_mut(policy_id)?.launcher_review.as_mut()?;
    if let Some(old) = previous.upgrade() {
        old.close();
    }
    let turn = Arc::new(LauncherReviewTurn::default());
    *previous = Arc::downgrade(&turn);
    Some(turn)
}
pub(super) fn claim(id: Option<&str>, args: &Value) -> Result<(), &'static str> {
    let registry = REGISTRY.lock();
    let state = id.and_then(|id| registry.get(id)).ok_or("scope_denied")?;
    if !state
        .policy
        .allows(sk_personal::api::Endpoint::Apply.profile_tool())
    {
        return Err("scope_denied");
    }
    match &state.launcher_review {
        None => Ok(()),
        Some(weak) if weak.upgrade().is_some_and(|turn| turn.take(args)) => Ok(()),
        Some(_) => Err("launcher_review_required"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mcp_agent_tools::{
        handle_rpc, register_bound_policy, unregister_policy, AgentRpcReply, AgentToolPolicy,
    };
    use serde_json::json;
    #[test]
    fn only_the_exact_live_one_time_approval_unlocks_a_dedicated_writer() {
        let policy = || AgentToolPolicy {
            allowed_tools: vec!["launcher_read".into(), "launcher_apply".into()],
            cwd: Default::default(),
            read_roots: Some(vec![]),
            write_roots: Some(vec![]),
            web_search_budget: None,
        };
        let id = register_bound_policy(policy(), true);
        let other = register_bound_policy(policy(), true);
        let a = json!({"op":"apply","request":{"requestId":"first","change":{"op":"edit","edit":{"target":"launcher.header.pinned-label.weight","value":"semiboldWhileDictating"}}}});
        let mut b = a.clone();
        b["request"]["change"]["edit"]["value"] = json!("semibold");
        let denied = || {
            let AgentRpcReply::Json(s)=handle_rpc(Some(&id),&json!({"id":1,"method":"tools/call","params":{"name":"launcher_apply","arguments":a}}).to_string(),None) else {panic!("missing tool reply")};
            assert!(s.contains("launcher_review_required"));
        };
        denied();
        let first = start_launcher_review(&id).unwrap();
        denied();
        assert!(first.present(1));
        assert!(first.present(2)); // The newer question replaced the older one in the same turn.
        assert!(!first.resolve(1, Some(a.clone())));
        assert!(first.resolve(2, None));
        denied();
        assert!(first.present(3));
        assert!(first.resolve(3, Some(a.clone())));
        assert!(claim(Some(&other), &a).is_err());
        assert!(claim(Some(&id), &b).is_err());
        denied();
        assert!(first.present(4));
        assert!(first.resolve(4, Some(a.clone())));
        let next = start_launcher_review(&id).unwrap();
        assert!(!first.present(5));
        denied();
        assert!(next.present(6));
        assert!(next.resolve(6, Some(a.clone())));
        assert!(claim(Some(&id), &a).is_ok());
        denied();
        assert!(next.present(7));
        assert!(next.resolve(7, Some(a)));
        next.close();
        denied();
        assert!(!next.present(8));
        unregister_policy(&id);
        unregister_policy(&other);
    }
}
