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

use crate::bus_thread::{BusReturn, BusThread, OneWireCommands};
use crate::bus_rw::BusReadWrite;
use crate::rom_id::RomId;
use crate::search_state::ROMSearchState;
use anyhow::{Context, Result};
use serialport::{DataBits, Parity, SerialPort, StopBits};
use std::io::{Read, Write};
use std::time::Duration;

pub struct DS9097E {
    port: Box<dyn SerialPort>,
    description: String,
}

impl BusReadWrite for DS9097E {
    fn read_bit(&mut self) -> Result<bool> {
        self.read_write_bit(true)
    }
    fn write_bit(&mut self, bit:bool) -> Result<()> {
        let _ = self.read_write_bit(bit) ? ;
        Ok(())
    }
    fn read_byte(&mut self) -> Result<u8> {
        let mut read = 0u8;
        let mut probe = 1u8;
        for _ in 0..8 {
            if self.read_write_bit(true)? {
                read |= probe;
            }
            probe <<= 1;
        }
        Ok(read)
    }
    fn write_byte(&mut self, write: u8) -> Result<()> {
        let mut probe = 1u8;
        for _ in 0..8 {
            let _ = self.read_write_bit((write & probe) != 0u8)? ;
            probe <<= 1;
        }
        Ok(())
    }
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
    fn select(&mut self, rom: &RomId) -> Result<BusReturn> {
        self.reset()?;
        self.write_byte(OneWireCommands::ROM_MATCH)?; // MATCH ROM command
        for byte in &rom.0 {
            self.write_byte(*byte)?;
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
    
    /// General 1-wir search for all devices  (regular or alarm)
    /// Since the bus master is error-prone, try 3 times before returning an error
    fn search(&mut self, search_type: u8) -> Result<Vec<RomId>> {
        let mut state = ROMSearchState::new();
        let mut rom_list = Vec::new();
        loop {
            let saved_state = state.clone();
            // First pass
            if let Ok(found) = self.search_next(&mut state, search_type) {
                if !found {
                    break;
                }
                if state.valid_rom() {
                    rom_list.push(state.rom);
                    if !state.is_done() {
                        continue;
                    }
                }
            }
            state = saved_state.clone();
            // second pass
            if let Ok(found) = self.search_next(&mut state, search_type) {
                if !found {
                    break;
                }
                if state.valid_rom() {
                    rom_list.push(state.rom);
                    if !state.is_done() {
                        continue;
                    }
                }
            }
            state = saved_state.clone();
            // third pass
            // io error are returned. CRC error is signalled
            let found = self.search_next(&mut state, search_type)?;
            if !found {
                break;
            }
            if state.valid_rom() {
                rom_list.push(state.rom);
                if !state.is_done() {
                    continue;
                }
            } else {
                return Err(anyhow::anyhow!("Bad CRC"));
            }
        }
        Ok(rom_list)
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
    }
}
