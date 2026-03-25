//! This library contains the core functionality for the `rusty-ecu` project which shall be
//! implemented by an embedded system.
// When running tests, link against standard. When not running tests (building
// for ARM, don't link against standard)
#![cfg_attr(not(test), no_std)]

// region module-imports

use crate::engine::engine_update;
use crate::lighting::signal_for_time;

pub mod engine;
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
    NONE
}

pub struct ECUSettings {
    pub signal_blink_period: u32
}

/// All ECU state that persists across update loops
pub struct ECUState {
    /// Active turn signal mode
    pub sig: Signal,
    /// True if headlights are requested
    pub headlights: bool,
}

impl ECUState {
    pub fn new() -> ECUState {
        ECUState {
            sig: Signal::NONE,
            headlights: false
        }
    }
}

/// Primary function of this library - called periodically to run the ECU
pub fn ecu_update(
    get_time_ms: impl Fn() -> u64,
    crank_sensor: &impl engine::CrankPositionSensor,
    c_outputs: &mut impl engine::CylinderOutputs,
    l_turn: &mut impl lighting::LightController,
    r_turn: &mut impl lighting::LightController,
    headlights: &mut impl lighting::LightController,
    ecu_state: &mut ECUState,
    ecu_settings: &ECUSettings
) {
    engine_update(crank_sensor, c_outputs);

    let ts = get_time_ms();
    // Current blink state, used for either left or right turn signal
    // This is super cheap to calculate, so no harm doing it every loop
    let blink = signal_for_time(ts, ecu_settings.signal_blink_period);

    // Handle headlights
    headlights.set_light(ecu_state.headlights);

    // Handle Turn Signals
    match ecu_state.sig {
        Signal::RIGHT => {
            l_turn.set_light(false);
            r_turn.set_light(blink);
        }
        Signal::LEFT => {
            l_turn.set_light(blink);
            r_turn.set_light(false);
        }
        Signal::HAZARD => {
            l_turn.set_light(blink);
            r_turn.set_light(blink);
        }
        Signal::NONE => {
            // Both signals should be off
            l_turn.set_light(false);
            r_turn.set_light(false);
        }
    }
}

// endregion