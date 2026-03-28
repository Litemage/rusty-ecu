# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Commands

```bash
# Build everything (host targets only)
cargo build

# Build the STM32 firmware (requires ARM toolchain)
cargo build -p ecu-stm32

# Run the SIL simulator (interactive terminal UI)
cargo run -p ecu-sim

# Run all tests (ecu-core only; tests run on host)
cargo test -p ecu-core

# Run a single test
cargo test -p ecu-core -- <test_name>
```

The `ecu-stm32` crate targets `thumbv7em-none-eabihf` (set in `ecu-stm32/.cargo/config.toml`) and requires the ARM cross-compilation toolchain. The other crates build for the host.

## Architecture

This is a Cargo workspace with three crates:

### `ecu-core` — platform-agnostic logic (`no_std`)
The heart of the project. Contains all ECU logic as pure Rust with no hardware dependencies. Uses `#![cfg_attr(not(test), no_std)]` so it compiles for both embedded targets and host (for tests/sim).

All hardware interaction is abstracted behind traits:
- `engine::CrankPositionSensor` — reads crank angle in degrees
- `engine::CylinderOutputs` — sets spark plug states for 4 cylinders
- `lighting::LightController` — get/set a single light on/off
- `input::SwitchInput` — reads a boolean switch state

The primary entry point is `ecu_update(...)` in `lib.rs`, called periodically by the platform layer. It orchestrates engine firing (`engine_update`), turn signal/hazard/headlight state (`ecu_update_state`, `ecu_update_turn_signals`), and blink timing (`lighting::signal_for_time`).

Engine firing order is 1→4→3→2 (fire angles: cyl1=0°, cyl4=90°, cyl3=180°, cyl2=270°), with a 45° window per cylinder.

### `ecu-sim` — SIL simulator (host, `std`)
Implements all `ecu-core` traits as virtual structs (`VirtualCrank`, `VirtualIgnition`, `VirtualLight`, `VirtualSwitch`). Runs an interactive terminal loop using `crossterm` with keyboard controls: `l`/`r`/`h`/`o` toggle left signal/right signal/hazards/headlights, `q` quits. The engine crank advances by 10°/tick at 10ms intervals to simulate rotation.

### `ecu-stm32` — STM32F767 firmware (`no_std`, cross-compiled)
Minimal stub that sets up the embedded runtime. Currently just loops. Hardware implementations of `ecu-core` traits go here. Targets `thumbv7em-none-eabihf`.

## Key design pattern

New hardware features follow this flow:
1. Define a trait in `ecu-core` (no hardware coupling)
2. Implement the trait in `ecu-sim` with a virtual struct
3. Implement the trait in `ecu-stm32` using the STM32 HAL
4. Wire it into `ecu_update(...)` in `ecu-core/src/lib.rs`

This keeps all logic testable without hardware via `cargo test -p ecu-core` and simulatable via `cargo run -p ecu-sim`.
