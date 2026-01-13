//! ### DS9097E 1-wire bus master struct
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

pub struct DS9097E {
    port: Box<dyn SerialPort>,
    description: String,
}

impl BusThread for DS9097E {
    /// Reset the 1-Wire bus. Returns true if a presence pulse is detected.
    fn reset(&mut self) -> Result<BusReturn> {
        // To generate a Reset pulse (>480us), we drop the baud rate.
        self.port.set_baud_rate(9600)?;
        self.port.write_all(&[0xF0])?;

        let mut buf = [0u8; 1];
        self.port.read_exact(&mut buf)?;

        self.port.set_baud_rate(115_200)?;
        // If the byte read back is NOT 0xF0, a device pulled the line low (Presence).
        Ok(BusReturn::Bool(buf[0] != 0xF0))
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

impl DS9097E {
    pub fn new<S>(path: S) -> Result<Self>
    where
        S: AsRef<str> + std::fmt::Display,
    {
        let port = serialport::new(path.to_string(), 115_200)
            .timeout(Duration::from_millis(100))
            .data_bits(DataBits::Six)
            .stop_bits(StopBits::One)
            .parity(Parity::None)
            .open()
            .context("Failed to open serial port")?;
        Ok(DS9097E {
            port,
            description: format!("DS9097E passive serial bus-master at {}", path),
        })
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
