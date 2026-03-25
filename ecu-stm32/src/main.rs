#![no_std]  // Don't link standard lib
#![no_main] // Don't use standard entry point

use panic_halt as _; // Pulls in the panic handler
use cortex_m_rt::entry;

// This HAL crate must be pulled in to get interrupt vectors, which is required
// for compilation.
use stm32f7xx_hal as _;

#[entry]
// The syntax (-> !) indicates the function never returns (required by #entry)
fn main() -> ! {
    loop {}
}
