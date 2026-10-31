//! Owned renderer fixture only. No OS handles, spawned helper, or operator state.
use super::*;
use serde_json::json;
pub const SENTENCE: &str = "Name this export “Workshop outline”; don’t save.";
pub const APP: &str = "Workshop Editor";
pub const TITLE: &str = "Lesson draft";
pub const PID: u32 = 9401;
pub const WINDOW: u32 = 42;
struct Owned(Mutex<String>);
impl Backend for Owned {
    fn windows(&self, _: &str) -> Result<Vec<Window>, Failure> {
        Ok(vec![Window {
            app: APP.into(),
            pid: PID,
            window_id: "w-42".into(),
            title: TITLE.into(),
            process_instance: "fixture:1".into(),
        }])
    }
    fn sheet(&self, _: &Window) -> Result<Value, Failure> {
        Ok(
            json!({"app":APP,"window":{"id":"w-42"},"complete":true,"snapshot_id":"s42","tree":{"role":"sheet","children":[{"role":"textfield","name":"Save As:","ref_id":"@s42:e1","available_actions":["SetValue"]},{"role":"button","name":"Save","ref_id":"@s42:e2","available_actions":["Click"]}]}}),
        )
    }
    fn value(&self, r: &str) -> Result<String, Failure> {
        if r != "@s42:e1" {
            return Err("window_changed");
        }
        Ok(self.0.lock().clone())
    }
    fn set_value(&self, r: &str, value: &str) -> Result<(), Failure> {
        self.value(r)?;
        *self.0.lock() = value.into();
        crate::runtime_policy::record_completed_fixture_effect();
        Ok(())
    }
}
pub(crate) fn api() -> &'static dyn FilenameApi {
    static S: LazyLock<Service<Owned>> =
        LazyLock::new(|| Service::new(Owned(Mutex::new("Untitled".into()))));
    &*S
}
