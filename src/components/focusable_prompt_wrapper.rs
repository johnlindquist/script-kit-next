//! Shared focus + key routing wrapper for prompt roots.
//!
//! ## Two-level key handler pattern
//!
//! **App-level handlers** (global shortcuts): Escape, Cmd+W, Cmd+K are intercepted first.
//! The `app_key_handler` callback receives the matched key and returns `true` to consume it
//! or `false` to let it fall through to the entity-level handler.
//!
//! **Entity-level handlers** (navigation/input): prompt-specific keys like arrow navigation,
//! tab cycling, enter/submit, and character input. These run only if the app-level handler
//! didn't consume the key.
//!
//! To add a new global shortcut, add a variant to `FocusablePromptInterceptedKey` and
//! match it in `match_focusable_prompt_intercepted_key`.

use gpui::{prelude::*, Context, Div, FocusHandle, Stateful, Window};

#[derive(Clone)]
pub struct FocusablePromptBase {
    pub focus_handle: FocusHandle,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FocusablePromptInterceptedKey {
    Escape,
    CmdW,
    CmdK,
}

#[inline]
pub fn match_focusable_prompt_intercepted_key(
    key: &str,
    has_platform_modifier: bool,
) -> Option<FocusablePromptInterceptedKey> {
    if key.eq_ignore_ascii_case("escape") || key.eq_ignore_ascii_case("esc") {
        return Some(FocusablePromptInterceptedKey::Escape);
    }

    if has_platform_modifier && key.eq_ignore_ascii_case("w") {
        return Some(FocusablePromptInterceptedKey::CmdW);
    }

    if has_platform_modifier && key.eq_ignore_ascii_case("k") {
        return Some(FocusablePromptInterceptedKey::CmdK);
    }

    None
}

pub struct FocusablePrompt {
    entity_render: Stateful<Div>,
    key_context: Option<&'static str>,
}

pub struct FocusablePromptConfigured {
    pub(crate) base: FocusablePromptBase,
    entity_render: Stateful<Div>,
    key_context: Option<&'static str>,
}

crate::impl_focusable_via_base!(FocusablePromptConfigured, base);

impl FocusablePrompt {
    pub fn new(entity_render: Stateful<Div>) -> Self {
        Self {
            entity_render,
            key_context: None,
        }
    }

    pub fn key_context(mut self, key_context: &'static str) -> Self {
        self.key_context = Some(key_context);
        self
    }

    pub fn focus_handle(self, focus_handle: FocusHandle) -> FocusablePromptConfigured {
        FocusablePromptConfigured {
            base: FocusablePromptBase { focus_handle },
            entity_render: self.entity_render,
            key_context: self.key_context,
        }
    }
}

impl FocusablePromptConfigured {
    /// Build the final element with two-level key handling.
    ///
    /// `app_key_handler` receives intercepted keys (Escape/Cmd+W/Cmd+K) first.
    /// Return `true` to consume the key, `false` to fall through.
    ///
    /// `entity_key_handler` receives all other keys (or unconsumed intercepted keys).
    pub fn build<T, AppKeyHandler, EntityKeyHandler>(
        self,
        _window: &Window,
        cx: &mut Context<T>,
        app_key_handler: AppKeyHandler,
        entity_key_handler: EntityKeyHandler,
    ) -> Stateful<Div>
    where
        T: 'static,
        AppKeyHandler: Fn(
                &mut T,
                FocusablePromptInterceptedKey,
                &gpui::KeyDownEvent,
                &mut Window,
                &mut Context<T>,
            ) -> bool
            + 'static,
        EntityKeyHandler: Fn(&mut T, &gpui::KeyDownEvent, &mut Window, &mut Context<T>) + 'static,
    {
        let on_key_down = cx.listener(
            move |this: &mut T,
                  event: &gpui::KeyDownEvent,
                  window: &mut Window,
                  cx: &mut Context<T>| {
                let intercepted_key = match_focusable_prompt_intercepted_key(
                    event.keystroke.key.as_str(),
                    event.keystroke.modifiers.platform,
                );

                if let Some(intercepted_key) = intercepted_key {
                    if app_key_handler(this, intercepted_key, event, window, cx) {
                        cx.stop_propagation();
                        return;
                    }
                }

                entity_key_handler(this, event, window, cx);
            },
        );

        let mut root = self.entity_render;
        if let Some(key_context) = self.key_context {
            root = root.key_context(key_context);
        }

        root.track_focus(&self.base.focus_handle)
            .on_key_down(on_key_down)
    }
}

#[cfg(test)]
mod tests {
    use super::{match_focusable_prompt_intercepted_key, FocusablePromptInterceptedKey};

    #[test]
    fn test_escape_aliases() {
        assert_eq!(
            match_focusable_prompt_intercepted_key("escape", false),
            Some(FocusablePromptInterceptedKey::Escape)
        );
        assert_eq!(
            match_focusable_prompt_intercepted_key("Esc", false),
            Some(FocusablePromptInterceptedKey::Escape)
        );
        assert_eq!(
            match_focusable_prompt_intercepted_key("Escape", false),
            Some(FocusablePromptInterceptedKey::Escape)
        );
    }

    #[test]
    fn test_cmd_shortcuts_require_platform_modifier() {
        assert_eq!(
            match_focusable_prompt_intercepted_key("w", true),
            Some(FocusablePromptInterceptedKey::CmdW)
        );
        assert_eq!(
            match_focusable_prompt_intercepted_key("K", true),
            Some(FocusablePromptInterceptedKey::CmdK)
        );
        assert_eq!(match_focusable_prompt_intercepted_key("w", false), None);
        assert_eq!(match_focusable_prompt_intercepted_key("k", false), None);
    }

    #[test]
    fn test_unrecognized_keys_return_none() {
        assert_eq!(match_focusable_prompt_intercepted_key("enter", false), None);
        assert_eq!(match_focusable_prompt_intercepted_key("tab", false), None);
        assert_eq!(match_focusable_prompt_intercepted_key("a", true), None);
    }
}
