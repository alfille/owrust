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
impl BusCmd {
    pub const SEARCH: u8 = 0xF0;
    pub const ALARM: u8 = 0xEC;
    pub const SELECT: u8 = 0x55;
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
    fn select(&mut self, rom: RomId) -> Result<BusReturn>;
    fn directory_regular(&mut self) -> Result<BusReturn>;
    fn directory_alarm(&mut self) -> Result<BusReturn>;
    fn command(&mut self, cmd: BusCmd) -> Result<BusReturn> {
        match cmd {
            BusCmd::Reset => self.reset(),
            BusCmd::Description => self.description(),
            BusCmd::ReadWrite(data) => self.read_write(data),
            BusCmd::ResetReadWrite(data) => self.reset_read_write(data),
            BusCmd::Select(data) => self.select(data),
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
