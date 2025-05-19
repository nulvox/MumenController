//! Simultaneous Opposite Cardinal Direction (SOCD) handling
//!
//! This module resolves situations where opposing directions
//! are pressed simultaneously (e.g., left+right, up+down).

use crate::config::SocdConfig;
use log::debug;

/// SOCD resolution methods
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SocdMethod {
    /// Both directions are turned off
    Neutral,
    /// Last input pressed takes priority
    LastWin,
    /// First input pressed takes priority
    FirstWin,
    /// Up takes priority over down (only for up/down)
    UpPriority,
    /// Second directional input overrides the first
    SecondInputPriority,
}

impl From<&'static str> for SocdMethod {
    fn from(s: &'static str) -> Self {
        match s {
            "neutral" => SocdMethod::Neutral,
            "last-win" => SocdMethod::LastWin,
            "first-win" => SocdMethod::FirstWin,
            "up-priority" => SocdMethod::UpPriority,
            "second-input-priority" => SocdMethod::SecondInputPriority,
            _ => SocdMethod::Neutral, // Default to neutral for unknown methods
        }
    }
}

/// SOCD handler for resolving contradictory inputs
pub struct SocdHandler {
    /// Resolution method for left+right
    left_right_method: SocdMethod,
    /// Resolution method for up+down
    up_down_method: SocdMethod,
    /// Last input states for "last-win" and "first-win" methods
    last_states: [bool; 4], // [left, right, up, down]
    /// Input order for priority-based methods
    /// true = first input was left/up, false = first input was right/down
    first_input_order: [bool; 2], // [left/right order, up/down order]
}

impl SocdHandler {
    /// Create a new SOCD handler with default neutral resolution
    pub fn new() -> Self {
        Self {
            left_right_method: SocdMethod::Neutral,
            up_down_method: SocdMethod::UpPriority, // Common default for fighting games
            last_states: [false; 4],
            first_input_order: [true; 2],
        }
    }
    
    /// Create a new SOCD handler with custom resolution methods
    pub fn with_methods(left_right: SocdMethod, up_down: SocdMethod) -> Self {
        Self {
            left_right_method: left_right,
            up_down_method: up_down,
            last_states: [false; 4],
            first_input_order: [true; 2],
        }
    }
    
    /// Create a new SOCD handler with method strings
    pub fn from_strings(left_right: &'static str, up_down: &'static str) -> Self {
        Self::with_methods(
            SocdMethod::from(left_right),
            SocdMethod::from(up_down)
        )
    }
    
    /// Resolve contradictory directional inputs
    ///
    /// Takes inputs for left, right, up, and down
    /// Returns resolved states for each direction
    pub fn resolve(&mut self, left: bool, right: bool, up: bool, down: bool) -> (bool, bool, bool, bool) {
        // Check for left+right conflict
        let (resolved_left, resolved_right) = if left && right {
            self.resolve_left_right(left, right)
        } else {
            // Update input order if only one direction is active
            if left && !right && !self.last_states[0] {
                // Left was just pressed
                self.first_input_order[0] = true; // Left was first
            } else if !left && right && !self.last_states[1] {
                // Right was just pressed
                self.first_input_order[0] = false; // Right was first
            }
            (left, right)
        };
        
        // Check for up+down conflict
        let (resolved_up, resolved_down) = if up && down {
            self.resolve_up_down(up, down)
        } else {
            // Update input order if only one direction is active
            if up && !down && !self.last_states[2] {
                // Up was just pressed
                self.first_input_order[1] = true; // Up was first
            } else if !up && down && !self.last_states[3] {
                // Down was just pressed
                self.first_input_order[1] = false; // Down was first
            }
            (up, down)
        };
        
        // Update last states for next frame
        self.last_states[0] = left;
        self.last_states[1] = right;
        self.last_states[2] = up;
        self.last_states[3] = down;
        
        (resolved_left, resolved_right, resolved_up, resolved_down)
    }
    
    /// Resolve left+right conflict
    fn resolve_left_right(&self, left: bool, right: bool) -> (bool, bool) {
        match self.left_right_method {
            SocdMethod::Neutral => (false, false),
            SocdMethod::LastWin => {
                if self.last_states[0] && !self.last_states[1] {
                    // Left was already active, right was just pressed
                    (false, true)
                } else if !self.last_states[0] && self.last_states[1] {
                    // Right was already active, left was just pressed
                    (true, false)
                } else {
                    // Both were pressed on the same frame or neither was active before
                    // Default to right in this case
                    (false, true)
                }
            },
            SocdMethod::FirstWin => {
                if self.first_input_order[0] {
                    // Left was first
                    (true, false)
                } else {
                    // Right was first
                    (false, true)
                }
            },
            SocdMethod::SecondInputPriority => {
                if self.last_states[0] && !self.last_states[1] {
                    // Left was already active, right was just pressed
                    (false, true)
                } else if !self.last_states[0] && self.last_states[1] {
                    // Right was already active, left was just pressed
                    (true, false)
                } else {
                    // Both were just pressed - treat as neutral
                    (false, false)
                }
            },
            // Up priority doesn't apply to left+right
            _ => (false, false),
        }
    }
    
    /// Resolve up+down conflict
    fn resolve_up_down(&self, up: bool, down: bool) -> (bool, bool) {
        match self.up_down_method {
            SocdMethod::Neutral => (false, false),
            SocdMethod::UpPriority => (true, false), // Up always takes priority
            SocdMethod::LastWin => {
                if self.last_states[2] && !self.last_states[3] {
                    // Up was already active, down was just pressed
                    (false, true)
                } else if !self.last_states[2] && self.last_states[3] {
                    // Down was already active, up was just pressed
                    (true, false)
                } else {
                    // Both were pressed on the same frame or neither was active before
                    // Default to up in this case
                    (true, false)
                }
            },
            SocdMethod::FirstWin => {
                if self.first_input_order[1] {
                    // Up was first
                    (true, false)
                } else {
                    // Down was first
                    (false, true)
                }
            },
            SocdMethod::SecondInputPriority => {
                if self.last_states[2] && !self.last_states[3] {
                    // Up was already active, down was just pressed
                    (false, true)
                } else if !self.last_states[2] && self.last_states[3] {
                    // Down was already active, up was just pressed
                    (true, false)
                } else {
                    // Both were just pressed - treat as neutral
                    (false, false)
                }
            },
        }
    }
    
    /// Convert directional inputs to HAT value for Switch Pro controller
    ///
    /// HAT values:
    /// 0 = North, 1 = North-East, 2 = East, 3 = South-East,
    /// 4 = South, 5 = South-West, 6 = West, 7 = North-West,
    /// 8 = None/Released
    pub fn to_hat_value(&self, up: bool, right: bool, down: bool, left: bool) -> u8 {
        match (up, right, down, left) {
            (true, false, false, false) => 0, // North
            (true, true, false, false) => 1,  // North-East
            (false, true, false, false) => 2, // East
            (false, true, true, false) => 3,  // South-East
            (false, false, true, false) => 4, // South
            (false, false, true, true) => 5,  // South-West
            (false, false, false, true) => 6, // West
            (true, false, false, true) => 7,  // North-West
            _ => 8, // None/Released or invalid combination
        }
    }
}