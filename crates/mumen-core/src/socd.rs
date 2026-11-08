//! Simultaneous Opposing Cardinal Directions (SOCD) resolution.

use crate::hid::Hat;

/// Raw directional switch states before SOCD cleaning.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct DpadRaw {
    pub up: bool,
    pub down: bool,
    pub left: bool,
    pub right: bool,
}

/// Supported SOCD resolution strategies for an opposing axis pair.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SocdMethod {
    /// Opposing directions cancel out to neutral (`Left + Right -> None`, `Up + Down -> None`).
    Neutral,
    /// Negative/primary direction wins (`Up` over `Down`, `Left` over `Right`).
    PrimaryPriority,
    /// Last pressed direction wins.
    LastWin,
}

/// Stateful SOCD cleaner for horizontal (`Left`/`Right`) and vertical (`Up`/`Down`) axes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SocdState {
    pub horizontal: SocdMethod,
    pub vertical: SocdMethod,
    prev_raw: DpadRaw,
    last_horizontal_right: bool,
    last_vertical_down: bool,
}

impl Default for SocdState {
    fn default() -> Self {
        Self::new(SocdMethod::Neutral, SocdMethod::PrimaryPriority)
    }
}

impl SocdState {
    /// Standard tournament/HitBox SOCD: Left+Right = Neutral, Up+Down = Up (`PrimaryPriority`).
    #[inline]
    pub const fn new(horizontal: SocdMethod, vertical: SocdMethod) -> Self {
        Self {
            horizontal,
            vertical,
            prev_raw: DpadRaw {
                up: false,
                down: false,
                left: false,
                right: false,
            },
            last_horizontal_right: false,
            last_vertical_down: false,
        }
    }

    /// Tekken / legacy `spc.rs` table mode: Left+Right = Left, Up+Down = Up.
    #[inline]
    pub const fn left_and_up_priority() -> Self {
        Self::new(SocdMethod::PrimaryPriority, SocdMethod::PrimaryPriority)
    }

    /// Resolve opposing directions according to the configured horizontal and vertical methods.
    pub fn resolve(&mut self, raw: DpadRaw) -> DpadRaw {
        // Track most recently pressed direction on each axis for LastWin
        if raw.left && !self.prev_raw.left && (!raw.right || self.prev_raw.right) {
            self.last_horizontal_right = false;
        } else if raw.right && !self.prev_raw.right && (!raw.left || self.prev_raw.left) {
            self.last_horizontal_right = true;
        }

        if raw.up && !self.prev_raw.up && (!raw.down || self.prev_raw.down) {
            self.last_vertical_down = false;
        } else if raw.down && !self.prev_raw.down && (!raw.up || self.prev_raw.up) {
            self.last_vertical_down = true;
        }

        self.prev_raw = raw;

        let (left, right) = match (raw.left, raw.right) {
            (true, true) => match self.horizontal {
                SocdMethod::Neutral => (false, false),
                SocdMethod::PrimaryPriority => (true, false),
                SocdMethod::LastWin => {
                    if self.last_horizontal_right {
                        (false, true)
                    } else {
                        (true, false)
                    }
                }
            },
            other => other,
        };

        let (up, down) = match (raw.up, raw.down) {
            (true, true) => match self.vertical {
                SocdMethod::Neutral => (false, false),
                SocdMethod::PrimaryPriority => (true, false),
                SocdMethod::LastWin => {
                    if self.last_vertical_down {
                        (false, true)
                    } else {
                        (true, false)
                    }
                }
            },
            other => other,
        };

        DpadRaw {
            up,
            down,
            left,
            right,
        }
    }

    /// Resolve raw directions and convert directly to a [`Hat`] switch value.
    #[inline]
    pub fn resolve_to_hat(&mut self, raw: DpadRaw) -> Hat {
        let cleaned = self.resolve(raw);
        Hat::from_directions(cleaned.up, cleaned.down, cleaned.left, cleaned.right)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_hitbox_socd_neutral_horizontal_and_up_priority() {
        let mut socd = SocdState::default();

        // Left + Right -> Neutral
        let res = socd.resolve(DpadRaw {
            up: false,
            down: false,
            left: true,
            right: true,
        });
        assert_eq!(
            res,
            DpadRaw {
                up: false,
                down: false,
                left: false,
                right: false
            }
        );

        // Up + Down -> Up
        let res = socd.resolve(DpadRaw {
            up: true,
            down: true,
            left: false,
            right: false,
        });
        assert_eq!(
            res,
            DpadRaw {
                up: true,
                down: false,
                left: false,
                right: false
            }
        );

        // All 4 pressed -> Up
        assert_eq!(
            socd.resolve_to_hat(DpadRaw {
                up: true,
                down: true,
                left: true,
                right: true,
            }),
            Hat::Up
        );
    }

    #[test]
    fn last_win_tracks_latest_edge() {
        let mut socd = SocdState::new(SocdMethod::LastWin, SocdMethod::LastWin);

        // Hold Left
        let r1 = socd.resolve(DpadRaw {
            left: true,
            ..Default::default()
        });
        assert!(r1.left && !r1.right);

        // Press Right while still holding Left -> Right wins
        let r2 = socd.resolve(DpadRaw {
            left: true,
            right: true,
            ..Default::default()
        });
        assert!(!r2.left && r2.right);

        // Release Left, then press Left again while holding Right -> Left wins
        let _ = socd.resolve(DpadRaw {
            left: false,
            right: true,
            ..Default::default()
        });
        let r3 = socd.resolve(DpadRaw {
            left: true,
            right: true,
            ..Default::default()
        });
        assert!(r3.left && !r3.right);
    }
}
