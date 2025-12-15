# USB Programming and Logging Guide for RP2040

This guide explains how to program your RP2040-based HDMI HPD controller via USB (using UF2 files) and receive logs over USB instead of using SWD/debug probes.

## Prerequisites

Install the required tools:

```bash
# Install elf2uf2-rs for converting ELF to UF2 format
cargo install elf2uf2-rs

# Install picotool (optional, for debugging)
# macOS:
brew install picotool

# Linux:
# Follow instructions at https://github.com/raspberrypi/picotool
```

## Step 1: Update Project Configuration

### 1.1 Update Cargo.toml

Add USB logging dependencies to `Cargo.toml`:

```toml
[dependencies]
# ... existing dependencies ...

# Add these for USB logging
embassy-usb = { version = "0.4", features = ["defmt"] }
embassy-usb-logger = "0.4"
```

### 1.2 Update .cargo/config.toml

Change the runner to use `elf2uf2-rs`:

```toml
[build]
target = "thumbv6m-none-eabi"

[target.thumbv6m-none-eabi]
# Use elf2uf2-rs for USB programming
runner = "elf2uf2-rs -d"
rustflags = [
    "-C", "link-arg=--nmagic",
    "-C", "link-arg=-Tlink.x",
    "-C", "link-arg=-Tdefmt.x",
    "-C", "inline-threshold=5",
    "-C", "no-vectorize-loops",
]

[env]
DEFMT_LOG = "info"
```

## Step 2: Programming via USB (UF2 Method)

### Method A: Using `cargo make` (Recommended)

1. **Build the UF2 file:**
   ```bash
   # Release build (optimized)
   cargo make uf2 --release

   # Debug build (faster compilation, larger binary)
   cargo make uf2
   ```

   This will create `target/hdmi-hpd-control.uf2`

2. **Enter bootloader mode:**
   - Hold down the BOOTSEL button on your RP2040 board
   - While holding BOOTSEL, connect the USB cable to your computer
   - Release BOOTSEL
   - The board should appear as a USB mass storage device named "RPI-RP2"

3. **Copy the UF2 file:**
   ```bash
   # macOS
   cp target/hdmi-hpd-control.uf2 /Volumes/RPI-RP2/

   # Linux
   cp target/hdmi-hpd-control.uf2 /media/$USER/RPI-RP2/

   # Or just drag and drop the file to the RPI-RP2 drive
   ```

4. The device will automatically reset and run your firmware

### Method B: Using `cargo make uf2-flash` (Auto-copy)

This attempts to automatically copy the UF2 file to the RP2040:

```bash
cargo make uf2-flash --release
```

Note: The RP2040 must already be in bootloader mode.

### Method C: Using `cargo run` (Automatic)

1. **Enter bootloader mode** (same as above)

2. **Build and upload:**
   ```bash
   cargo run --release
   ```

   The `elf2uf2-rs -d` runner will automatically:
   - Build the firmware
   - Convert ELF to UF2 format
   - Copy the UF2 file to the RP2040
   - Reset the device

### Method D: Manual UF2 Upload

1. **Build the UF2 file manually:**
   ```bash
   cargo build --release
   elf2uf2-rs target/thumbv6m-none-eabi/release/hdmi-hpd-control target/hdmi-hpd-control.uf2
   ```

2. **Enter bootloader mode** and copy the file (same as Method A)

## Step 3: Viewing Logs via USB

### Option 1: Using defmt-rtt over USB (Current Setup)

The current setup uses `defmt-rtt` which requires a debug probe. To get logs over USB, you need to modify the code.

### Option 2: Switch to USB Logging (Recommended)

Modify your `src/main.rs` to use USB logging instead of RTT:

```rust
// Replace this line:
use {defmt_rtt as _, panic_probe as _};

// With this:
use embassy_usb_logger as _;
use panic_reset as _;
```

Update dependencies in `Cargo.toml`:

```toml
# Replace these:
# defmt-rtt = "0.4"
# panic-probe = { version = "0.3", features = ["print-defmt"] }

# With these:
embassy-usb-logger = "0.4"
panic-reset = "0.1"
```

Then rebuild and upload. Logs will appear on your computer's serial port.

### Viewing USB Serial Logs

Once USB logging is enabled:

**macOS/Linux:**
```bash
# Find the serial port
ls /dev/tty.usbmodem*

# Connect with screen
screen /dev/tty.usbmodem14201 115200

# Or use minicom
minicom -D /dev/tty.usbmodem14201 -b 115200
```

**Windows:**
- Use PuTTY, TeraTerm, or the Arduino Serial Monitor
- Look for COM ports in Device Manager
- Connect at 115200 baud

**Using picotool:**
```bash
# Get device info
picotool info

# Reboot to bootloader
picotool reboot -f -u
```

## Troubleshooting

### Device Not Appearing in BOOTSEL Mode

- Make sure you're holding BOOTSEL before connecting USB
- Try a different USB cable (some are charge-only)
- Try a different USB port
- On Linux, check `dmesg` for USB events

### Build Errors

```bash
# Clean build
cargo clean

# Update dependencies
cargo update

# Check target is installed
rustup target add thumbv6m-none-eabi
```

### No Serial Port After Upload

- USB logging requires code changes (see Option 2 above)
- Check that `embassy-usb-logger` is properly initialized
- Some RP2040 boards need USB enumeration time (wait 2-3 seconds)

### Permission Denied (Linux)

```bash
# Add user to dialout group
sudo usermod -a -G dialout $USER

# Log out and back in, or:
newgrp dialout
```

## Quick Reference

```bash
# Build UF2 file (recommended method)
cargo make uf2 --release

# Build and auto-flash (if RP2040 already in bootloader mode)
cargo make uf2-flash --release

# Build only
cargo build --release

# Build and upload via elf2uf2-rs runner (bootloader mode required)
cargo run --release

# Create UF2 manually
elf2uf2-rs target/thumbv6m-none-eabi/release/hdmi-hpd-control target/firmware.uf2

# View logs (after enabling USB logging)
screen /dev/tty.usbmodem* 115200

# Other useful commands
cargo make check      # Check for errors
cargo make clippy     # Run lints
cargo make size       # Show binary size
cargo make clean      # Clean build artifacts
```

## Additional Resources

- [RP2040 Datasheet](https://datasheets.raspberrypi.com/rp2040/rp2040-datasheet.pdf)
- [elf2uf2-rs Documentation](https://github.com/JoNil/elf2uf2-rs)
- [Embassy USB Examples](https://github.com/embassy-rs/embassy/tree/main/examples/rp/src/bin)
- [picotool GitHub](https://github.com/raspberrypi/picotool)
