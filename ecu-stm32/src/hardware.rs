//! Mock ECU hardware interface - owns controller and CPU peripherals. All hardware interraction
//! is done through ECUHardware.
use stm32f7xx_hal::prelude::*;
use stm32f7xx_hal::gpio::{Input, Output, Pin, PinState, PullDown, PushPull};
use stm32f7xx_hal::pac::Peripherals;
use stm32f7xx_hal::timer::SysDelay;
use ecu_core::engine::{CrankPositionSensor, CylinderOutputs, Throttle};
use ecu_core::lighting::LightController;
use ecu_core::input::{PedalInput, SwitchInput};

// region engine-impls

/// Simulated crank position sensor — advances by a fixed amount each loop tick.
/// Replace with a real hall-effect/optical sensor impl when hardware is available.
pub struct SimCrank {
    pub angle_deg: f32,
}

impl CrankPositionSensor for SimCrank {
    fn read_angle(&self) -> f32 {
        self.angle_deg
    }
}

impl SimCrank {
    pub fn increment(&mut self, deg: f32) {
        self.angle_deg = (self.angle_deg + deg) % 360.0;
    }
}

/// No-op cylinder outputs — placeholder until ignition coil drivers are wired up.
pub struct StubCylinders;

impl CylinderOutputs for StubCylinders {
    fn set_all(&mut self, _states: [bool; 4]) {}
}

/// Stub for future implementation of throttle TODO: Implement this.
pub struct StubThrottle;

impl Throttle for StubThrottle {
    fn set_throttle(&mut self, value: u8) {
        // TODO: Implement this
    }
}

// endregion

// region i/o

/// A single push-pull GPIO output implementing `LightController`.
pub struct HwOutputLight<const PORT: char, const PIN: u8>(pub Pin<PORT, PIN, Output<PushPull>>);

impl<const PORT: char, const PIN: u8> LightController for HwOutputLight<PORT, PIN> {
    fn get_light(&self) -> bool {
        self.0.is_set_high()
    }

    fn set_light(&mut self, on: bool) {
        self.0.set_state(PinState::from(on));
    }
}

/// A single pull-down GPIO input implementing `SwitchInput`.
pub struct HwInputSwitch<const PORT: char, const PIN: u8>(pub Pin<PORT, PIN, Input<PullDown>>);

impl<const PORT: char, const PIN: u8> SwitchInput for HwInputSwitch<PORT, PIN> {
    fn read_switch(&self) -> bool {
        self.0.is_high()
    }
}

/// A stub where we will put an input pedal. TODO: Implement this.
pub struct StubInputPedal;

impl PedalInput for StubInputPedal {
    fn read_pedal(&self) -> u8 {
        return 0;
    }
}

// endregion

// region ecu-hardware

/// Owns all peripherals and exposes all hardware interraction the ECU uses
pub struct ECUHardware {
    pub timer: SysDelay,

    // Engine
    pub crank:     SimCrank,
    pub cylinders: StubCylinders,
    pub throttle:  StubThrottle,

    // Outputs
    pub l_turn:          HwOutputLight<'E', 11>,
    pub r_turn:          HwOutputLight<'F', 13>,
    pub headlights_out:  HwOutputLight<'E', 9>,

    // Inputs
    pub l_switch:         HwInputSwitch<'B', 8>,
    pub r_switch:         HwInputSwitch<'B', 9>,
    pub h_switch:         HwInputSwitch<'A', 5>,
    pub headlight_switch: HwInputSwitch<'A', 6>,
    pub accel_pedal:      StubInputPedal,
}

impl ECUHardware {
    /// Initializes a new ECUHardware structure. Takes ownership of all peripherals
    pub fn init(dp: Peripherals, cp: cortex_m::Peripherals) -> Self {
        let rcc = dp.RCC.constrain();
        let clocks = rcc.cfgr.sysclk(216.MHz()).freeze();

        let gpioa = dp.GPIOA.split();
        let gpiob = dp.GPIOB.split();
        let gpioe = dp.GPIOE.split();
        let gpiof = dp.GPIOF.split();

        let delay = cp.SYST.delay(&clocks);

        ECUHardware {
            timer: delay,

            crank:     SimCrank { angle_deg: 0.0 },
            cylinders: StubCylinders,
            throttle:  StubThrottle,

            l_turn:         HwOutputLight(gpioe.pe11.into_push_pull_output()),
            r_turn:         HwOutputLight(gpiof.pf13.into_push_pull_output()),
            headlights_out: HwOutputLight(gpioe.pe9.into_push_pull_output()),

            l_switch:         HwInputSwitch(gpiob.pb8.into_pull_down_input()),
            r_switch:         HwInputSwitch(gpiob.pb9.into_pull_down_input()),
            h_switch:         HwInputSwitch(gpioa.pa5.into_pull_down_input()),
            headlight_switch: HwInputSwitch(gpioa.pa6.into_pull_down_input()),
            accel_pedal:      StubInputPedal,
        }
    }

    /// Use the Cortex-M SysTick delay
    pub fn delay_ms(&mut self, ms: u32) {
        self.timer.delay_ms(ms);
    }
}

// endregion
