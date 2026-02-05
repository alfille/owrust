//! ### DS9490R 1-wire bus master
//! * uses the DS2490 chip internally
//! * native USB
//! * USB vendor-specific commands to control the 1-Wire bus.
//! * USB IDs: VendorID=0x04FA, ProductID=0x2490

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
use crate::bus_manage::BusManage;
use crate::bus_thread::BusThread;
use crate::rom_id::RomId;
use crate::search_state::ROMSearchState;

use anyhow::{Result,Context as _};
use rusb::{DeviceHandle, GlobalContext, Context, Direction, RequestType, Recipient};
use std::time::Duration;
use std::io::{Read, Write};

pub struct DS9490R {
    handle: DeviceHandle<GlobalContext>,
    description: String,
}

impl BusThread for DS9490R {
    fn get_description(&self) -> String {
        self.description.clone()
    }
}
impl BusReadWrite for DS9490R {
    /// Reset the 1-Wire bus. Returns true if a presence pulse is detected.
    /// Resets the bus and returns true if a Presence Pulse was detected.
    fn reset(&mut self) -> Result<bool> {
        // 1. Clear any pending data in the status pipe
        let mut trash = [0u8; 16];
        let _ = self.handle.read_interrupt(0x81, &mut trash, Duration::from_millis(1));

        // 2. Send the Reset Command
        // bRequest: COMM_CMD (0x01), wValue: OP_RESET (0x01)
        self.handle.write_control(
            0x40, 0x01, 0x01, 0x0000, &[], Duration::from_millis(100)
        )?;

        // 3. Read the Result Register from the Interrupt Endpoint (0x81)
        let mut status = [0u8; 16];
        let bytes_read = self.handle.read_interrupt(
            0x81, 
            &mut status, 
            Duration::from_millis(500) // Allow time for the 1-Wire reset timing
        )?;

        if bytes_read < 1 {
            return Ok(false);
        }

        // 4. Interpret the result
        // Bit 0: Short Detected
        // Bit 1: Presence Pulse Detected
        let short_detected = (status[0] & 0x01) != 0;
        let presence_detected = (status[0] & 0x02) != 0;

        if short_detected {
            return Err(rusb::Error::Other); // Bus is physically shorted
        }

        Ok(presence_detected)
    }
    fn write_byte(&mut self, byte: u8) -> Result<()> {
        // Prepare the chip for a 1-byte data transfer
        self.handle.write_control(
            0x40, DS9490R::COMM_CMD, DS9490R::OP_BYTE, 0x0000, &[], Duration::from_millis(100)
        )?;
        
        // Send the actual data to Bulk Out (EP2)
        self.handle.write_bulk(0x02, &[byte], Duration::from_millis(100))?;
        
        // Every write generates an echo on Bulk In (EP3); clear it
        let mut echo = [0u8; 1];
        self.handle.read_bulk(0x83, &mut echo, Duration::from_millis(100))?;
        Ok(())
    }

    fn read_byte(&mut self) -> Result<u8> {
        // To read, the master writes a 0xFF (all ones) to the bus
        self.handle.write_control(
            0x40, DS9490R::COMM_CMD, DS9490R::OP_BYTE, 0x0000, &[], Duration::from_millis(100)
        )?;
        self.handle.write_bulk(0x02, &[0xFF], Duration::from_millis(100))?;

        // The data returned by the slave is read via Bulk In (EP3)
        let mut buf = [0u8; 1];
        self.handle.read_bulk(0x83, &mut buf, Duration::from_millis(100))?;
        Ok(buf[0])
    }
}
impl BusSearch for DS9490R {}
impl BusManage for DS9490R {}

impl DS9490R {
    // USE Device
    const VID: u16 = 0x04FA;
    const PID: u16 = 0x2490;    
    
    // Mode Commands
    pub const DATA_MODE: u8 = 0xE1;
    pub const COMMAND_MODE: u8 = 0xE3;

    // Communication Speed
    pub const SPEED_REGULAR: u8 = 0x00;
    pub const SPEED_OVERDRIVE: u8 = 0x02;

    // 1-Wire Reset Command
    pub const RESET: u8 = 0xC1;

    // Reset Response Codes
    pub const RESET_PRESENCE: u8 = 0xCD; // Device present
    pub const RESET_NO_PRESENCE: u8 = 0xCC; // No device
    pub const RESET_SHORT: u8 = 0xC1; // Short detected

    // Strong Pullup
    pub const STRONG_PULLUP_5V: u8 = 0xED;
    pub const STRONG_PULLUP_12V: u8 = 0xEE;

    // 1-Wire ROM Commands (sent in data mode)
    pub fn new() -> Result<Self> {
       let handle = rusb::open_device_with_vid_pid(DS9490R::VID, DS9490R::PID)
            .context(format!("No DS9490R found with VID {} PID {}",DS9490R::VID,DS9490R::PID))?;
        let bus = Self {
            handle,
            description: "USB DS9490R bus master".to_string(),
        } ;
        
        // Claim the interface (required for bulk transfers)
        handle.claim_interface(0)?;
        Ok(bus)
    }

    /// Read a bit from the 1-Wire bus
    fn read_bit(&mut self) -> Result<bool> {
        // Read a full byte and check the LSB
        let byte = self.read_write_byte(0xFF)?;
        Ok((byte & 0x01) != 0)
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
        if self.reset()? == false {
            state.done();
            return Ok(false);
        }

        // 2. Setup Search Accelerator
        // wValue 0x0008 = Search Accelerator
        // wIndex 0x00F0 = Normal Search ROM command
        self.handle.write_control(0x40, 0x01, 0x0008, search_type as u16, &[], Duration::from_millis(100))?;

        // 3. Construct the "Search Path"
        // We start with the last ROM found.
        let mut path = state.rom;
        
        if state.last_discrepancy >= 0 {
            let bit_idx = state.last_discrepancy as usize;
            // Set the bit at the last discrepancy to '1' to explore the other side of the fork
            path[bit_idx / 8] |= 1 << (bit_idx % 8);
            
            // Clear all subsequent bits to '0' to find the "first" device on this new branch
            for i in (bit_idx + 1)..64 {
                path[i / 8] &= !(1 << (i % 8));
            }
        }

        // 4. Send the 8-byte path to Bulk-Out (EP2)
        self.handle.write_bulk(0x02, &path, Duration::from_millis(100))?;

        // 5. Read 16-byte response from Bulk-In (EP3)
        let mut response = [0u8; 16];
        self.handle.read_bulk(0x83, &mut response, Duration::from_millis(200))?;

        let discovered_rom = &response[0..8];
        let discrepancies = &response[8..16];

        // 6. Calculate the next discrepancy for the FUTURE search
        let mut next_fork = -1i8;
        for i in 0..64 {
            let byte_idx = i / 8;
            let bit_mask = 1 << (i % 8);

            // A discrepancy exists if the hardware flagged it (discrepancies bit = 1)
            // AND the ROM we just found has a '0' at that position.
            // This means a '1' path exists that we haven't explored yet.
            if (discrepancies[byte_idx] & bit_mask) != 0 && (discovered_rom[byte_idx] & bit_mask) == 0 {
                next_fork = i as i8;
            }
        }

        state.rom.copy_from_slice(discovered_rom);
        state.last_discrepancy = next_fork;
        
        if state.last_discrepancy == -1 { 
            state.done(); 
        }
        Ok(true)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ds9097u::DS9097U;
    #[test]
    fn t_9097e() {
        let bh = <DS9097U as BusThread>::spawn("/dev/ttyS0".to_string(), DS9097U::new);
        let d = bh.send(BusCmd::Description);
        assert!(d.is_ok())
    }
}
/*
    /// Skip ROM command (address all devices)
    pub fn skip_rom(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        self.reset()?;
        self.write_byte(OneWireCommands::ROM_SKIP)?;
        Ok(())
    }

    /// Check if a device is present
    pub fn is_present(&mut self, address: &RomId) -> Result<bool, Box<dyn std::error::Error>> {
        self.select(address)?;
        Ok(true)
    }
}
*/
