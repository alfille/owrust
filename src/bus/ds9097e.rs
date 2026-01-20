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

use crate::bus_rw::BusReadWrite;
use crate::bus_search::BusSearch;
use crate::bus_spawn::BusSpawn;
use crate::bus_thread::BusThread;
use anyhow::{Context, Result};
use serialport::{DataBits, Parity, SerialPort, StopBits};
use std::io::{Read, Write};
use std::time::Duration;

pub struct DS9097E {
    port: Box<dyn SerialPort>,
    description: String,
}
impl BusSpawn for DS9097E {}

impl BusReadWrite for DS9097E {
    /// Reset the 1-Wire bus. Returns true if a presence pulse is detected.
    fn reset(&mut self) -> Result<bool> {
        // To generate a Reset pulse (>480us), we drop the baud rate.
        self.port.set_baud_rate(9600)?;
        self.port.write_all(&[0xF0])?;

        let mut buf = [0u8; 1];
        self.port.read_exact(&mut buf)?;

        self.port.set_baud_rate(115_200)?;
        // If the byte read back is NOT 0xF0, a device pulled the line low (Presence).
        Ok(buf[0] != 0xF0)
    }
    fn read_bit(&mut self) -> Result<bool> {
        self.read_write_bit(true)
    }
    fn write_bit(&mut self, bit: bool) -> Result<()> {
        let _ = self.read_write_bit(bit)?;
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
            let _ = self.read_write_bit((write & probe) != 0u8)?;
            probe <<= 1;
        }
        Ok(())
    }
}
impl BusThread for DS9097E {
    fn get_description(&self) -> String {
        self.description.clone()
    }
}
impl BusSearch for DS9097E {}

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
