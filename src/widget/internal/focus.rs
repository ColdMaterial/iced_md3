use std::time::Duration;

use iced::time::Instant;

/// How long the caret stays in each blink phase.
pub(crate) const CURSOR_BLINK_INTERVAL: Duration = Duration::from_millis(500);

/// Bookkeeping for a blinking caret: when it last moved (or the widget gained
/// focus), the time of the current frame, and whether the window has focus.
///
/// Shared by [`TextInput`](crate::widget::TextInput) and the search field of
/// [`Select`](crate::widget::Select) so both blink the same way.
#[derive(Debug, Clone, Copy)]
pub struct Focus {
    /// When the caret last moved or the widget last gained focus; the blink
    /// phase starts here.
    pub updated_at: Instant,
    /// Time of the most recent frame, refreshed on every animation tick.
    pub now: Instant,
    /// Whether the window hosting the widget currently has focus.
    pub is_window_focused: bool,
}

impl Focus {
    /// Creates the blink bookkeeping, anchored at `now`.
    pub fn new(now: Instant) -> Self {
        Self {
            updated_at: now,
            now,
            is_window_focused: true,
        }
    }

    /// Restarts the blink phase so the caret is visible right after an
    /// interaction.
    pub fn restart(&mut self, now: Instant) {
        self.updated_at = now;
        self.now = now;
    }

    /// Whether the caret should currently be drawn.
    ///
    /// This folds in the window focus check, so callers only need to handle
    /// their own enabled/focused conditions.
    pub fn is_caret_visible(&self) -> bool {
        let elapsed = self.now.saturating_duration_since(self.updated_at);

        self.is_window_focused
            && (elapsed.as_millis() / CURSOR_BLINK_INTERVAL.as_millis()).is_multiple_of(2)
    }

    /// Milliseconds until the caret flips phase, for scheduling a redraw.
    #[must_use]
    pub fn millis_until_blink(&self) -> u128 {
        let elapsed = self
            .now
            .saturating_duration_since(self.updated_at)
            .as_millis();
        let interval = CURSOR_BLINK_INTERVAL.as_millis();

        interval - elapsed % interval
    }
}
