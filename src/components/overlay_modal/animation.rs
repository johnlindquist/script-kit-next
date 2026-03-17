use std::time::Instant;

use gpui::Context;

use super::types::{compute_overlay_appear_style, OverlayAppearStyle};

/// Mixin trait for overlay modal animation.
///
/// Implementors store `overlay_animation_started_at: Instant` and
/// `overlay_animation_tick_scheduled: bool`, then delegate to the
/// provided default methods.
pub(crate) trait OverlayAnimation: Sized + 'static {
    fn overlay_animation_started_at(&self) -> Instant;
    fn overlay_animation_tick_scheduled(&self) -> bool;
    fn set_overlay_animation_tick_scheduled(&mut self, scheduled: bool);

    fn overlay_appear_style(&self) -> OverlayAppearStyle {
        compute_overlay_appear_style(self.overlay_animation_started_at().elapsed())
    }

    fn schedule_overlay_animation_tick_if_needed(
        &mut self,
        animation_complete: bool,
        cx: &mut Context<Self>,
    ) {
        if animation_complete || self.overlay_animation_tick_scheduled() {
            return;
        }

        self.set_overlay_animation_tick_scheduled(true);
        cx.spawn(async move |this, cx| {
            cx.background_executor()
                .timer(std::time::Duration::from_millis(16))
                .await;
            cx.update(|cx| {
                let _ = this.update(cx, |entity, cx| {
                    entity.set_overlay_animation_tick_scheduled(false);
                    cx.notify();
                });
            });
        })
        .detach();
    }
}
