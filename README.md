# HDMI HPD Control for RP2040

A Rust/Embassy firmware for controlling HDMI Hot Plug Detection (HPD) signals using an RP2040 microcontroller.

## Overview

This firmware allows you to:
- **Assert/De-assert HPD** - Control whether the HDMI source sees a connected sink
- **Pulse HPD** - Trigger EDID re-read without full disconnect
- **Full reconnect cycle** - Force complete re-negotiation

## Hardware Connections

Based on the schematic, the following GPIO pins are used:

| Signal     | GPIO | Description                              |
|------------|------|------------------------------------------|
| HPD_CNTRL  | 20   | Controls HPD state via MOSFET circuit    |
| GEN_BTN    | 11   | General purpose button (active low)      |
| GPIO_LED   | 19   | Status LED indicator                     |

### Circuit Notes

The HPD control circuit uses an AO3400A N-channel MOSFET (Q1):
- R48 (220Ω) connects HDMI_HPD to the MOSFET gate
- R1 (100Ω) is in series with the MOSFET drain
- R2 (10kΩ) provides pull-down on HPD_CNTRL

When GPIO20 drives HPD_CNTRL HIGH, the HDMI source will see the HPD signal indicating a sink is present.

## Prerequisites

1. **Rust toolchain**:
   ```bash
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   ```

2. **RP2040 target**:
   ```bash
   rustup target add thumbv6m-none-eabi
   ```

3. **probe-rs** (for flashing):
   ```bash
   cargo install probe-rs-tools
   ```

   Or use **elf2uf2-rs** for UF2 flashing:
   ```bash
   cargo install elf2uf2-rs
   ```

## Building

```bash
# Debug build
cargo build

# Release build (smaller, optimized)
cargo build --release
```

## Flashing

### Option 1: Using probe-rs (with SWD debugger)

```bash
cargo run --release
```

### Option 2: Using UF2 (BOOTSEL button)

1. Hold BOOTSEL button and plug in RP2040
2. Convert and copy:
   ```bash
   elf2uf2-rs target/thumbv6m-none-eabi/release/hdmi-hpd-control
   # Copy the .uf2 file to the mounted RP2040 drive
   ```

## Usage

### Button Controls

| Action       | Effect                                           |
|--------------|--------------------------------------------------|
| Short press  | Toggle HPD state (connected ↔ disconnected)      |
| Long press   | Pulse HPD (~200ms) to trigger EDID re-read       |

### LED Indicator

The LED on GPIO19 indicates system status:
- **Blinking**: System running normally
- (Can be customized to show HPD state)

## Debugging

Debug output is available via RTT (Real-Time Transfer). Use:

```bash
# With probe-rs
probe-rs run --chip RP2040

# Or attach to running target
probe-rs attach --chip RP2040
```

## Project Structure

```
hdmi-hpd-control/
├── Cargo.toml          # Dependencies and build config
├── build.rs            # Build script for linker
├── memory.x            # Memory layout for RP2040
├── .cargo/
│   └── config.toml     # Target and runner config
└── src/
    ├── main.rs         # Main application
    └── hpd.rs          # HPD control module
```

## Extending the Firmware

### Adding USB Serial Control

You can add USB serial for PC control by enabling the USB feature:

```rust
// In Cargo.toml, uncomment:
// embassy-usb = { version = "0.4", features = ["defmt"] }
```

Then implement a USB CDC ACM class to receive commands like:
- `A` - Assert HPD
- `D` - De-assert HPD
- `P` - Pulse HPD
- `R` - Full reconnect

### Adding I2C EDID Passthrough

To implement EDID manipulation, you could use the I2C peripheral:
- SCL is typically pin 15 on HDMI connectors
- SDA is typically pin 16 on HDMI connectors

## License

MIT License
