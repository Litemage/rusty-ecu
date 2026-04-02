//! Engine monitoring and control logic

// region public-traits

use crate::input::PedalInput;

/// Defines a source for motor crank shaft position.
pub trait CrankPositionSensor {
    /// Reports the current position of the crank sensor, in positive, unbounded degrees.
    fn read_angle(&self) -> f32;
}

/// Defines cylinder outputs to spark
pub trait CylinderOutputs {
    /// Set the state of each cylinder's spark plug
    ///
    /// The index of the array corresponds to the cylinder number - 1. i.e. index 0 is cylinder 1
    fn set_all(&mut self, states: [bool; 4]);
}

pub trait Throttle {
    /// Sets the throttle to a value between closed and full open mapped to the range [0,255] with
    /// 0 being fully closed, and 255 being fully open.
    fn set_throttle(&mut self, value: u8);
}

// endregion

// region public-functions

/// Updates the cylinder states based off of a provided crank position sensor and applies the outputs.
/// `pedal_cmd` is the requested command from the user, in the range [0, 65,535]
pub fn engine_update(
    sensor: &impl CrankPositionSensor,
    outputs: &mut impl CylinderOutputs,
    throttle: &mut impl Throttle,
    accel_pedal: &impl PedalInput,
) {
    let degrees = sensor.read_angle();
    let states = cylinders_for_angle(degrees);
    outputs.set_all(states);

    // Process the pedal request - no filtering for now, just set the throttle to whatever
    // the pedal is.
    throttle.set_throttle(accel_pedal.read_pedal());
}

// endregion
// region private-functions

fn cylinders_for_angle(degrees: f32) -> [bool; 4] {
    //                           Cylinder 1
    //                           |    Cylinder 2
    //                           |    |      Cylinder 3
    //                           |    |      |      Cylinder 4
    //                           |    |      |      |
    let fire_angles: [f32; 4] = [0.0, 270.0, 180.0, 90.0];
    let window = 45.0; // degrees each cylinder stays on

    let mut states = [false; 4];
    for (i, &fire_at) in fire_angles.iter().enumerate() {
        let past = rem_euclid(degrees - fire_at, 360.0);
        states[i] = past < window;
    }

    states
}

fn rem_euclid(val: f32, rhs: f32) -> f32 {
    let r = val % rhs;
    if r < 0.0 { r + rhs } else { r }
}

// endregion

// region unit-tests

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_only_one_cylinder_fires_at_a_time() {
        for deg in 0..360 {
            let states = cylinders_for_angle(deg as f32);
            let active_count = states.iter().filter(|&&s| s).count();
            assert!(
                active_count <= 1,
                "Expected at most 1 active cylinder at {deg}°, but got {active_count}: {states:?}"
            );
        }
    }

    #[test]
    fn test_cylinders_fire_in_correct_order() {
        // fire_angles: cyl1=0, cyl4=90, cyl3=180, cyl2=270
        // Expected firing order: 1 → 4 → 3 → 2
        let expected_order = [
            (0, 0),   // cylinder 1 fires at 0
            (90, 3),  // cylinder 4 fires at 90
            (180, 2), // cylinder 3 fires at 180
            (270, 1), // cylinder 2 fires at 270
        ];

        for (angle, expected_index) in expected_order {
            let states = cylinders_for_angle(angle as f32);
            assert!(
                states[expected_index],
                "Expected cylinder {} to be active at {angle} deg, but states were {states:?}",
                expected_index + 1
            );
        }
    }
}

// endregion
