//! Mock ECU hardware interface - owns controller and CPU peripherals. All hardware interraction
//! is done through ECUHardware.
use ecu_core::engine::{CrankPositionSensor, CylinderOutputs, Throttle};
use ecu_core::input::{PedalInput, SwitchInput};
use ecu_core::lighting::LightController;
use stm32f7xx_hal::gpio::{Analog, Input, Output, Pin, PinState, PullDown, PushPull};
use stm32f7xx_hal::pac::{ADC1, Peripherals};
use stm32f7xx_hal::prelude::*;
use stm32f7xx_hal::timer::SysDelay;

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
pub struct Cylinders {
    cyl_1: Pin<'E', 15, Output<PushPull>>,
    cyl_2: Pin<'E', 14, Output<PushPull>>,
    cyl_3: Pin<'E', 12, Output<PushPull>>,
    cyl_4: Pin<'E', 10, Output<PushPull>>,
}

impl CylinderOutputs for Cylinders {
    fn set_all(&mut self, _states: [bool; 4]) {
        // Note with a combination of gpio::Pin::erase() from stm32f7xx_hal and an array in "Cylinders"
        // to do this much  cleaner but... for 4 pins, it seems unnecessary for now.
        self.cyl_1.set_state(PinState::from(_states[0]));
        self.cyl_2.set_state(PinState::from(_states[1]));
        self.cyl_3.set_state(PinState::from(_states[2]));
        self.cyl_4.set_state(PinState::from(_states[3]));
    }
}

/// Stub for future implementation of throttle TODO: Implement this.
pub struct HwThrottle {
    val: u8,
}

impl Throttle for HwThrottle {
    fn set_throttle(&mut self, value: u8) {
        self.val = value;
    }
}

impl HwThrottle {
    /// Returns the positon of the throttle as a proportion [0,255]
    pub fn get_throttle_pos(&self) -> u8 {
        self.val
    }
}

// endregion

// region i/o

/// A single push-pull GPIO output
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
pub struct StubInputPedal {
    adc: ADC1, // TODO: we'll need a custom solution for picking which ADC we use.... damnit
}

impl PedalInput for StubInputPedal {
    fn read_pedal(&self) -> u8 {
        // Assuming ADC1 has been set up by now...

        // Write channel to sequence slot 1 (channel 10)
        self.adc.sqr3.modify(|_, w| w.sq1().variant(10));

        // clear EOC flag (prevents acting on a stale request in a moment)
        self.adc.sr.modify(|_, w| w.eoc().clear_bit());

        // Start conversion by setting SWSTART
        self.adc.cr2.modify(|_, w| w.swstart().set_bit());

        while self.adc.sr.read().eoc().bit_is_clear() {
            // Waiting for the end of the conversion
        }

        // We got a conversion
        let adc_code: u16 = (self.adc.dr.read().bits() & 0x0FFF) as u16;

        // Figure out the proportion of "on" we are
        (255.0 * (adc_code as f32 / 4096.0)) as u8
    }
}

// endregion

// region ecu-hardware

/// Owns all peripherals and exposes all hardware interraction the ECU uses
pub struct ECUHardware {
    pub timer: SysDelay,

    // Engine
    pub crank: SimCrank,
    pub cylinders: Cylinders,
    pub throttle: HwThrottle,

    // Outputs
    pub l_turn: HwOutputLight<'E', 11>,
    pub r_turn: HwOutputLight<'F', 13>,
    pub headlights_out: HwOutputLight<'E', 9>,

    // Inputs
    pub l_switch: HwInputSwitch<'B', 8>,
    pub r_switch: HwInputSwitch<'B', 9>,
    pub h_switch: HwInputSwitch<'A', 5>,
    pub headlight_switch: HwInputSwitch<'A', 6>,
    pub accel_pedal: StubInputPedal,
}

impl ECUHardware {
    /// Initializes a new ECUHardware structure. Takes ownership of all peripherals
    pub fn init(dp: Peripherals, cp: cortex_m::Peripherals) -> Self {
        // region adc-notes

        // ===== NOTES ON ADC ===== // Before touching the ADC:
        // - We need to use the RCC to enable the ADC1 digital clock (Claude caught this)
        // - Delay ~16 CPU cycles

        // TO POWER ON THE ADC:
        // - Set the ADON bit in the ADC_CR2 register.
        // - Wait for about 600 CPU (@216MHHz) cycles before using ADC (t_STAB = 2us typ, 3us max)
        // - Conversion starts when either the SWSTART or the JSWSTART bit is set
        // - Clear ADON to stop conversion and power down ADC
        // (datasheet: 15.3.1)

        // ADC CLOCKS
        // ADC Max clock: 36MHz
        // APB2 freq: (Max: 108MHz
        // - ADC requires "ADCCLK" sources from APB2 clock, divided by a prescaler.
        //   I think we'll have to set up this clock (correct, you need to go into the APB2EN register
        //   and enable the ADC1 clock)
        // - Digital interface is equal to APB2, and can be enabled/disabled through the RCC_APB2ENR reg
        // (datahsset 15.3.3)

        // GROUPS
        // - Conversions can happen in any order by configuring the ADC_SQRx registers for regular channels

        // SINGLE CONVERSTION MODE
        // - CONT bit must be 0
        // - Started by any of: SWSTART in ADC_CR2 for reg channels. JSWSTART for injected, or external trigger for injected
        //   I imagine we will use regular channels
        // (datasheet 15.3.5)

        // CONVERSION TIMING
        // - After 15 clock cycles, data is ready: Reading from the ADC_DR regiister auto clears
        //   the EOC bit.
        // (datasheet 15.3.7)

        // ALIGNMENT
        // - ALIGN bit in ADC_CR2 selects left or right-aligned data in ADC data registers
        // (datasheet 15.4)

        // CONVERSION
        // - RES bits are used to select ADC bits
        // (datasheet 15.7)

        // (15.3 is ADC registers description)
        // ===== END ======

        // DEFAULTS NOTES
        // REG: ADC_CR1
        // - ADC is by default, at 12 bits
        // - AWD (Analog Watch Dog) is disabled by default, on both reg and inj
        // - DISCON mode defaults OK (DISABLED)
        // - SCAN mode disabled
        // - Interrupts OFF
        // REG: ADC_CR2
        // - CONT mode off
        // - DMA disabled
        // - ALIGN ok (right-aligned)
        // - (datasheet 15.13.11) ADC_SQR3 -> Set SQ1[4:0] to channel number of current ADC
        //   note that length of sequence is left at "1" by default which is what we want for now
        // - (datashheet 15.13.16) ADC_CCR. Will need to set the proper prescaler for ADC. See
        //   datasheet for maximum ADC clock frequency, and adjust accordingly.
        //
        // - (datasheet 15.13.14) ADC_DR regular data register which contains converted data.

        // ADC READ NOTES
        // - Make sure to clear EOC *before* setting SWSTART to avoid stale flag
        // - Do NOT need seperate clear after reading ADC_DR
        // (datasheet 15.3.7, 15.13.1)

        // endregion

        // Initialize the ADC...
        // NOTE: We are on pin PC0 for throttle which is: ADC1_IN10 (channel 10)

        // Enable the clock controlling the digital ADC interfaces - note the clock will be
        // enabled for a brief moment before the real clock is configured by the HAL
        dp.RCC.apb2enr.modify(|_, w| {
            // (datasheet 5.3.14, pg 192)
            // BIT 8: ADC1 EN
            w.adc1en().enabled()
        });

        // Set up the clocks. Note that these functions do A LOT of behind-the-scenes work with
        // configuring the clocks and adjusting prescalers.
        let rcc = dp.RCC.constrain();
        let clocks = rcc.cfgr.sysclk(216.MHz()).freeze();

        // Delay for 16 clock cycles to allow clock to stabilize (source???)
        cortex_m::asm::delay(16);

        // Set up the ADC prescaler
        // We know for our situation APB2 is running at 108MHz, and F_MAX for ADC is 36MHz
        // so, the scaler has to be at least 4
        dp.ADC_COMMON.ccr.modify(|_, w| {
            // BIT[17:16] - ADCPRE - set to 0b01, which is /4 scaler
            w.adcpre().bits(0b01)
        });

        // NOTE: ADC1_CR1 needs no modifications, we are using reset defaults
        // NOTE: ADC_CR2 needs no modification either, using reset defaults
        // NOTE: ADC_SQR1 needs no modification, but sets the length of ADC sequence (we use only 1)
        // NOTE: ADC_SMPR1 defaults are OK for sampling time on potentiometers, but may need to change

        // Enable the ADC
        dp.ADC1.cr2.modify(|_, w| {
            // ADC_CR2 Bit[0] - ADCON, turns on the ADC
            w.adon().enabled()
        });

        // Wait to allow the ADC to stabilize (~500 cpu cycles) or 2-3 us
        cortex_m::asm::delay(500);

        // Get various GPIO ports
        let gpioa = dp.GPIOA.split();
        let gpiob = dp.GPIOB.split();
        let gpioc = dp.GPIOC.split();
        let gpioe = dp.GPIOE.split();
        let gpiof = dp.GPIOF.split();

        // Set accel pedal pin as analog
        gpioc.pc0.into_analog();

        // Set up the delay
        let delay = cp.SYST.delay(&clocks);

        ECUHardware {
            timer: delay,

            crank: SimCrank { angle_deg: 0.0 },
            cylinders: Cylinders {
                cyl_1: gpioe.pe15.into_push_pull_output(),
                cyl_2: gpioe.pe14.into_push_pull_output(),
                cyl_3: gpioe.pe12.into_push_pull_output(),
                cyl_4: gpioe.pe10.into_push_pull_output(),
            },
            throttle: HwThrottle { val: 0 },

            l_turn: HwOutputLight(gpioe.pe11.into_push_pull_output()),
            r_turn: HwOutputLight(gpiof.pf13.into_push_pull_output()),
            headlights_out: HwOutputLight(gpioe.pe9.into_push_pull_output()),

            l_switch: HwInputSwitch(gpiob.pb8.into_pull_down_input()),
            r_switch: HwInputSwitch(gpiob.pb9.into_pull_down_input()),
            h_switch: HwInputSwitch(gpioa.pa5.into_pull_down_input()),
            headlight_switch: HwInputSwitch(gpioa.pa6.into_pull_down_input()),
            accel_pedal: StubInputPedal { adc: dp.ADC1 },
        }
    }

    /// Use the Cortex-M SysTick delay
    pub fn delay_ms(&mut self, ms: u32) {
        self.timer.delay_ms(ms);
    }
}

// endregion
