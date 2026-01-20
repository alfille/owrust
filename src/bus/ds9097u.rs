//! ### DS9097U 1-wire bus master struct
//! * uses the DS2480B chip internally
//! * Serial port
//! * At reset, and byte level
//!
//! The DS2480B is an intelligent serial-to-1-Wire bridge that handles all timing
//! internally. Unlike the passive DS9097E, it uses a command protocol where
//! data and control information are merged into a single byte stream.

// owrust project
// https://github.com/alfille/owrust
//
// This is a Rust version of my C owfs code for talking to 1-wire devices via owserver
// Basically owserver can talk to the physical devices, and provides network access via my "owserver protocol"
//
// MIT Licence
// {c} 2025 Paul H Alfille

use crate::bus_rw::BusReadWrite;
use crate::bus_search::BusSearch;
use crate::bus_thread::BusThread;
use anyhow::{Context, Result};
use serialport::{DataBits, Parity, SerialPort, StopBits};
use std::io::{Read, Write};
use std::time::Duration;

#[derive(PartialEq)]
enum DS2480Mode {
    Data,
    Command,
}

pub struct DS9097U {
    port: Box<dyn SerialPort>,
    mode: DS2480Mode,
    description: String,
}

impl BusReadWrite for DS9097U {
    /// Reset the 1-Wire bus. Returns true if a presence pulse is detected.
    fn reset(&mut self) -> Result<bool> {
        self.set_command_mode()?;

        // Send reset command
        self.port.write_all(&[DS9097U::RESET])?;
        self.port.flush()?;

        // Read response
        let mut buf = [0u8; 1];
        self.port.read_exact(&mut buf)?;

        Ok(buf[0] == DS9097U::RESET_PRESENCE)
    }

    fn read_byte(&mut self) -> Result<u8> {
        self.read_write_byte(0xFF)
    }
    fn write_byte(&mut self, write: u8) -> Result<()> {
        let _ = self.read_write_byte(write)?;
        Ok(())
    }
}
impl BusThread for DS9097U {
    fn get_description(&self) -> String {
        self.description.clone()
    }
}
impl BusSearch for DS9097U {}

impl DS9097U {
    // Mode Commands
    pub const DATA_MODE: u8 = 0xE1;
    pub const COMMAND_MODE: u8 = 0xE3;

    // Communication Speed
    pub const SPEED_REGULAR: u8 = 0x00;
    pub const SPEED_OVERDRIVE: u8 = 0x02;

    // 1-Wire Reset Command
    pub const RESET: u8 = 0xC1;

    // Reset Response Codes
    const RESET_PRESENCE: u8 = 0xCD; // Device present
    const RESET_NO_PRESENCE: u8 = 0xCC; // No device
    const RESET_SHORT: u8 = 0xC1; // Short detected

    // Strong Pullup
    const STRONG_PULLUP_5V: u8 = 0xED;
    const STRONG_PULLUP_12V: u8 = 0xEE;

    // 1-Wire ROM Commands (sent in data mode)

    pub fn new<S>(path: S) -> Result<Self>
    where
        S: AsRef<str> + std::fmt::Display,
    {
        let port = serialport::new(path.to_string(), 115_200)
            .data_bits(DataBits::Eight)
            .stop_bits(StopBits::One)
            .parity(Parity::None)
            .timeout(Duration::from_millis(500))
            .open()
            .context("Failed to open serial port")?;
        let mut bus = DS9097U {
            port,
            mode: DS2480Mode::Command,
            description: format!("DS9097U serial bus-master at {}", path),
        };
        bus.initialize()?;
        Ok(bus)
    }

    /// Initialize the DS2480B chip
    /// Uses DTR to reset to initialized state
    fn initialize(&mut self) -> Result<()> {
        // Send a break to reset the DS2480B
        self.port.write_request_to_send(false)?;
        std::thread::sleep(Duration::from_millis(2));
        self.port.write_request_to_send(true)?;
        std::thread::sleep(Duration::from_millis(2));

        // Flush any pending data
        self.port.clear(serialport::ClearBuffer::All)?;

        // Send reset command for timing calibration
        self.port.write_all(&[DS9097U::RESET])?;
        self.port.flush()?;

        let mut buf = [0u8; 1];
        match self.port.read_exact(&mut buf) {
            Ok(_) => {}
            Err(_) => {
                // If timeout, try again
                self.port.write_all(&[DS9097U::RESET])?;
                self.port.flush()?;
                self.port.read_exact(&mut buf)?;
            }
        }

        // Switch to command mode
        self.set_command_mode()?;

        Ok(())
    }

    /// Switch to data mode
    fn set_data_mode(&mut self) -> Result<()> {
        if self.mode == DS2480Mode::Command {
            self.port.write_all(&[DS9097U::DATA_MODE])?;
            self.port.flush()?;
            self.mode = DS2480Mode::Data;
        }
        Ok(())
    }
    fn set_speed(&mut self, overdrive: bool) -> Result<()> {
        self.set_command_mode()?;

        let speed_cmd = if overdrive {
            DS9097U::SPEED_OVERDRIVE
        } else {
            DS9097U::SPEED_REGULAR
        };

        self.port.write_all(&[0x17 | speed_cmd])?; // Speed change command
        self.port.flush()?;

        Ok(())
    }

    /// Switch to command mode
    fn set_command_mode(&mut self) -> Result<()> {
        if self.mode == DS2480Mode::Data {
            self.port.write_all(&[DS9097U::COMMAND_MODE])?;
            self.port.flush()?;
            self.mode = DS2480Mode::Command;
        }
        Ok(())
    }

    fn read_write_byte(&mut self, write: u8) -> Result<u8> {
        self.set_data_mode()?;

        // In data mode, sending a byte writes it to the 1-Wire bus
        self.port.write_all(&[write])?;
        self.port.flush()?;

        // DS2480B returns the actual data read from the bus
        let mut buf = [0u8; 1];
        self.port.read_exact(&mut buf)?;

        Ok(buf[0])
    }
}
