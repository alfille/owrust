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

use anyhow::{Context, Result};
use std::sync::{mpsc};
use std::thread;

pub struct BusQuery {
    cmd: BusCmd,
    my_tx: mpsc::Sender<BusReturn>,
}

impl BusQuery {
    pub fn send(cmd: BusCmd, bus: &BusHandle) -> Result<BusReturn> {
        let (my_tx, my_rx) = mpsc::channel();
        let query = BusQuery { cmd, my_tx };
        bus.tx
            .clone()
            .send(query)
            .context("Unable to clone bus channel")?;
        Ok(my_rx.recv()?)
    }
}

/// BusHandle is the external view of the bus
/// * holds the mpsc handle for sending data
pub struct BusHandle {
    tx: mpsc::Sender<BusQuery>,
}

pub struct BusList {
    list: Vec<BusHandle>,
}
impl Default for BusList {
    fn default() -> Self {
        Self::new()
    }
}
impl BusList {
    pub fn new() -> Self {
        Self { list: Vec::new() }
    }
    pub fn add(&mut self, bus: BusHandle) {
        self.list.push(bus)
    }
}

pub enum BusCmd {
    Reset,
    Status,
    Write(Vec<u8>),
    RWrite(Vec<u8>),
}

pub enum BusReturn {
    Bad,
    Bool(bool),
    Bytes(Vec<u8>),
}

///pub trait BusThread: Send + Sync + 'static {
pub trait BusThread {
    /// Returns the presence pulse (true if any slaves)
    fn reset(&mut self) -> Result<BusReturn>;
    fn status(&self) -> Result<BusReturn>;
    fn write(&mut self, data: Vec<u8>) -> Result<BusReturn>;
    fn reset_write(&mut self, data: Vec<u8>) -> Result<BusReturn> {
        self.reset()?;
        self.write(data)
    }
    fn command(&mut self, cmd: BusCmd) -> Result<BusReturn> {
        match cmd {
            BusCmd::Reset => self.reset(),
            BusCmd::Status => self.status(),
            BusCmd::Write(data) => self.write(data),
            BusCmd::RWrite(data) => self.reset_write(data),
        }
    }
    /// create the bus thread
    /// * Works with different typoes of buses
    /// * actual bus structure is created in thread
    /// * External BusHandle us just the address
    /// * Uses a factory patern to create the internal bus device
    /// Example:
    /// ```
    /// use owrust::bus_list::BusThread;
    /// use owrust::ds9097e::DS9097E ; 
    /// let _ = <DS9097E as BusThread>::spawn( "/dev/ttyS0".to_string(), |p| { DS9097E::new(p) } );
    /// ```
    fn spawn<T, F>(path: String, factory: F) -> BusHandle
    where
        T : BusThread + Send + 'static,
        F: FnOnce(String) -> Result<T> + Send + 'static,
    {
        let (tx, rx) = mpsc::channel::<BusQuery>();
        thread::spawn(move || {
			let mut bus = match factory(path) {
				Ok(b) => b,
				Err(e) => {
					eprintln!("Could not create bus. {}",e);
					return ;
				},
			};
            while let Ok(req) = rx.recv() {
                let result = bus.command(req.cmd).unwrap_or(BusReturn::Bad);
                let _ = req.my_tx.send(result);
            }
        });
        BusHandle { tx }
    }
}
