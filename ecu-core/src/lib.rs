//! This library contains the core functionality for the `rusty-ecu` project which shall be
//! implemented by an embedded system.
// When running tests, link against standard. When not running tests (building
// for ARM, don't link against standard)
#![cfg_attr(not(test), no_std)]

use crate::engine::engine_update;
use crate::input::SwitchInput;
use crate::lighting::{LightController, signal_for_time};

// region module-imports

pub mod engine;
pub mod input;
pub mod lighting;

// endregion

// region ecu

#[derive(PartialEq)]
/// Turn signal possible states
pub enum Signal {
    RIGHT,
    LEFT,
    HAZARD,
    // Both turn signals off when "NONE"
    NONE,
}

pub struct ECUSettings {
    pub signal_blink_period: u32,
}

/// All ECU state that persists across update loops
pub struct ECUState {
    /// Active turn signal mode
    sig: Signal,
    /// True if headlights are requested
    headlights: bool,
}

impl ECUState {
    pub fn new() -> ECUState {
        ECUState {
            sig: Signal::NONE,
            headlights: false,
        }
    }
}

fn ecu_update_state(
    ecu_state: &mut ECUState,
    l_sig_switch: &impl SwitchInput,
    r_sig_switch: &impl SwitchInput,
    h_sig_switch: &impl SwitchInput,
    headlights: &impl SwitchInput,
) {
    let l_switch = l_sig_switch.read_switch();
    let r_switch = r_sig_switch.read_switch();
    let h_switch = h_sig_switch.read_switch();
    let headlight = headlights.read_switch();

    // Turn signals
    if h_switch {
        // Hazzard switch takes overall priority
        ecu_state.sig = Signal::HAZARD;
    } else if l_switch && r_switch {
        // This state is invalid - no light should be signaled
        ecu_state.sig = Signal::NONE;
    } else if l_switch {
        ecu_state.sig = Signal::LEFT;
    } else if r_switch {
        ecu_state.sig = Signal::RIGHT;
    } else {
        ecu_state.sig = Signal::NONE;
    }

    // Headlights
    ecu_state.headlights = headlight;
}

fn ecu_update_turn_signals(
    ecu_state: &mut ECUState,
    l_signal: &mut impl LightController,
    r_signal: &mut impl LightController,
    blink: bool,
) {
    match ecu_state.sig {
        Signal::RIGHT => {
            l_signal.set_light(false);
            r_signal.set_light(blink);
        }
        Signal::LEFT => {
            l_signal.set_light(blink);
            r_signal.set_light(false);
        }
        Signal::HAZARD => {
            l_signal.set_light(blink);
            r_signal.set_light(blink);
        }
        Signal::NONE => {
            // Both signals should be off
            l_signal.set_light(false);
            r_signal.set_light(false);
        }
    }
}

/// Primary function of this library - called periodically to run the ECU
pub fn ecu_update(
    get_time_ms: impl Fn() -> u64,
    crank_sensor: &impl engine::CrankPositionSensor,
    c_outputs: &mut impl engine::CylinderOutputs,
    throttle: &mut impl engine::Throttle,
    l_turn: &mut impl lighting::LightController,
    r_turn: &mut impl lighting::LightController,
    headlights: &mut impl lighting::LightController,
    l_switch: &mut impl input::SwitchInput,
    r_switch: &mut impl input::SwitchInput,
    h_switch: &mut impl input::SwitchInput,
    headlight_switch: &mut impl input::SwitchInput,
    accel_pedal: &mut impl input::PedalInput,
    ecu_state: &mut ECUState,
    ecu_settings: &ECUSettings,
) {
    engine_update(crank_sensor, c_outputs, throttle, accel_pedal);

    // Updates the ECU state with button/switch input collected from the embedded system
    ecu_update_state(ecu_state, l_switch, r_switch, h_switch, headlight_switch);

    let ts = get_time_ms();
    // Current blink state, used for either left or right turn signal
    // This is super cheap to calculate, so no harm doing it every loop
    let blink = signal_for_time(ts, ecu_settings.signal_blink_period);

    // Handle headlights
    headlights.set_light(ecu_state.headlights);

    // Handle Turn Signals
    ecu_update_turn_signals(ecu_state, l_turn, r_turn, blink);
}

// endregion
