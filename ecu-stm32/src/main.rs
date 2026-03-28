#![no_std]  // Don't link standard lib
#![no_main] // Don't use standard entry point

use panic_halt as _; // Pulls in the panic handler - halts processor on panic
use cortex_m_rt::entry;
use stm32f7xx_hal::{pac, prelude::*, timer::Timer};

#[entry]
// The syntax (-> !) indicates the function never returns (required by #entry)
fn main() -> ! {
    // Device peripherals
    let dp = pac::Peripherals::take().unwrap();
    // Cortex-m peripherals
    let cp = cortex_m::Peripherals::take().unwrap();

    let rcc = dp.RCC.constrain();
    let clocks = rcc.cfgr.sysclk(216.MHz()).freeze();

    let timer = Timer::syst(cp.SYST, &clocks);
    let mut delay = timer.delay();

    let gpiob = dp.GPIOB.split();
    let mut led = gpiob.pb0.into_push_pull_output();

    loop {
        led.set_high();
        delay.delay_ms(500u32);
        led.set_low();
        delay.delay_ms(500u32);
    }
}
