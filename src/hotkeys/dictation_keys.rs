//! Exact, temporary hotkeys. Carbon consumes only registered combinations. There is no
//! event tap, modifier polling, or listener for arbitrary typing in the foreground app.
use super::*;
use std::collections::VecDeque;
use std::sync::atomic::AtomicU32;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Control {
    Cancel,
    Switch,
}
#[derive(Clone, Copy)]
struct Binding {
    chord: HotKey,
    control: Control,
}
fn bindings(held: Modifiers, switch: &str) -> Vec<Binding> {
    let held = HotKey::new(Some(held), Code::Escape).mods;
    let switch = switch.parse::<HotKey>().ok().filter(|h| !h.mods.is_empty());
    let cancel = HotKey::new(Some(held), Code::Escape);
    let mut result = Vec::new();
    // Don't turn an ambiguous modified Escape into a destructive cancel. And never
    // request macOS Force Quit, even if the user's launch chord already holds ⌘⌥.
    let safe =
        |h: HotKey| !(h.key == Code::Escape && h.mods.contains(Modifiers::SUPER | Modifiers::ALT));
    let combined = switch.map(|s| HotKey::new(Some(held | s.mods), s.key));
    if safe(cancel) && combined != Some(cancel) {
        result.push(Binding {
            chord: cancel,
            control: Control::Cancel,
        });
    }
    if let Some(chord) = combined.filter(|s| *s != cancel && safe(*s)) {
        result.push(Binding {
            chord,
            control: Control::Switch,
        });
    }
    result
}
struct Registered {
    generation: u64,
    control: Control,
    down: bool,
}
#[derive(Default)]
struct Dispatch {
    registered: HashMap<u32, Registered>,
    pending: VecDeque<(u64, Control)>,
}
static DISPATCH: LazyLock<parking_lot::Mutex<Dispatch>> = LazyLock::new(Default::default);
static NEXT_ID: AtomicU32 = AtomicU32::new(0x8000_0000);
impl Dispatch {
    fn edge(&mut self, id: u32, state: HotKeyState) -> bool {
        let Some(r) = self.registered.get_mut(&id) else {
            return false;
        };
        let pressed = state == HotKeyState::Pressed;
        if pressed && !r.down {
            self.pending.push_back((r.generation, r.control));
        }
        r.down = pressed;
        true
    }
}
pub(super) fn dispatch(id: u32, state: HotKeyState) -> bool {
    DISPATCH.lock().edge(id, state)
}

pub(crate) struct ListeningKeys {
    generation: u64,
    keys: Vec<(HotKey, Control)>,
    pub switch_label: Option<String>,
    pub cancel_available: bool,
    pub switch_unavailable: bool,
    fixture: bool,
    toggle_held: bool,
    configured: String,
}
impl ListeningKeys {
    pub fn install(generation: u64, main_held: bool, switch: &str, fixture: bool) -> Self {
        Self::install_mode(
            generation,
            main_held,
            switch,
            fixture,
            !main_held && !fixture && DICTATION_PTT_PRESSED_AT.lock().is_some(),
        )
    }
    fn install_mode(
        generation: u64,
        main_held: bool,
        switch: &str,
        fixture: bool,
        toggle_held: bool,
    ) -> Self {
        let held = if fixture && main_held {
            Modifiers::SUPER
        } else if main_held || toggle_held {
            let action = if main_held {
                HotkeyAction::Main
            } else {
                HotkeyAction::Dictation
            };
            routes()
                .read()
                .get_builtin_entry(&action)
                .map(|e| e.hotkey.mods)
                .unwrap_or_default()
        } else {
            Modifiers::empty()
        };
        let mut lease = Self {
            generation,
            keys: Vec::new(),
            toggle_held,
            configured: switch.to_owned(),
            switch_label: None,
            cancel_available: false,
            switch_unavailable: switch != "off",
            fixture,
        };
        for b in bindings(held, switch) {
            let mut hk = b.chord;
            if !fixture {
                if crate::runtime_policy::check(
                    crate::runtime_policy::ExternalEffect::GlobalMonitor,
                )
                .is_err()
                {
                    continue;
                }
                let Some(manager) = MAIN_MANAGER.get().and_then(|m| m.lock().ok()) else {
                    continue;
                };
                if routes().read().routes.contains_key(&hk.id()) {
                    continue;
                }
                let Ok(id) = NEXT_ID
                    .fetch_update(Ordering::Relaxed, Ordering::Relaxed, |i| i.checked_add(1))
                else {
                    continue;
                };
                hk.id = id;
                if manager.register(hk).is_err() {
                    continue;
                }
                DISPATCH.lock().registered.insert(
                    id,
                    Registered {
                        generation,
                        control: b.control,
                        down: false,
                    },
                );
            }
            lease.keys.push((hk, b.control));
            match b.control {
                Control::Cancel => lease.cancel_available = true,
                Control::Switch => {
                    lease.switch_label = Some(if switch.eq_ignore_ascii_case("shift+escape") {
                        "⇧Esc".into()
                    } else {
                        switch.into()
                    });
                    lease.switch_unavailable = false;
                }
            }
        }
        lease
    }
    pub fn after_toggle_release(&mut self) -> bool {
        if !self.toggle_held || DICTATION_PTT_PRESSED_AT.lock().is_some() {
            return false;
        }
        let switch = self.configured.clone();
        self.release();
        *self = Self::install_mode(self.generation, false, &switch, self.fixture, false);
        true
    }
    pub fn finishing(&mut self) {
        let g = self.generation;
        let f = self.fixture;
        self.release();
        *self = Self::install_mode(g, false, "off", f, false);
    }
    pub fn take_pending(&self) -> Vec<Control> {
        let mut d = DISPATCH.lock();
        let mut out = Vec::new();
        d.pending.retain(|(g, c)| {
            if *g == self.generation {
                out.push(*c);
                false
            } else {
                true
            }
        });
        out
    }
    pub fn fixture_control(&self, key: &str, m: gpui::Modifiers) -> Option<Control> {
        if !self.fixture {
            return None;
        }
        let mut mods = Modifiers::empty();
        for (down, flag) in [
            (m.shift, Modifiers::SHIFT),
            (m.platform, Modifiers::SUPER),
            (m.alt, Modifiers::ALT),
            (m.control, Modifiers::CONTROL),
        ] {
            if down {
                mods |= flag;
            }
        }
        let (_, code) = shortcuts::parse_shortcut(key)?;
        self.keys
            .iter()
            .find(|(h, _)| h.matches(mods, code))
            .map(|(_, c)| *c)
    }
}
impl Drop for ListeningKeys {
    fn drop(&mut self) {
        self.release();
    }
}
impl ListeningKeys {
    fn release(&mut self) {
        if self.fixture || self.keys.is_empty() {
            return;
        }
        let manager = MAIN_MANAGER.get().and_then(|m| m.lock().ok());
        let mut d = DISPATCH.lock();
        for (key, _) in self.keys.drain(..) {
            d.registered.remove(&key.id());
            if let Some(m) = &manager {
                if m.unregister(key).is_err() {
                    tracing::error!("Could not release temporary dictation shortcut");
                }
            }
        }
        d.pending.retain(|(g, _)| *g != self.generation);
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn only_exact_chords_for_this_recording() {
        for held in [Modifiers::empty(), Modifiers::SUPER] {
            let keys = bindings(held, "Shift+Escape");
            assert_eq!(keys.len(), 2);
            assert_eq!(keys[0].chord.mods, held);
            assert_eq!(keys[1].chord.mods, held | Modifiers::SHIFT);
            assert!(keys.iter().all(|b| b.chord.key == Code::Escape));
        }
        assert!(bindings(Modifiers::SUPER | Modifiers::SHIFT, "Shift+Escape").is_empty());
        assert!(bindings(Modifiers::SUPER | Modifiers::ALT, "Shift+Escape").is_empty());
        assert_eq!(bindings(Modifiers::empty(), "off").len(), 1);
        assert_eq!(
            bindings(Modifiers::empty(), "Alt+F12")[1].chord.key,
            Code::F12
        );
    }
    #[test]
    fn release_and_repeat_never_cancel_or_switch_twice() {
        let mut d = Dispatch::default();
        d.registered.insert(
            91,
            Registered {
                generation: 7,
                control: Control::Switch,
                down: false,
            },
        );
        for edge in [
            HotKeyState::Pressed,
            HotKeyState::Pressed,
            HotKeyState::Released,
        ] {
            d.edge(91, edge);
        }
        assert_eq!(
            d.pending.into_iter().collect::<Vec<_>>(),
            [(7, Control::Switch)]
        );
    }
}
