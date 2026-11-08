//! Eager ("Prompt-Detection") switch debouncer.
//!
//! ## Latency Design (based on [inputlag.science](https://inputlag.science/controller))
//!
//! Traditional integrating / shift-register debouncers wait for `N` consecutive identical
//! samples before reporting a state transition. At a 1 ms scan rate with `N = 4`, that adds
//! a deterministic **3–4 ms latency penalty** to every button press (as seen in the difference
//! between the Brook UFB Positive Edge at 1.0 ms avg / 95% on-time vs. Negative Edge at 3.9 ms
//! avg / 78% on-time on [inputlag.science/controller/results](https://inputlag.science/controller/results)).
//!
//! `Debouncer` implements **eager (prompt-detection) lockout debouncing**:
//! - When the lock timer is `0` and the raw input differs from the current debounced state,
//!   the state transitions **immediately on that sample (0-tick delay)** and starts a lockout
//!   countdown of `lockout_ticks`.
//! - While the lockout countdown is active (`remaining > 0`), mechanical contact bounce is
//!   ignored and the timer decrements by 1 each scan tick.

/// Eager (prompt-detection) switch debouncer with configurable bounce lockout window.
///
/// Uses 2 bytes per switch (`remaining: u8`, `lockout_ticks: u8`, `state: bool` — 3 bytes total)
/// and zero heap allocation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Debouncer {
    remaining: u8,
    lockout_ticks: u8,
    state: bool,
}

impl Default for Debouncer {
    fn default() -> Self {
        Self::new(4)
    }
}

impl Debouncer {
    /// Create a prompt-detection debouncer that transitions immediately on an edge
    /// and ignores subsequent bounce transitions for `lockout_ticks` scan cycles.
    #[inline]
    pub const fn new(lockout_ticks: u8) -> Self {
        Self {
            remaining: 0,
            lockout_ticks,
            state: false,
        }
    }

    /// Update the debouncer with a new raw reading (`true` = pressed) and return the debounced state.
    ///
    /// - If the lockout window has expired (`remaining == 0`) and `pressed != self.state`,
    ///   transitions immediately (`0` ticks of added latency) and arms the lockout counter.
    /// - If the lockout window is active (`remaining > 0`), decrements `remaining` and holds
    ///   `self.state` steady against contact bounce.
    #[inline]
    pub fn update(&mut self, pressed: bool) -> bool {
        if self.remaining > 0 {
            self.remaining -= 1;
        } else if pressed != self.state {
            self.state = pressed;
            self.remaining = self.lockout_ticks;
        }
        self.state
    }

    /// Return the current debounced state without advancing the lockout counter.
    #[inline]
    pub const fn is_pressed(&self) -> bool {
        self.state
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn prompt_detection_emits_press_and_release_with_zero_tick_delay() {
        let mut d = Debouncer::new(4);
        assert!(!d.is_pressed());

        // Tick 0: switch closes -> reported IMMEDIATELY on the very first tick!
        assert!(d.update(true));
        assert!(d.is_pressed());

        // Ticks 1..=4 (lockout window): contact bounce to false is ignored
        assert!(d.update(false));
        assert!(d.update(true));
        assert!(d.update(false));
        assert!(d.update(true));

        // Lockout expired; switch is now released -> reported IMMEDIATELY on first tick after lockout
        assert!(!d.update(false));

        // Bounce during release lockout window (4 ticks) is ignored
        assert!(!d.update(true));
        assert!(!d.update(false));
        assert!(!d.update(true));
        assert!(!d.update(false));

        // Lockout expired; stays false when idle
        assert!(!d.update(false));
    }

    #[test]
    fn zero_lockout_acts_as_passthrough() {
        let mut d = Debouncer::new(0);
        assert!(d.update(true));
        assert!(!d.update(false));
        assert!(d.update(true));
    }
}
