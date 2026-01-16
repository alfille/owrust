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
