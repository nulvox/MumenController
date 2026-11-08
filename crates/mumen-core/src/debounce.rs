//! Compact shift-register switch debouncer.

/// Shift-register debouncer requiring `N` consecutive identical samples
/// before transitioning the debounced state.
///
/// Uses 2 bytes per switch and zero heap allocation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Debouncer {
    history: u8,
    mask: u8,
    state: bool,
}

impl Default for Debouncer {
    fn default() -> Self {
        Self::new(4)
    }
}

impl Debouncer {
    /// Create a debouncer requiring `samples` (`1..=8`) consecutive matching reads.
    #[inline]
    pub const fn new(samples: u8) -> Self {
        let clamped = if samples == 0 {
            1
        } else if samples > 8 {
            8
        } else {
            samples
        };
        let mask = if clamped == 8 {
            0xFF
        } else {
            (1u8 << clamped) - 1
        };
        Self {
            history: 0,
            mask,
            state: false,
        }
    }

    /// Push a new raw reading (`true` = pressed) and return the debounced state.
    #[inline]
    pub fn update(&mut self, pressed: bool) -> bool {
        self.history = ((self.history << 1) | (pressed as u8)) & self.mask;
        if self.history == self.mask {
            self.state = true;
        } else if self.history == 0 {
            self.state = false;
        }
        self.state
    }

    /// Return the current debounced state without updating.
    #[inline]
    pub const fn is_pressed(&self) -> bool {
        self.state
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn debouncer_rejects_glitches_and_transitions_after_threshold() {
        let mut d = Debouncer::new(4);
        assert!(!d.is_pressed());

        // 3 consecutive trues then a glitch false -> stays false
        assert!(!d.update(true));
        assert!(!d.update(true));
        assert!(!d.update(true));
        assert!(!d.update(false));

        // 4 consecutive trues -> becomes true
        assert!(!d.update(true));
        assert!(!d.update(true));
        assert!(!d.update(true));
        assert!(d.update(true));

        // Single false glitch while pressed -> stays true
        assert!(d.update(false));
        assert!(d.update(true));

        // 4 consecutive falses -> becomes false
        assert!(d.update(false));
        assert!(d.update(false));
        assert!(d.update(false));
        assert!(!d.update(false));
    }
}
