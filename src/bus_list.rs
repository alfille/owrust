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

use anyhow::{Result, Context};
use std::sync::mpsc ;
use std::thread ;
use crate::bus_talk::{BusCmd,BusReturn};

pub struct BusQuery {
	cmd: BusCmd,
	my_tx: mpsc::Sender<BusReturn>,
}

impl BusQuery {
	pub fn send( cmd: BusCmd, bus: Bus ) -> Result<BusReturn> {
        let (my_tx,my_rx) = mpsc::channel() ;
		let query = BusQuery {
			cmd,
			my_tx,
		} ;
		bus.tx.clone().send( query ).context("Unable to clone bus channel") ? ;
		Ok(my_rx.recv()?)
	}
}

pub struct Bus {
	tx: mpsc::Sender<BusQuery>,
	rx: mpsc::Receiver<BusCmd>,
}

pub struct BusList {
	list: Vec<Bus>,
}
impl Default for BusList {
	fn default() -> Self {
		Self::new()
	}
}
impl BusList {
	pub fn new() -> Self {
		Self {
			list: Vec::new(),
		}
	}
	pub fn add( &mut self, bus: Bus ) {
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
    Bool(bool),
    Bytes(Vec<u8>),
}

pub trait BusTalk {
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
	fn spawn( &self ) -> Result<Bus> {
		let ( rx,tx ) = mpsc::channel() ;
		thread::spawn( move || {
			while let Ok(req) = rx.recv() {
				let result = req.command( req.cmd ).context("Failed to process this request") ?;
				let _ = req.my_tx.send( result ) ;
			}
		});
		Ok( Bus{
			tx,
			rx,
		} )
	}		
}
