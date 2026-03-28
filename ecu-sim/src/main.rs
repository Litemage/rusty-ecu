use std::sync::OnceLock;
use std::time::{Duration, Instant};
use std::io::{self, Write};
use ecu_core::{ecu_update, ECUSettings, ECUState};
use ecu_core::engine::*;
use ecu_core::lighting::LightController;
use crossterm::terminal::{enable_raw_mode, disable_raw_mode};
use crossterm::event;
use crossterm::event::{Event, KeyEventKind, KeyCode};
use ecu_core::input::SwitchInput;
// region private-vars

/// The main update loop delay
const LOOP_DELAY_MS: u64 = 10;
/// The value to advance the virtual engine crankshaft by, as we simulate
const ENGINE_ADVANCE_DEG: f32 = 10.0;

// endregion

// region virtual-ignition-controller

/// Mock cylinder which will hold the state of each cylinder
struct VirtualIgnition {
    /// State of each cylinder, held in an array
    states: [bool; 4]
}

// Implement the cylinder outputs
impl CylinderOutputs for VirtualIgnition {
    fn set_all(&mut self, states: [bool; 4]) {
        self.states = states;
    }
}

impl VirtualIgnition {
    fn new() -> VirtualIgnition { VirtualIgnition {states: [false; 4]}}
}

// endregion

// region virtual-crank-sensor

/// A virtual crank sensor
struct VirtualCrank {
    angle_deg: f32
}

// Implement the CrankPositionSensor trait
impl CrankPositionSensor for VirtualCrank {
    fn read_angle(&self) -> f32 {
        return self.angle_deg;
    }
}

impl VirtualCrank {
    fn new() -> VirtualCrank { VirtualCrank {angle_deg: 0.0}}

    /// Increment the virtual crank sensor by `deg` degrees
    fn increment(&mut self, deg: f32) {
        self.angle_deg += deg;
        // Bound to [0.0, 360.0] degrees
        while self.angle_deg > 360.0 {
            self.angle_deg -= 360.0;
        }
    }
}

// endregion

// region virtual-lights

struct VirtualLight {
    on: bool
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

struct VirtualSwitch {
    on: bool
}

impl SwitchInput for VirtualSwitch {
    fn read_switch(&self) -> bool {
        return self.on;
    }
}

// endregion

// region misc

fn get_time_ms() -> u64 {
    static START: OnceLock<Instant> = OnceLock::new();
    START.get_or_init(Instant::now).elapsed().as_millis() as u64
}

// endregion

fn main() {
    // ===== SIMULATION SETUP =====

    let mut virtual_crank = VirtualCrank::new();
    let mut virtual_ignition = VirtualIgnition::new();
    let mut l_turn = VirtualLight { on: false};
    let mut r_turn = VirtualLight { on: false};
    let mut headlights = VirtualLight { on: false};
    let mut l_switch = VirtualSwitch { on: false};
    let mut r_switch = VirtualSwitch { on: false};
    let mut h_switch = VirtualSwitch { on: false};
    let mut headlight_switch = VirtualSwitch { on: false};
    let mut ecu_state = ECUState::new();
    let ecu_settings = ECUSettings {signal_blink_period: 1000};

    println!("Simulator for rusty-ecu. Press 'q' to quit. Controls:");
    println!("'r' for right turn signal, 'l' for left, 'h' for hazards, 'o' for headlights");

    // Enables "raw" terminal mode - processes input silently
    enable_raw_mode().unwrap();

    let mut last_engine_update = Instant::now();

    // ===== SIMULATION LOOP =====

    loop {
        // ===== ENGINE SIMULATION (rate-limited) =====
        if last_engine_update.elapsed() >= Duration::from_millis(LOOP_DELAY_MS) {
            last_engine_update = Instant::now();

            ecu_update(
                &get_time_ms,
                &virtual_crank,
                &mut virtual_ignition,
                &mut l_turn,
                &mut r_turn,
                &mut headlights,
                &mut l_switch,
                &mut r_switch,
                &mut h_switch,
                &mut headlight_switch,
                &mut ecu_state,
                &ecu_settings
            );

            // Visualize to user
            print_engine_state(&virtual_ignition, &virtual_crank, &l_turn, &r_turn, &headlights);

            // Advance the engine and pretend it's running
            virtual_crank.increment(ENGINE_ADVANCE_DEG);
        }

        // ===== INPUT (runs every iteration) =====

        if event::poll(Duration::from_millis(0)).unwrap() {
            if let Event::Key(key) = event::read().unwrap() {
                if key.kind == KeyEventKind::Press {
                    match key.code {
                        // Simulate inputs with keys (act as toggleable switches)
                        KeyCode::Char('l') => { l_switch.on = ! l_switch.on; }
                        KeyCode::Char('r') => { r_switch.on = ! r_switch.on; }
                        KeyCode::Char('h') => { h_switch.on = ! h_switch.on; }
                        KeyCode::Char('o') => { headlight_switch.on = ! headlight_switch.on; }
                        KeyCode::Char('q') => break, // Break out of simulator
                        _ => {/* Do nothing for everything else */}
                    }
                }
            }
        }
    }

    // On exit, disable raw mode
    disable_raw_mode().unwrap();
    println!();
}

fn print_engine_state(
    virtual_ignition: &VirtualIgnition,
    virtual_crank: &VirtualCrank,
    l_turn: &VirtualLight,
    r_turn: &VirtualLight,
    headlights: &VirtualLight,
) {
    let states = virtual_ignition.states;

    print!("\r Time: {:0>8} Crank positon: [{:0>6}] deg -- Firing: [{}{}{}{}] -- L: [",
           get_time_ms(),
           virtual_crank.angle_deg,
           if states[0] {"1"} else {"-"},
           if states[1] {"2"} else {"-"},
           if states[2] {"3"} else {"-"},
           if states[3] {"4"} else {"-"},
    );
    if l_turn.on { print!("{}", "O"); } else { print!("."); }
    print!("] R: [");
    if r_turn.on { print!("{}", "O"); } else { print!("."); }
    print!("] -- H: [");
    if headlights.on { print!("{}", "O-O"); } else { print!(".-."); }
    print!("]");

    io::stdout().flush().unwrap();
}