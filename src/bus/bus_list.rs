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

use crate::bus_spawn::BusQuery;
use crate::bus_thread::{BusCmd, BusReturn};
use anyhow::{Context, Result};
use std::ops::Deref;
use std::sync::mpsc;
use std::sync::{OnceLock, RwLock};
use std::thread::JoinHandle;

/// BusHandle is the external view of the bus
/// * holds the mpsc handle for sending data
pub struct BusHandle {
    pub tx: mpsc::Sender<BusQuery>,
    pub handle: Option<JoinHandle<()>>,
}
impl BusHandle {
    pub fn send(&self, cmd: BusCmd) -> Result<BusReturn> {
        let (my_tx, my_rx) = mpsc::channel();
        let query = BusQuery::new(cmd, my_tx);
        self.tx
            .clone()
            .send(query)
            .context("Unable to clone bus channel")?;
        Ok(my_rx.recv()?)
    }
    pub fn close(&mut self) -> Result<()> {
        if let Some(handle) = self.handle.take() {
            let _ = self.send(BusCmd::Close);
            match handle.join() {
                Ok(_) => (),
                Err(_) => {
                    return Err(anyhow::anyhow!("Thread join problem"));
                }
            }
        }
        Ok(())
    }
}

pub struct BusList(Vec<RwLock<BusHandle>>);
impl Deref for BusList {
    type Target = Vec<RwLock<BusHandle>>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl IntoIterator for BusList {
    type Item = RwLock<BusHandle>;
    type IntoIter = std::vec::IntoIter<RwLock<BusHandle>>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

// Implement for a reference (borrows the struct)
impl<'a> IntoIterator for &'a BusList {
    type Item = &'a RwLock<BusHandle>;
    type IntoIter = std::slice::Iter<'a, RwLock<BusHandle>>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.iter()
    }
}
impl Default for BusList {
    fn default() -> Self {
        Self::new()
    }
}

impl BusList {
    pub fn new() -> Self {
        Self(Vec::new())
    }
    pub fn add(&mut self, bus: RwLock<BusHandle>) {
        self.0.push(bus)
    }
    pub fn list(&self) -> Vec<String> {
        self.iter()
            .map(|b| {
                match b
                    .read()
                    .expect("bus read error")
                    .send(BusCmd::Description)
                    .unwrap()
                {
                    BusReturn::String(s) => s,
                    _ => "unknown".to_string(),
                }
            })
            .collect()
    }
    pub fn broadcast(cmd: BusCmd) -> Vec<Result<BusReturn>> {
        if let Ok(list) = global_buses().read() {
            list.iter()
                .map(|bus| bus.read().expect("Bus read error").send(cmd.clone()))
                .collect()
        } else {
            vec![]
        }
    }
    /// Executes a generic function/closure on every bus in the list
    /// returns a Vector of the results
    pub fn for_each_bus<F, T>(&self, f: F) -> Vec<T>
    where
        F: Fn(&BusHandle) -> T,
    {
        self.iter()
            .map(|h| f(&h.read().expect("bus handle problem")))
            .collect()
    }
}

/// The global registry of all 1-wire buses
pub static BUSES: OnceLock<RwLock<BusList>> = OnceLock::new();

/// Helper to initialize or get the global bus list
pub fn global_buses() -> &'static RwLock<BusList> {
    BUSES.get_or_init(|| RwLock::new(BusList::new()))
}

pub fn register_bus(handle: BusHandle) -> Result<()> {
    let mut list = global_buses()
        .write()
        .map_err(|_| anyhow::anyhow!("Poisoned lock"))?;
    list.add(RwLock::new(handle));
    Ok(())
}
