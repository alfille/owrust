//! ### DS9097U 1-wire bus master struct
//! * uses the DS2480B chip internally
//! * Serial port
//! * At reset, and byte level

// owrust project
// https://github.com/alfille/owrust
//
// This is a Rust version of my C owfs code for talking to 1-wire devices via owserver
// Basically owserver can talk to the physical devices, and provides network access via my "owserver protocol"
//
// MIT Licence
// {c} 2025 Paul H Alfille

use crate::bus_thread::{BusCmd, BusReturn, BusThread};
use crate::rom_id::RomId;
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
    fn read_write(&mut self, data: Vec<u8>) -> Result<BusReturn> {
        let mut read = Vec::<u8>::new();
        for byte in &data {
            read.push(self.read_write_byte(*byte)?);
        }
        Ok(BusReturn::Bytes(read))
    }
    fn reset_read_write(&mut self, data: Vec<u8>) -> Result<BusReturn> {
        self.reset()?;
        self.read_write(data)
    }
    fn select(&mut self, rom: RomId) -> Result<BusReturn> {
        self.reset()?;
        self.read_write_byte(BusCmd::ROM_MATCH)?; // MATCH ROM command
        for byte in &rom.0 {
            self.read_write_byte(*byte)?;
        }
        Ok(BusReturn::Good)
    }
    fn directory_regular(&mut self) -> Result<BusReturn> {
        Ok(BusReturn::RomDir(self.search(BusCmd::ROM_SEARCH)?))
    }
    fn directory_alarm(&mut self) -> Result<BusReturn> {
        Ok(BusReturn::RomDir(self.search(BusCmd::ROM_ALARM_SEARCH)?))
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
    pub const RESET_PRESENCE: u8 = 0xCD;  // Device present
    pub const RESET_NO_PRESENCE: u8 = 0xCC;  // No device
    pub const RESET_SHORT: u8 = 0xC1;  // Short detected
    
    // Strong Pullup
    pub const STRONG_PULLUP_5V: u8 = 0xED;
    pub const STRONG_PULLUP_12V: u8 = 0xEE;
    
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
        Ok(DS9097U {
            port,
            mode: DS2480Mode::Command,
            description: format!("DS9097U serial bus-master at {}", path),
        })
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
            Ok(_) => {},
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
    fn set_data_mode(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        if self.mode==DS2480Mode::Command {
            self.port.write_all(&[DS9097U::DATA_MODE])?;
            self.port.flush()?;
            self.mode = DS2480Mode::Data;
        }
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

    fn read_write_bit(&mut self, bit: bool) -> Result<bool> {
        let data = if bit { 0xFF } else { 0x00 };
        self.port.write_all(&[data])?;
        let mut buf = [0u8; 1];
        self.port.read_exact(&mut buf)?;
        Ok(buf[0] == 0xFF) // If bit is 1, we get 0xFF back
    }

    fn read_write_byte(&mut self, write: u8) -> Result<u8> {
        let mut read = 0u8;
        let mut probe = 1u8;
        for _ in 0..8 {
            if self.read_write_bit((write & probe) != 0u8)? {
                read |= probe;
            }
            probe <<= 1;
        }
        Ok(read)
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

        // Perform the search algorithm
        while rom_byte_number < 8 {
            // Read bit and its complement
            let id_bit = self.read_write_bit(true)?;
            let cmp_id_bit = self.read_write_bit(true)?;

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

            // Write the chosen direction back to the bus
            let _ = self.read_write_bit(search_direction)?;

            // Track the last discrepancy
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
    pub fn enable_power(&mut self) -> Result<()> {
        // High level (logical true) on DTR/RTS provides the positive voltage
        self.port.write_data_terminal_ready(true)?;
        self.port.write_request_to_send(true)?;
        
        // It is often helpful to wait a few milliseconds for 
        // parasitic capacitors on the bus to charge up.
        std::thread::sleep(std::time::Duration::from_millis(50));
        
        Ok(())
    }

    pub fn disable_power(&mut self) -> Result<()> {
        self.port.write_data_terminal_ready(false)?;
        self.port.write_request_to_send(false)?;
        Ok(())
    }}

#[derive(Debug, Clone)]
struct ROMSearchState {
    rom: RomId,
    last_discrepancy: i8,
    last_device_flag: bool,
}

impl ROMSearchState {
    /// Creates a state initialized for the very first search
    pub fn new() -> Self {
        Self {
            rom: RomId::blank(),
            last_discrepancy: -1,
            last_device_flag: false,
        }
    }
    pub fn done(&mut self) {
        self.last_device_flag = true;
    }
    pub fn is_done(&self) -> bool {
        self.last_device_flag
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ds9097e::DS9097E;
    #[test]
    fn t_9097e() {
        let bh = <DS9097E as BusThread>::spawn("/dev/ttyS0".to_string(), DS9097E::new);
        let d = bh.send(BusCmd::Description);
        assert!(d.is_ok())
    }
}


/*
/// 1-Wire Search Algorithm Implementation for DS9097U (DS2480B-based adapter)
/// 
/// The DS2480B is an intelligent serial-to-1-Wire bridge that handles all timing
/// internally. Unlike the passive DS9097E, it uses a command protocol where
/// data and control information are merged into a single byte stream.

use std::io::{Read, Write};
use std::time::Duration;
use serialport::{SerialPort, DataBits, StopBits, Parity};

/// Represents a 64-bit 1-Wire device address
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RomId([u8; 8]);

impl RomId {
    /// Create a new device address from 8 bytes
    pub fn new(bytes: [u8; 8]) -> Self {
        Self(bytes)
    }

    /// Get the family code (first byte)
    pub fn family_code(&self) -> u8 {
        self.0[0]
    }

    /// Get the serial number (middle 6 bytes)
    pub fn serial_number(&self) -> [u8; 6] {
        let mut serial = [0u8; 6];
        serial.copy_from_slice(&self.0[1..7]);
        serial
    }

    /// Get the CRC (last byte)
    pub fn crc(&self) -> u8 {
        self.0[7]
    }

    /// Verify the CRC is valid
    pub fn is_valid_crc(&self) -> bool {
        compute_crc8(&self.0[0..7]) == self.crc()
    }

    /// Format address as a hex string
    pub fn to_hex_string(&self) -> String {
        self.0.iter()
            .map(|b| format!("{:02X}", b))
            .collect::<Vec<_>>()
            .join("")
    }

    /// Get the raw bytes
    pub fn as_bytes(&self) -> &[u8; 8] {
        &self.0
    }
}

/// 1-Wire bus controller for DS9097U (DS2480B-based adapter)
pub struct OneWireBus {
    port: Box<dyn SerialPort>,
    in_data_mode: bool,
}

impl OneWireBus {
    /// Create a new 1-Wire bus using the DS9097U adapter
    /// 
    /// # Arguments
    /// * `port_name` - Serial port device (e.g., "/dev/ttyUSB0" or "COM3")
    pub fn new(port_name: &str) -> Result<Self, Box<dyn std::error::Error>> {
        // DS2480B defaults to 9600 baud on power-up
        let mut port = serialport::new(port_name, 9600)
            .data_bits(DataBits::Eight)
            .stop_bits(StopBits::One)
            .parity(Parity::None)
            .timeout(Duration::from_millis(500))
            .open()?;

        let mut bus = Self {
            port,
            in_data_mode: false,
        };

        // Initialize the DS2480B
        bus.initialize()?;

        Ok(bus)
    }

    /// Write a byte to the 1-Wire bus (must be in data mode)
    pub fn write_byte(&mut self, byte: u8) -> Result<(), Box<dyn std::error::Error>> {
        self.set_data_mode()?;

        // In data mode, sending a byte writes it to the 1-Wire bus
        self.port.write_all(&[byte])?;
        self.port.flush()?;

        // DS2480B echoes back the data
        let mut response = [0u8; 1];
        self.port.read_exact(&mut response)?;

        Ok(())
    }

    /// Read a byte from the 1-Wire bus (must be in data mode)
    pub fn read_byte(&mut self) -> Result<u8, Box<dyn std::error::Error>> {
        self.set_data_mode()?;

        // Send 0xFF to read (this generates read time slots)
        self.port.write_all(&[0xFF])?;
        self.port.flush()?;

        // DS2480B returns the actual data read from the bus
        let mut buf = [0u8; 1];
        self.port.read_exact(&mut buf)?;

        Ok(buf[0])
    }

    /// Read a bit from the 1-Wire bus
    fn read_bit(&mut self) -> Result<bool, Box<dyn std::error::Error>> {
        // Read a full byte and check the LSB
        let byte = self.read_byte()?;
        Ok((byte & 0x01) != 0)
    }

    /// Perform search ROM algorithm to find all devices
    pub fn search(&mut self) -> Result<Vec<RomId>, Box<dyn std::error::Error>> {
        let mut devices = Vec::new();
        let mut search_state = SearchState::new();

        loop {
            match self.search_next(&mut search_state)? {
                Some(address) => {
                    if address.is_valid_crc() {
                        devices.push(address);
                    }
                    if search_state.last_device {
                        break;
                    }
                }
                None => break,
            }
        }

        Ok(devices)
    }

    /// Search for the next device on the bus
    fn search_next(&mut self, state: &mut SearchState) -> Result<Option<RomId>, Box<dyn std::error::Error>> {
        // Reset and check for presence
        if !self.reset()? {
            state.last_device = true;
            return Ok(None);
        }

        // Send Search ROM command
        self.write_byte(ds2480b_commands::ROM_SEARCH)?;

        let mut id_bit_number = 1;
        let mut last_zero = 0;
        let mut rom_byte_number = 0;
        let mut rom_byte_mask = 1u8;

        // Perform the search algorithm - read 64 bit pairs
        while rom_byte_number < 8 {
            // Read the bit and its complement
            // In data mode, we read actual bits from the bus
            let id_bit = self.read_bit()?;
            let cmp_id_bit = self.read_bit()?;

            // Determine search direction
            let search_direction = if id_bit && cmp_id_bit {
                // No devices responded
                return Ok(None);
            } else if id_bit != cmp_id_bit {
                // All devices have same bit at this position
                id_bit
            } else {
                // Discrepancy - make a choice
                if id_bit_number < state.last_discrepancy {
                    // Follow previous path
                    (state.rom[rom_byte_number] & rom_byte_mask) != 0
                } else if id_bit_number == state.last_discrepancy {
                    // Take the 1 branch
                    true
                } else {
                    // Take the 0 branch
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
            self.write_byte(if search_direction { 0xFF } else { 0x00 })?;

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

        // Update search state
        state.last_discrepancy = last_zero;
        
        if state.last_discrepancy == 0 {
            state.last_device = true;
        }

        Ok(Some(RomId::new(state.rom)))
    }

    /// Select a specific device using Match ROM
    pub fn select(&mut self, address: &RomId) -> Result<(), Box<dyn std::error::Error>> {
        self.reset()?;
        self.write_byte(ds2480b_commands::ROM_MATCH)?;
        
        for &byte in address.as_bytes() {
            self.write_byte(byte)?;
        }
        
        Ok(())
    }

    /// Skip ROM command (address all devices)
    pub fn skip_rom(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        self.reset()?;
        self.write_byte(ds2480b_commands::ROM_SKIP)?;
        Ok(())
    }

    /// Check if a device is present
    pub fn is_present(&mut self, address: &RomId) -> Result<bool, Box<dyn std::error::Error>> {
        self.select(address)?;
        Ok(true)
    }

    /// Read bytes from the bus (convenience method)
    pub fn read_bytes(&mut self, count: usize) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        let mut data = Vec::with_capacity(count);
        for _ in 0..count {
            data.push(self.read_byte()?);
        }
        Ok(data)
    }

    /// Write bytes to the bus (convenience method)
    pub fn write_bytes(&mut self, data: &[u8]) -> Result<(), Box<dyn std::error::Error>> {
        for &byte in data {
            self.write_byte(byte)?;
        }
        Ok(())
    }

    /// Set communication speed
    pub fn set_speed(&mut self, overdrive: bool) -> Result<(), Box<dyn std::error::Error>> {
        self.set_command_mode()?;
        
        let speed_cmd = if overdrive {
            ds2480b_commands::SPEED_OVERDRIVE
        } else {
            ds2480b_commands::SPEED_REGULAR
        };
        
        self.port.write_all(&[0x17 | speed_cmd])?;  // Speed change command
        self.port.flush()?;
        
        Ok(())
    }
}

/// State maintained between search iterations
struct SearchState {
    rom: [u8; 8],
    last_discrepancy: u8,
    last_device: bool,
}

impl SearchState {
    fn new() -> Self {
        Self {
            rom: [0; 8],
            last_discrepancy: 0,
            last_device: false,
        }
    }
}
*/
