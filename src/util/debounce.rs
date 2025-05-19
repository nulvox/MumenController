//! Button debouncing implementation
//!
//! This module provides a simple debouncer for reducing noise in button inputs.

/// Simple button debouncer
pub struct Debouncer {
    // Current stabilized state (true = pressed, false = released)
    current_state: bool,
    // Counter for consecutive samples in the opposite state
    counter: u8,
    // Threshold for state change (number of consecutive samples)
    threshold: u8,
}

impl Debouncer {
    /// Create a new debouncer with default settings
    pub fn new() -> Self {
        Self {
            current_state: false, // Start in released state
            counter: 0,
            threshold: 3, // Default: 3 consecutive samples to change state
        }
    }
    
    /// Create a new debouncer with custom threshold
    pub fn with_threshold(threshold: u8) -> Self {
        Self {
            current_state: false,
            counter: 0,
            threshold,
        }
    }
    
    /// Update the debouncer with a new sample
    /// Returns the current debounced state
    pub fn update(&mut self, sample: bool) -> bool {
        // If the sample matches the current state, reset the counter
        if sample == self.current_state {
            self.counter = 0;
        } else {
            // Otherwise, increment the counter
            self.counter = self.counter.saturating_add(1);
            
            // If we've seen enough consecutive samples, change state
            if self.counter >= self.threshold {
                self.current_state = sample;
                self.counter = 0;
            }
        }
        
        self.current_state
    }
    
    /// Get the current debounced state without updating
    pub fn state(&self) -> bool {
        self.current_state
    }
}