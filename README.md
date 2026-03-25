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

## Parts

The major parts used in this project are:

- [STM32 F767 Nucleo Development Board](https://www.digikey.com/en/products/detail/stmicroelectronics/NUCLEO-F767ZI/6004740)
