# Rust-ECU TODO

This file lays out the tasks needed to complete to consider this project
"minimum viable". This file omits a checklist because tasks like that should
be tracked using GitHub issues at this repositories' homepage located at:

[GitHub - Litemage/Rusty-ECU](https://github.com/Litemage/rusty-ecu)

Note, as in the README: the term "ECU" is synonymous in this context to the
STM32-F767 development board that we are imagining is a cool, enclosed, and
totally professionally designed automotive ECU.

## Requirements

The following requirements must be satisfied to consider the project "Minimum 
Viable"

**ECU Input**

The ECU shall take input from 2 potentiometers, and 2 DIP switches, and 3
buttons at minimum.

- 1 potentiometer for throttle
- 1 potentiometer for brake
- 1 DIP switch each for: headlights and hazzards
- 1 button each for left and right turn signals, and horn

**ECU Output**

The ECU shall provide the following output visualizations

- 2 LEDs for headlights
- 2 LEDs for blinkers
- 1 buzzer for horn
- 4 LEDs for engine timing visualization

Headlights/Blinkers are self-explanatory

**Programming Language**

The software for this project should be written as close to entirely in Rust
as possible.

**Testability**

This project should be fully testable without hardware through SIL testing. This
can be as detailed or abstract as we want.

**SIL/HIL Testing/Simulator**

We should have both SIL and HIL testing in this project, because that is
important for the viability of a product.

**Car-Shaped Mounting Plate**

We should make a car-shaped plate to mount all of these LEDs and stuff too,
in order to make it look cool. This can be either a top or front view 
silhouette of a car.

We should also make a simple "engine block" for the engine LEDs to mount to.
(this could literally just a mount that holds LEDs at a slightly engine-shaped
angle.

Finally, an enclosure for our "ECU".

## Extras

- Real motors (static)??
- LED for starter/fuel pump and make a fake startup sequence?
- Shifting?
- Connectors that aren't simple Dupont wires? Maybe everything plugs into the
  enclosure?
