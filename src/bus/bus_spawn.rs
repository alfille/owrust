//! ### Bus Spawn trait
//! creates bus in separate thread

// owrust project
// https://github.com/alfille/owrust
//
// This is a Rust version of my C owfs code for talking to 1-wire devices via owserver
// Basically owserver can talk to the physical devices, and provides network access via my "owserver protocol"
//
// MIT Licence
// {c} 2025 Paul H Alfille

use crate::bus_list::BusHandle;
use crate::bus_thread::{BusCmd, BusReturn, BusThread};
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

///pub trait BusSpawn {
pub trait BusSpawn {
    /// create the bus thread
    /// * Works with different typoes of buses
    /// * actual bus structure is created in thread
    /// * External BusHandle is just the address
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
