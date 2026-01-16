//! ### ROMSearchState
//! * 1-wire state for device search

// owrust project
// https://github.com/alfille/owrust
//
// This is a Rust version of my C owfs code for talking to 1-wire devices via owserver
// Basically owserver can talk to the physical devices, and provides network access via my "owserver protocol"
//
// MIT Licence
// {c} 2025 Paul H Alfille

use crate::rom_id::RomId;

#[derive(Debug, Clone)]
pub struct ROMSearchState {
    pub rom: RomId,
    pub last_discrepancy: i8,
    pub last_device_flag: bool,
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

