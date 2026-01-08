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

pub enum BusCmd {
    Reset,
    Status,
    Description,
    Write(Vec<u8>),
    RWrite(Vec<u8>),
}

pub enum BusReturn {
    Bad,
    Bool(bool),
    Bytes(Vec<u8>),
    String(String),
}

///pub trait BusThread: Send + Sync + 'static {
pub trait BusThread {
    /// Returns the presence pulse (true if any slaves)
    fn reset(&mut self) -> Result<BusReturn>;
    fn status(&self) -> Result<BusReturn>;
    fn description(&self) -> Result<BusReturn> {
        Ok(BusReturn::String("Unspecified 1-wire bus".to_string()))
    }
    fn write(&mut self, data: Vec<u8>) -> Result<BusReturn>;
    fn reset_write(&mut self, data: Vec<u8>) -> Result<BusReturn> {
        self.reset()?;
        self.write(data)
    }
    fn command(&mut self, cmd: BusCmd) -> Result<BusReturn> {
        match cmd {
            BusCmd::Reset => self.reset(),
            BusCmd::Status => self.status(),
            BusCmd::Description => self.description(),
            BusCmd::Write(data) => self.write(data),
            BusCmd::RWrite(data) => self.reset_write(data),
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
		let bh = <DS9097E as BusThread>::spawn( "/dev/ttyS0".to_string(), DS9097E::new );
		let d = bh.send( BusCmd::Description ) ;
		assert!(d.is_ok()) 
    }
}
