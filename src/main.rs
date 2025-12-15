#![no_std]
#![no_main]

mod hpd;

use defmt::*;
use embassy_executor::Spawner;
use embassy_rp::gpio::{Input, Level, Output, Pull};
use embassy_sync::blocking_mutex::raw::ThreadModeRawMutex;
use embassy_sync::channel::Channel;
use embassy_time::{Duration, Timer};
use {defmt_rtt as _, panic_probe as _};

use hpd::{HpdCommand, HpdController};

/// GPIO pin assignments based on schematic
mod pins {
    
    /// HPD_CNTRL - Connected to GPIO20 (directly controls hot plug detection)
    pub const HPD_CNTRL: u8 = 19;
    
    /// GEN_BTN - Connected to GPIO11 (general purpose button)
    pub const GEN_BTN: u8 = 11;
    
    /// GPIO_LED - Connected to GPIO19 (LED indicator)
    pub const GPIO_LED: u8 = 18;
}

/// Channel for sending HPD commands between tasks
static HPD_CHANNEL: Channel<ThreadModeRawMutex, HpdCommand, 4> = Channel::new();

/// Main entry point
#[embassy_executor::main]
async fn main(spawner: Spawner) {
    info!("========================================");
    info!("   HDMI HPD Control - RP2040");
    info!("========================================");
    
    let p = embassy_rp::init(Default::default());
    
    // Configure HPD_CNTRL as output (GPIO20)
    // Based on your schematic:
    // - The MOSFET Q1 (AO3400A) inverts the logic
    // - When GPIO20 is HIGH -> HPD is asserted (sink connected)
    // - When GPIO20 is LOW -> HPD is de-asserted (sink disconnected)
    let hpd_pin = Output::new(p.PIN_20, Level::Low);
    let mut hpd = HpdController::new(hpd_pin);
    
    // Configure the general button as input with pull-up (GPIO11)
    // Button press = LOW (active low)
    let button = Input::new(p.PIN_11, Pull::Up);
    
    // Configure LED as output (GPIO19)
    let led = Output::new(p.PIN_19, Level::Low);
    
    info!("GPIO configured:");
    info!("  - HPD_CNTRL: GPIO{}", pins::HPD_CNTRL);
    info!("  - GEN_BTN:   GPIO{}", pins::GEN_BTN);
    info!("  - LED:       GPIO{}", pins::GPIO_LED);
    
    // Spawn button handler task
    spawner.spawn(button_handler(button)).unwrap();
    
    // Spawn LED indicator task
    spawner.spawn(led_indicator(led)).unwrap();
    
    // Initial delay before asserting HPD
    info!("Waiting for power stabilization...");
    Timer::after(Duration::from_millis(500)).await;
    
    // Assert HPD - normal operation (sink connected)
    hpd.assert();
    info!("System ready. Press button to toggle HPD.");
    info!("  - Short press: Toggle HPD state");
    info!("  - Long press:  HPD pulse (EDID re-read)");
    
    // Main loop - process HPD commands
    loop {
        let cmd = HPD_CHANNEL.receive().await;
        
        match cmd {
            HpdCommand::Assert => {
                hpd.assert();
            }
            HpdCommand::Deassert => {
                hpd.deassert();
            }
            HpdCommand::Toggle => {
                hpd.toggle();
            }
            HpdCommand::Pulse => {
                hpd.pulse().await;
            }
            HpdCommand::Reconnect => {
                hpd.reconnect_cycle().await;
            }
        }
        
        info!("Current HPD state: {:?}", hpd.state());
    }
}

/// Button handler task
/// 
/// Detects button presses and sends appropriate commands:
/// - Short press (< 500ms): Toggle HPD state
/// - Long press (>= 500ms): Trigger HPD pulse for EDID re-read
#[embassy_executor::task]
async fn button_handler(mut button: Input<'static>) {
    const LONG_PRESS_THRESHOLD: Duration = Duration::from_millis(500);
    const DEBOUNCE: Duration = Duration::from_millis(50);
    
    loop {
        // Wait for button press (falling edge - button is active low)
        button.wait_for_falling_edge().await;
        Timer::after(DEBOUNCE).await;
        
        // Make sure it's still pressed after debounce
        if button.is_high() {
            continue;
        }
        
        info!("Button pressed");
        
        // Measure how long button is held
        let press_start = embassy_time::Instant::now();
        
        // Wait for button release
        button.wait_for_rising_edge().await;
        Timer::after(DEBOUNCE).await;
        
        let press_duration = press_start.elapsed();
        
        if press_duration >= LONG_PRESS_THRESHOLD {
            // Long press - trigger HPD pulse
            info!("Long press detected - triggering HPD pulse");
            HPD_CHANNEL.send(HpdCommand::Pulse).await;
        } else {
            // Short press - toggle HPD
            info!("Short press detected - toggling HPD");
            HPD_CHANNEL.send(HpdCommand::Toggle).await;
        }
    }
}

/// LED indicator task
/// 
/// Shows HPD state via LED:
/// - Solid ON: HPD asserted (connected)
/// - Blinking: HPD de-asserted (disconnected)
/// - Fast blink: Processing command
#[embassy_executor::task]
async fn led_indicator(mut led: Output<'static>) {
    // For now, just blink periodically to show the system is alive
    // A more sophisticated version would track actual HPD state
    loop {
        led.toggle();
        Timer::after(Duration::from_millis(500)).await;
    }
}
