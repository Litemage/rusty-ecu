// region virtual-ignition-controller

use ecu_core::engine::{CrankPositionSensor, CylinderOutputs};
use ecu_core::input::{PedalInput, SwitchInput};
use ecu_core::lighting::LightController;

/// Mock cylinder which will hold the state of each cylinder
pub struct VirtualIgnition {
    /// State of each cylinder, held in an array
    pub states: [bool; 4]
}

// Implement the cylinder outputs
impl CylinderOutputs for VirtualIgnition {
    fn set_all(&mut self, states: [bool; 4]) {
        self.states = states;
    }
}

impl VirtualIgnition {
    pub fn new() -> VirtualIgnition { VirtualIgnition {states: [false; 4]}}
}

// endregion

// region virtual-crank-sensor

/// A virtual crank sensor
pub struct VirtualCrank {
    angle_deg: f32
}

// Implement the CrankPositionSensor trait
impl CrankPositionSensor for VirtualCrank {
    fn read_angle(&self) -> f32 {
        return self.angle_deg;
    }
}

impl VirtualCrank {
    pub fn new() -> VirtualCrank { VirtualCrank {angle_deg: 0.0}}

    /// Increment the virtual crank sensor by `deg` degrees
    pub fn increment(&mut self, deg: f32) {
        self.angle_deg += deg;
        // Bound to [0.0, 360.0] degrees
        while self.angle_deg > 360.0 {
            self.angle_deg -= 360.0;
        }
    }
}

// endregion

// region virtual-lights

pub struct VirtualLight {
    pub on: bool
}

impl LightController for VirtualLight {
    fn get_light(&self) -> bool {
        return self.on;
    }

    fn set_light(&mut self, on: bool) {
        self.on = on;
    }
}

// endregion

// region virtual-switch

pub struct VirtualSwitch {
   pub on: bool
}

impl SwitchInput for VirtualSwitch {
    fn read_switch(&self) -> bool {
        return self.on;
    }
}

pub struct VirtualPedal {
    pub val: u8
}

impl PedalInput for VirtualPedal {
    fn read_pedal(&self) -> u8 {
        return self.val;
    }
}

// endregion