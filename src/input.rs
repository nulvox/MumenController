// Define the array offsets for each switch
pub static SWITCH_A: usize = 0;
pub static SWITCH_B: usize = 1;
pub static SWITCH_X: usize = 2;
pub static SWITCH_Y: usize = 3;
pub static SWITCH_L1: usize = 4;
pub static SWITCH_R1: usize = 5;
pub static SWITCH_L2: usize = 6;
pub static SWITCH_R2: usize = 7;
pub static SWITCH_SELECT: usize = 8;
pub static SWITCH_START: usize = 9;
pub static SWITCH_HOME: usize = 10;
pub static SWITCH_SHIFT: usize = 11;
pub static SWITCH_UP: usize = 12;
pub static SWITCH_DOWN: usize = 13;
pub static SWITCH_LEFT: usize = 14;
pub static SWITCH_RIGHT: usize = 15;

/// If the switch is a pull-up or pull-down type
#[derive(Debug, Copy, Clone)]
pub enum SwitchType {
    PullUp,
    PullDown,
}

pub enum ButtonName {
    ButtonA,
    ButtonB,
    ButtonX,
    ButtonY,
    ButtonL1,
    ButtonR1,
    ButtonL2,
    ButtonR2,
    ButtonSelect,
    ButtonStart,
    ButtonHome,
    ButtonShift,
    ButtonUp,
    ButtonDown,
    ButtonLeft,
    ButtonRight,
}
