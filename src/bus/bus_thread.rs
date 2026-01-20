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

use crate::bus_rw::BusReadWrite;
use crate::bus_search::BusSearch;
use crate::rom_id::RomId;
use anyhow::Result;

#[derive(Clone)]
pub enum BusCmd {
    Reset,
    Description,
    Select(RomId),
    Read(usize),
    Write(Vec<u8>),
    Search,
    AlarmSearch,
    Power(bool),
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
pub trait BusThread: BusReadWrite + BusSearch {
    fn get_description(&self) -> String;
    fn command(&mut self, cmd: BusCmd) -> Result<BusReturn> {
        match cmd {
            BusCmd::Reset => {
                let present = self.reset()?;
                Ok(BusReturn::Bool(present))
            }
            BusCmd::Description => Ok(BusReturn::String(self.get_description())),
            BusCmd::Select(rom) => {
                self.select(&rom)?;
                Ok(BusReturn::Good)
            }
            BusCmd::Read(n) => {
                let v = self.read_bytes(n)?;
                Ok(BusReturn::Bytes(v))
            }
            BusCmd::Write(data) => {
                self.write_bytes(data)?;
                Ok(BusReturn::Good)
            }
            BusCmd::Search => {
                let r = self.directory_regular()?;
                Ok(BusReturn::RomDir(r))
            }
            BusCmd::AlarmSearch => {
                let r = self.directory_alarm()?;
                Ok(BusReturn::RomDir(r))
            }
            BusCmd::Power(_bool) => Ok(BusReturn::Bad),
        }
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
