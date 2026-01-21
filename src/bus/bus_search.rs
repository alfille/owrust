//! ### Bus Search performs the 1-wire device discovery

// owrust project
// https://github.com/alfille/owrust
//
// This is a Rust version of my C owfs code for talking to 1-wire devices via owserver
// Basically owserver can talk to the physical devices, and provides network access via my "owserver protocol"
//
// MIT Licence
// {c} 2025 Paul H Alfille

use crate::bus_rw::BusReadWrite;
use crate::bus_thread::OneWireCommands;
use crate::rom_id::RomId;
use crate::search_state::ROMSearchState;
use anyhow::Result;

pub trait BusSearch: BusReadWrite {
    /// Send a Match ROM command to select a specific device
    fn select(&mut self, rom: &RomId) -> Result<()> {
        self.reset()?;
        let mut v = vec![ OneWireCommands::ROM_MATCH, ] ;
        v.append( rom.as_bytes() ) ;
        self.write_bytes( v ) ;
        Ok(())
    }
    fn directory_regular(&mut self) -> Result<Vec<RomId>> {
        let list = self.search(OneWireCommands::ROM_SEARCH)?;
        Ok(list)
    }
    fn directory_alarm(&mut self) -> Result<Vec<RomId>> {
        let list = self.search(OneWireCommands::ROM_ALARM_SEARCH)?;
        Ok(list)
    }

    /// General 1-wire search for all devices  (regular or alarm)
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

    fn search_next(&mut self, state: &mut ROMSearchState, search_type: u8) -> Result<bool> {
        // Check for presence pulse
        if !self.reset()? {
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
            self.write_bit(search_direction)?;

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
