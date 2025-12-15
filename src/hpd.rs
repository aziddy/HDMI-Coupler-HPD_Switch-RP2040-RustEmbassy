//! HDMI Hot Plug Detect Control Module
//!
//! This module provides utilities for controlling HPD signals to trigger
//! HDMI re-negotiation, EDID re-reads, and connection state management.

use defmt::*;
use embassy_rp::gpio::Output;
use embassy_time::{Duration, Timer};

/// Timing constants for HPD control
pub mod timing {
    use embassy_time::Duration;
    
    /// Minimum HPD low pulse to trigger re-read of EDID (per HDMI spec)
    /// HDMI spec requires minimum 100ms low pulse
    pub const HPD_PULSE_MIN: Duration = Duration::from_millis(100);
    
    /// Recommended HPD low pulse duration for reliable detection
    pub const HPD_PULSE_RECOMMENDED: Duration = Duration::from_millis(200);
    
    /// Long HPD pulse for forcing full re-negotiation
    pub const HPD_PULSE_LONG: Duration = Duration::from_millis(500);
    
    /// Delay after asserting HPD before source typically reads EDID
    pub const EDID_READ_DELAY: Duration = Duration::from_millis(50);
    
    /// Debounce time for button inputs
    pub const DEBOUNCE: Duration = Duration::from_millis(50);
}

/// HPD Controller state
#[derive(Clone, Copy, Debug, Format, PartialEq)]
pub enum HpdState {
    /// HPD asserted - sink appears connected to source
    Connected,
    /// HPD de-asserted - sink appears disconnected
    Disconnected,
    /// Currently pulsing HPD (transient state)
    Pulsing,
}

/// HPD Controller for managing hot plug detection
pub struct HpdController<'a> {
    pin: Output<'a>,
    state: HpdState,
}

impl<'a> HpdController<'a> {
    /// Create a new HPD controller
    /// 
    /// Initially sets HPD to de-asserted (low) state
    pub fn new(pin: Output<'a>) -> Self {
        Self {
            pin,
            state: HpdState::Disconnected,
        }
    }
    
    /// Get current HPD state
    pub fn state(&self) -> HpdState {
        self.state
    }
    
    /// Assert HPD (signal that sink is connected)
    pub fn assert(&mut self) {
        self.pin.set_high();
        self.state = HpdState::Connected;
        info!("HPD asserted (connected)");
    }
    
    /// De-assert HPD (signal that sink is disconnected)
    pub fn deassert(&mut self) {
        self.pin.set_low();
        self.state = HpdState::Disconnected;
        info!("HPD de-asserted (disconnected)");
    }
    
    /// Toggle HPD state
    pub fn toggle(&mut self) {
        match self.state {
            HpdState::Connected => self.deassert(),
            HpdState::Disconnected => self.assert(),
            HpdState::Pulsing => {} // Don't toggle during pulse
        }
    }
    
    /// Pulse HPD low to trigger EDID re-read
    /// 
    /// This is useful when you want to force the source to re-read
    /// EDID from the sink without a full disconnect cycle.
    /// 
    /// Uses the recommended pulse duration (200ms)
    pub async fn pulse(&mut self) {
        self.pulse_duration(timing::HPD_PULSE_RECOMMENDED).await;
    }
    
    /// Pulse HPD low for a specific duration
    pub async fn pulse_duration(&mut self, duration: Duration) {
        let was_connected = self.state == HpdState::Connected;
        
        self.state = HpdState::Pulsing;
        self.pin.set_low();
        info!("HPD pulse started ({} ms)", duration.as_millis());
        
        Timer::after(duration).await;
        
        if was_connected {
            self.pin.set_high();
            self.state = HpdState::Connected;
        } else {
            self.state = HpdState::Disconnected;
        }
        
        info!("HPD pulse complete");
    }
    
    /// Perform a full disconnect/reconnect cycle
    /// 
    /// This forces the source to completely re-negotiate the connection
    pub async fn reconnect_cycle(&mut self) {
        info!("Starting full reconnect cycle");
        
        self.deassert();
        Timer::after(timing::HPD_PULSE_LONG).await;
        
        self.assert();
        info!("Reconnect cycle complete");
    }
}

/// Commands that can be sent to the HPD controller
#[derive(Clone, Copy, Debug, Format)]
pub enum HpdCommand {
    /// Assert HPD (connect)
    Assert,
    /// De-assert HPD (disconnect)
    Deassert,
    /// Toggle current state
    Toggle,
    /// Pulse HPD for EDID re-read
    Pulse,
    /// Full reconnect cycle
    Reconnect,
}
