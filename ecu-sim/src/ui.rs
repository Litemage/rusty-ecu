use crate::stub::{
    VirtualCrank, VirtualIgnition, VirtualLight, VirtualPedal, VirtualSwitch, VirtualThrottle,
};
use ecu_core::engine::CrankPositionSensor;
use ecu_core::input::PedalInput;
use ecu_core::{ECUSettings, ECUState, ecu_update};
use eframe::Frame;
use egui::{Context, Ui};
use std::ops::Sub;
use std::sync::OnceLock;
use std::time::{Duration, Instant};

// region private-vars

/// The period of the ECU logic
const ECU_LOOP_PERIOD_MS: u32 = 10;
/// The increment of the crank position sensor at idle
const ENGINE_IDLE: f32 = 5.0;
/// The value to advance the virtual engine crankshaft by, as we simulate, multiplied by the accelerator
const ENGINE_ADVANCE_DEG: f32 = 20.0;

// endregion

// region misc

fn get_time_ms() -> u64 {
    static START: OnceLock<Instant> = OnceLock::new();
    START.get_or_init(Instant::now).elapsed().as_millis() as u64
}

// endregion

pub struct ECUSimApp {
    virtual_crank: VirtualCrank,
    virtual_ignition: VirtualIgnition,
    virtual_throttle: VirtualThrottle,
    l_turn: VirtualLight,
    r_turn: VirtualLight,
    headlights: VirtualLight,
    l_switch: VirtualSwitch,
    r_switch: VirtualSwitch,
    h_switch: VirtualSwitch,
    headlight_switch: VirtualSwitch,
    accel_pedal: VirtualPedal,
    brake_pedal: VirtualPedal,
    ecu_state: ECUState,
    ecu_settings: ECUSettings,
    last_run: Instant,
}

impl ECUSimApp {
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        ECUSimApp {
            virtual_crank: VirtualCrank::new(),
            virtual_ignition: VirtualIgnition::new(),
            virtual_throttle: VirtualThrottle::new(),
            l_turn: VirtualLight { on: false },
            r_turn: VirtualLight { on: false },
            headlights: VirtualLight { on: false },
            l_switch: VirtualSwitch { on: false },
            r_switch: VirtualSwitch { on: false },
            h_switch: VirtualSwitch { on: false },
            headlight_switch: VirtualSwitch { on: false },
            accel_pedal: VirtualPedal { val: 0 },
            brake_pedal: VirtualPedal { val: 0 },
            ecu_state: ECUState::new(),
            ecu_settings: ECUSettings {
                signal_blink_period: 1000,
            },
            last_run: Instant::now().sub(Duration::from_hours(10)),
        }
    }

    /// Run function periodically, on a loop
    pub fn run_app(&mut self) {
        let now = Instant::now();
        if (now - self.last_run).as_millis() >= ECU_LOOP_PERIOD_MS as u128 {
            self.last_run = now;

            ecu_update(
                &get_time_ms,
                &self.virtual_crank,
                &mut self.virtual_ignition,
                &mut self.virtual_throttle,
                &mut self.l_turn,
                &mut self.r_turn,
                &mut self.headlights,
                &mut self.l_switch,
                &mut self.r_switch,
                &mut self.h_switch,
                &mut self.headlight_switch,
                &mut self.accel_pedal,
                &mut self.ecu_state,
                &self.ecu_settings,
            );

            // Advance the engine and pretend it's running
            self.virtual_crank.increment(
                ENGINE_IDLE
                    + (ENGINE_ADVANCE_DEG * (self.virtual_throttle.read_throttle() as f32 / 255.0)),
            );
        }
    }
}

impl eframe::App for ECUSimApp {
    fn logic(&mut self, ctx: &Context, _frame: &mut Frame) {
        self.run_app();
        ctx.request_repaint();
    }

    fn ui(&mut self, ui: &mut Ui, _frame: &mut Frame) {
        egui::CentralPanel::default().show_inside(ui, |ui| {
            ui.heading("Front Lights");
            let (light_response, light_painter) =
                ui.allocate_painter(egui::Vec2::new(150.0, 50.0), egui::Sense::hover());
            ui.heading("Engine Ignition");
            let (engine_response, engine_painter) =
                ui.allocate_painter(egui::Vec2::new(150.0, 100.0), egui::Sense::hover());

            let light_origin = light_response.rect.min; // Top-left corner of the paint region
            let engine_origin = engine_response.rect.min;
            let lights = [
                self.l_turn.on,
                self.headlights.on,
                self.headlights.on,
                self.r_turn.on,
            ];

            draw_car_lights(&light_painter, &light_origin, 10.0, lights);
            draw_engine_lights(
                &engine_painter,
                &engine_origin,
                10.0,
                self.virtual_ignition.states,
            );

            ui.heading("Raw Engine Values");
            raw_value_widget(
                ui,
                self.virtual_throttle.read_throttle(),
                "Throttle Value (u8 %)",
            );
            raw_value_widget(
                ui,
                self.accel_pedal.read_pedal(),
                "Accelerator Pedal (u8 %)",
            );
            raw_value_widget(
                ui,
                self.virtual_crank.read_angle(),
                "Crank position sensor (degrees)",
            );
            raw_value_widget(ui, self.virtual_ignition.states, "Ignition states");

            ui.separator();
            ui.heading("ECU Inputs");
            ui.horizontal(|ui| {
                ui.toggle_value(&mut self.l_switch.on, "Left Signal");
                ui.toggle_value(&mut self.r_switch.on, "Right Signal");
                ui.toggle_value(&mut self.h_switch.on, "Hazard Signal");
                ui.toggle_value(&mut self.headlight_switch.on, "Headlights");
            });
            ui.add(egui::Slider::new(&mut self.accel_pedal.val, 0..=255).text("Accelerator"));
            ui.add(egui::Slider::new(&mut self.brake_pedal.val, 0..=255).text("Brake"));
        });
    }
}

fn raw_value_widget(ui: &mut Ui, val: impl std::fmt::Debug, label: &str) {
    ui.horizontal(|ui| {
        ui.label(label);
        ui.label(format!("{:?}", val));
    });
}

fn sig_color_from_active(active: bool) -> egui::Color32 {
    if active {
        egui::Color32::YELLOW
    } else {
        egui::Color32::GRAY
    }
}

fn h_color_from_active(active: bool) -> egui::Color32 {
    if active {
        egui::Color32::WHITE
    } else {
        egui::Color32::GRAY
    }
}

fn cyl_color_from_active(active: bool) -> egui::Color32 {
    if active {
        egui::Color32::LIGHT_RED
    } else {
        egui::Color32::GRAY
    }
}

/// Draw circles representing the car lights. The `lights` array contains all the values of lights:
/// <left signal, headlight1, headlight2, right signal>
fn draw_car_lights(painter: &egui::Painter, origin: &egui::Pos2, radius: f32, lights: [bool; 4]) {
    let diameter = 2.0 * radius;
    let l_sig = egui::Pos2::new(origin.x + (2.0 * diameter), origin.y + 30.0);
    let h_1 = egui::Pos2::new(origin.x + (3.5 * diameter), origin.y + 30.0);
    let h_2 = egui::Pos2::new(origin.x + (5.0 * diameter), origin.y + 30.0);
    let r_sig = egui::Pos2::new(origin.x + (6.5 * diameter), origin.y + 30.0);

    painter.circle_filled(l_sig, radius, sig_color_from_active(lights[0]));
    painter.circle_filled(h_1, radius, h_color_from_active(lights[1]));
    painter.circle_filled(h_2, radius, h_color_from_active(lights[2]));
    painter.circle_filled(r_sig, radius, sig_color_from_active(lights[3]));
}

fn draw_engine_lights(
    painter: &egui::Painter,
    origin: &egui::Pos2,
    radius: f32,
    lights: [bool; 4],
) {
    let diameter = 2.0 * radius;

    let cylindars = [
        egui::Pos2::new(origin.x + (2.0 * diameter), origin.y + 30.0),
        egui::Pos2::new(origin.x + (3.5 * diameter), origin.y + 30.0),
        egui::Pos2::new(
            origin.x + (2.0 * diameter),
            origin.y + (2.0 * diameter) + 30.0,
        ),
        egui::Pos2::new(
            origin.x + (3.5 * diameter),
            origin.y + (2.0 * diameter) + 30.0,
        ),
    ];

    for (i, c) in cylindars.iter().enumerate() {
        painter.circle_filled(*c, radius, cyl_color_from_active(lights[i]));
    }
}
