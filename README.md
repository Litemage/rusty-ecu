# Rusty ECU

This is an exploratory, educational project designed to evaluate (for myself) the
viability of Rust as an embedded language, specifically targeting the STM32
family of microcontrollers, which is well supported by the current ecosystem
of embedded rust projects.

For this purpose, we are choosing an imaginary automotive ECU. This "ECU" will
just be a STM32-F767 Nucleo development board. When this README refers to an
"*ECU*", it is referring to the development board. 

You can find requirements in [TODO.md](./TODO.md). This file outlines the project's
minimum functionality. As a brief overview, this "ECU" will take inputs from
buttons, switches, and potentiometers on a breadboard and output it's state as
LEDs mounted cleverly to (a) 3D-printed mechanical system(s) which will help
visualize the turn signals, headlights, engine, etc.

## Resources

- [Embedded Rust Book](https://docs.rust-embedded.org/book/intro/index.html)
- [Embedonomicon](https://docs.rust-embedded.org/embedonomicon/)
- [Cargo Book](https://doc.rust-lang.org/cargo/index.html)

### Interesting Crates Being Used

- [cortex-m-rt Crate](https://docs.rs/cortex-m-rt/latest/cortex_m_rt/)
- [cortex-m](https://docs.rs/cortex-m/latest/cortex_m/)
- [panic-halt](https://docs.rs/panic-halt/latest/panic_halt/)
- [stm32f7xx_hal](https://docs.rs/stm32f7xx-hal/latest/stm32f7xx_hal/)

## Building

*This project was developed using RustRover by JetBrains, but can be compiled outside of it no problem*

I can't quite figure out how to get RustRover to build the project correctly from the root (yet) so for now, to build
firmware, you will have to run `cargo build --release` from the [ecu-stm32](./ecu-stm32) directory.

## Useful Commands

*Note this is targeted at us baby Rustaceans :)*

Build and open this project's documentation:
```shell
cargo doc --open
```

## Parts

The major parts used in this project are:

- [STM32 F767 Nucleo Development Board](https://www.digikey.com/en/products/detail/stmicroelectronics/NUCLEO-F767ZI/6004740)
