//! Control for all lighting on the vehicle

// region public-traits

/// Interface for lighting controls. Used for headlights, tail lights, turn signals, and any other
/// lighting features that must be controlled by the ECU
pub trait LightController {
    fn get_light(&self) -> bool;
    fn set_light(&mut self, on: bool);
}

// endregion

/// Calculates if a turn signal should be on, given a timestamp and period. Only should be used
/// during blink cycle.
pub fn signal_for_time(ts_ms: u64, per_ms: u32) -> bool {
    // Modulo calculates how far "into" the period we are, and if we're in the top half, the signal
    // should be inactive (active would be 180 deg out of phase in reference to start of period)
    // Note this logic would break for floating-point values
    !((ts_ms % per_ms as u64) > (per_ms as u64 >> 1u64))
}

/// Given a timestamp in milliseconds, blink period in milliseconds, and light controller: blink a
/// light with the given period.
pub fn update_signal(ts_ms: u64, per_ms: u32, signal: &mut impl LightController) {
    signal.set_light(signal_for_time(ts_ms, per_ms));
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_signal_high_first_half_low_second_half() {
        let per_ms = 1000;
        assert!(
            signal_for_time(0, per_ms),
            "should be high at start of period"
        );
        assert!(
            signal_for_time(499, per_ms),
            "should be high just before midpoint"
        );
        assert!(
            !signal_for_time(501, per_ms),
            "should be low just after midpoint"
        );
        assert!(
            !signal_for_time(999, per_ms),
            "should be low at end of period"
        );
    }

    #[test]
    fn test_signal_wraps_correctly_beyond_one_period() {
        let per_ms = 1000;
        // ts=2100 -> phase=100 (first half) -> high
        assert!(signal_for_time(2100, per_ms), "phase 100 should be high");
        // ts=1600 -> phase=600 (second half) -> low
        assert!(!signal_for_time(1600, per_ms), "phase 600 should be low");
    }
}
