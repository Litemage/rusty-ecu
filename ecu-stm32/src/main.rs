//! Mock-ECU program designed to test the viability of embedded rust. Interfaces with a test bench
//! that will visualize all the I/O. Logic lives in ecu-core library.
#![no_std] // Don't link standard lib
#![no_main] // Don't use standard entry point

mod hardware;

use cortex_m_rt::entry;
use ecu_core::{ECUSettings, ECUState, ecu_update};
use hardware::ECUHardware;
use panic_halt as _;
use rtt_target::{rprintln, rtt_init_print}; // Required for the panic handler

/// The delay executed at the end of each loop
const LOOP_PERIOD_MS: u32 = 10;
/// Degrees to advance the simulated crank each loop tick.
/// At 5ms/tick this gives ~333 RPM equivalent.
const CRANK_ADVANCE_DEG: f32 = 5.0;

/// ECU compile-time configuration
const ECU_SETTINGS: ECUSettings = ECUSettings {
    signal_blink_period: 1000,
};

#[entry]
fn main() -> ! {
    rtt_init_print!();

    let dp = stm32f7xx_hal::pac::Peripherals::take().unwrap();
    let cp = cortex_m::Peripherals::take().unwrap();

    let mut ecu_hw = ECUHardware::init(dp, cp);
    let mut ecu_state = ECUState::new();
    let mut time_ms: u64 = 0;

    loop {
        ecu_update(
            || time_ms,
            &ecu_hw.crank,
            &mut ecu_hw.cylinders,
            &mut ecu_hw.throttle,
            &mut ecu_hw.l_turn,
            &mut ecu_hw.r_turn,
            &mut ecu_hw.headlights_out,
            &mut ecu_hw.l_switch,
            &mut ecu_hw.r_switch,
            &mut ecu_hw.h_switch,
            &mut ecu_hw.headlight_switch,
            &mut ecu_hw.accel_pedal,
            &mut ecu_state,
            &ECU_SETTINGS,
        );

        ecu_hw.crank.increment(CRANK_ADVANCE_DEG);

        // TODO: Change this to use a general-purpose timer to keep time (interrupt increments a time variable)
        // Specifically, look into the SysTick timer to see if we can utilize that here
        ecu_hw.delay_ms(LOOP_PERIOD_MS);
        time_ms += LOOP_PERIOD_MS as u64;
    }
}
