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

use crate::bus_thread::{BusReturn, BusThread, OneWireCommands};
use crate::bus_rw::BusReadWrite;
use crate::rom_id::RomId;
use crate::search_state::ROMSearchState;
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
    fn read_byte(&mut self) -> Result<u8> {
        self.read_write_byte( 0xFF )
    }
    fn write_byte(&mut self, write: u8) -> Result<()> {
        let _ = self.read_write_byte(write)?;
        Ok(())
    }
}
impl BusThread for DS9097U {
    /// Reset the 1-Wire bus. Returns true if a presence pulse is detected.
    fn reset(&mut self) -> Result<BusReturn> {
        self.set_command_mode()?;

        // Send reset command
        self.port.write_all(&[DS9097U::RESET])?;
        self.port.flush()?;

        // Read response
        let mut buf = [0u8; 1];
        self.port.read_exact(&mut buf)?;

        Ok(BusReturn::Bool(buf[0] == DS9097U::RESET_PRESENCE))
    }

    fn description(&self) -> Result<BusReturn> {
        Ok(BusReturn::String("Unspecified 1-wire bus".to_string()))
    }
    fn reset_read_write(&mut self, data: Vec<u8>) -> Result<BusReturn> {
        self.reset()?;
        self.read_write(data)
    }
    fn select(&mut self, rom: &RomId) -> Result<BusReturn> {
        self.reset()?;
        self.read_write_byte(OneWireCommands::ROM_MATCH)?; // MATCH ROM command
        for byte in rom.as_bytes() {
            self.read_write_byte(*byte)?;
        }
        Ok(BusReturn::Good)
    }
    fn directory_regular(&mut self) -> Result<BusReturn> {
        Ok(BusReturn::RomDir(self.search(OneWireCommands::ROM_SEARCH)?))
    }
    fn directory_alarm(&mut self) -> Result<BusReturn> {
        Ok(BusReturn::RomDir(
            self.search(OneWireCommands::ROM_ALARM_SEARCH)?,
        ))
    }
}

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

    fn search(&mut self, search_type: u8) -> Result<Vec<RomId>> {
        let mut state = ROMSearchState::new();
        let mut rom_list = Vec::new();
        loop {
            if self.search_next(&mut state, search_type)? {
                rom_list.push(state.rom);
                if state.is_done() {
                    break;
                }
            } else {
                break;
            }
        }
        Ok(rom_list)
    }

    /// Search for the next device on the bus
    ///
    /// This implements the core 1-Wire search algorithm using the binary tree
    /// search method. It handles discrepancies by exploring all branches.
    fn search_next(&mut self, state: &mut ROMSearchState, search_type: u8) -> Result<bool> {
        // Check for presence pulse
        if self.reset()? == BusReturn::Bool(false) {
            state.done();
            return Ok(false);
        }

        // Send search ROM command (0xF0)
        self.read_write_byte(search_type)?;

        let mut id_bit_number = 1;
        let mut last_zero = 0;
        let mut rom_byte_number = 0;
        let mut rom_byte_mask = 1u8;
        // Send Search ROM command

        // Perform the search algorithm
        while rom_byte_number < 8 {
            // Read bit and its complement
            let id_bit = self.read_bit()?;
            let cmp_id_bit = self.read_bit()?;

            // Check for search conflict or errors
            let search_direction = if id_bit && cmp_id_bit {
                // No devices responded
                return Ok(false);
            } else if id_bit != cmp_id_bit {
                // All devices have the same bit value at this position
                id_bit
            } else {
                // Discrepancy: both 0s and 1s present
                // This is where the search algorithm makes its choice
                if id_bit_number < state.last_discrepancy {
                    // Follow previous path
                    (state.rom[rom_byte_number] & rom_byte_mask) != 0
                } else if id_bit_number == state.last_discrepancy {
                    // Take the 1 branch at the last discrepancy point
                    true
                } else {
                    // Take the 0 branch for new discrepancies
                    false
                }
            };

            // Update ROM bit
            if search_direction {
                state.rom[rom_byte_number] |= rom_byte_mask;
            } else {
                state.rom[rom_byte_number] &= !rom_byte_mask;
            }

            // Write the search direction back to the bus
            // Write 1 or 0 to continue the search
            let _ = self.read_write_byte(if search_direction { 0xFF } else { 0x00 })?;

            // Track discrepancies
            if !id_bit && !cmp_id_bit && !search_direction {
                last_zero = id_bit_number;
            }

            // Move to next bit
            id_bit_number += 1;
            rom_byte_mask <<= 1;

            if rom_byte_mask == 0 {
                rom_byte_number += 1;
                rom_byte_mask = 1;
            }
        }

        // Update search state for next iteration
        state.last_discrepancy = last_zero;

        if state.last_discrepancy == 0 {
            state.done();
        }

        Ok(true)
    }
}
