//! This module holds mathematical models useful for simulating engine movement in the time domain
use core::f32::consts::PI;

// These values were pulled from the python simulation
const J: f32 = 1.4;
const B: f32 = 1.0;
const IDLE_T: f32 = 5.0;
const MAX_T: f32 = 20.0;

#[derive(Default)]
pub struct EngineModel {
    /// Angular velocity of the shaft, in radians per second
    last_omega: f32,
    /// Angle of the crankshaft, in degrees, [0,359]
    last_angle: f32,
    /// Moment of inertia
    j: f32,
    /// Viscous damping
    b: f32,
    /// Idle torque, which is assumed to be applied whenever the engine is on
    idle_t: f32,
    /// Max torque that can be applied with throttle
    max_t: f32,
}

impl EngineModel {
    pub fn new() -> Self {
        EngineModel {
            j: J,
            b: B,
            idle_t: IDLE_T,
            max_t: MAX_T,
            ..Default::default()
        }
    }

    pub fn get_last_shaft_rpm(&self) -> f32 {
        self.last_omega * (60.0 / (2.0 * PI))
    }

    pub fn engine_step(&mut self, throttle: f32, dt: f32) -> (f32, f32) {
        let t_engine = self.idle_t + throttle * self.max_t;
        let t_friction = self.b * self.last_omega;
        let alpha = (t_engine - t_friction) / self.j;

        let omega = (self.last_omega + alpha * dt).max(0.0);
        let delta_deg = omega * dt * (180.0 / PI);
        let angle = (self.last_angle + delta_deg) % 360.0;

        self.last_omega = omega;
        self.last_angle = angle;

        (omega, angle)
    }
}
