//! ### Bus_List struct
//! Low level commands to bus
//! * At reset, and byte level

// owrust project
// https://github.com/alfille/owrust
//
// This is a Rust version of my C owfs code for talking to 1-wire devices via owserver
// Basically owserver can talk to the physical devices, and provides network access via my "owserver protocol"
//
// MIT Licence
// {c} 2025 Paul H Alfille

use crate::bus_list::BusHandle;
use crate::rom_id::RomId;
use crate::search_state::ROMSearchState;
use anyhow::Result;
use std::sync::mpsc;
use std::thread;

pub struct BusQuery {
    cmd: BusCmd,
    my_tx: mpsc::Sender<BusReturn>,
}

impl BusQuery {
    pub fn new(cmd: BusCmd, my_tx: mpsc::Sender<BusReturn>) -> Self {
        Self { cmd, my_tx }
    }
}

#[derive(Clone)]
pub enum BusCmd {
    Reset,
    Description,
    ReadWrite(Vec<u8>),
    ResetReadWrite(Vec<u8>),
    Select(RomId),
    DirRegular,
    DirAlarm,
}

pub struct OneWireCommands;
impl OneWireCommands {
    pub const ROM_SEARCH: u8 = 0xF0;
    pub const ROM_READ: u8 = 0x33;
    pub const ROM_MATCH: u8 = 0x55;
    pub const ROM_SKIP: u8 = 0xCC;
    pub const ROM_ALARM_SEARCH: u8 = 0xEC;
}

#[derive(PartialEq)]
pub enum BusReturn {
    Bad,
    Good,
    Bool(bool),
    Bytes(Vec<u8>),
    String(String),
    RomDir(Vec<RomId>),
    DevDir(Vec<String>),
}

///pub trait BusThread: Send + Sync + 'static {
pub trait BusThread {
    /// Returns the presence pulse (true if any slaves)
    fn reset(&mut self) -> Result<BusReturn>;
    fn description(&self) -> Result<BusReturn> {
        Ok(BusReturn::String("Unspecified 1-wire bus".to_string()))
    }
    fn read_write(&mut self, data: Vec<u8>) -> Result<BusReturn>;
    fn reset_read_write(&mut self, data: Vec<u8>) -> Result<BusReturn> {
        self.reset()?;
        self.read_write(data)
    }
    /// Send a Match ROM command to select a specific device
    fn select(&mut self, rom: &RomId) -> Result<BusReturn>;
    fn directory_regular(&mut self) -> Result<BusReturn>;
    fn directory_alarm(&mut self) -> Result<BusReturn>;
    fn command(&mut self, cmd: BusCmd) -> Result<BusReturn> {
        match cmd {
            BusCmd::Reset => self.reset(),
            BusCmd::Description => self.description(),
            BusCmd::ReadWrite(data) => self.read_write(data),
            BusCmd::ResetReadWrite(data) => self.reset_read_write(data),
            BusCmd::Select(data) => self.select(&data),
            BusCmd::DirRegular => self.directory_regular(),
            BusCmd::DirAlarm => self.directory_alarm(),
        }
    }
    /// create the bus thread
    /// * Works with different typoes of buses
    /// * actual bus structure is created in thread
    /// * External BusHandle us just the address
    /// * Uses a factory patern to create the internal bus device
    ///
    /// Example:
    /// ```
    /// use owrust::bus_thread::BusThread;
    /// use owrust::ds9097e::DS9097E ;
    /// let _ = <DS9097E as BusThread>::spawn( "/dev/ttyS0".to_string(), |p| { DS9097E::new(p) } );
    /// ```
    fn spawn<T, F>(path: String, factory: F) -> BusHandle
    where
        T: BusThread + Send + 'static,
        F: FnOnce(String) -> Result<T> + Send + 'static,
    {
        let (tx, rx) = mpsc::channel::<BusQuery>();
        thread::spawn(move || {
            let mut bus = match factory(path) {
                Ok(b) => b,
                Err(e) => {
                    eprintln!("Could not create bus. {}", e);
                    return;
                }
            };
            while let Ok(req) = rx.recv() {
                let result = bus.command(req.cmd).unwrap_or(BusReturn::Bad);
                let _ = req.my_tx.send(result);
            }
        });
        BusHandle { tx }
    }
    fn read_bit(&mut self) -> Result<bool> ;
    fn write_bit(&mut self, bit:bool) -> Result<()> ;
    fn write_byte(&mut self, write:u8) -> Result<()> ;
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
        self.write_byte(search_type)?;

        let mut id_bit_number = 1;
        let mut last_zero = 0;
        let mut rom_byte_number = 0;
        let mut rom_byte_mask = 1u8;

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

            // Write the chosen direction back to the bus
            let _ = self.write_bit(search_direction)?;

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
