//! Defines input logic to the ECU, such as switches, buttons, and throttle

/// Generic switch input, either on or off
pub trait SwitchInput {
    /// Returns the current state of the switch by either reading form hardware, or a cached value.
    fn read_switch(&self) -> bool;
}

pub trait PedalInput {
    /// Proportional value representing the proportion a pedal is "on", between 0 and 256
    fn read_pedal(&self) -> u8;
}