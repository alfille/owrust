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

use crate::bus_thread::{BusCmd, BusQuery, BusReturn};
use anyhow::{Context, Result};
use std::ops::Deref;
use std::sync::mpsc;
use std::sync::{OnceLock, RwLock};

/// BusHandle is the external view of the bus
/// * holds the mpsc handle for sending data
pub struct BusHandle {
    pub tx: mpsc::Sender<BusQuery>,
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
}

pub struct BusList(Vec<BusHandle>);
impl Deref for BusList {
    type Target = Vec<BusHandle>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl IntoIterator for BusList {
    type Item = BusHandle;
    type IntoIter = std::vec::IntoIter<BusHandle>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

// Implement for a reference (borrows the struct)
impl<'a> IntoIterator for &'a BusList {
    type Item = &'a BusHandle;
    type IntoIter = std::slice::Iter<'a, BusHandle>;

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
    pub fn add(&mut self, bus: BusHandle) {
        self.0.push(bus)
    }
    pub fn list(&self) -> Vec<String> {
        self.iter()
            .map(|b| match b.send(BusCmd::Description).unwrap() {
                BusReturn::String(s) => s,
                _ => "unknown".to_string(),
            })
            .collect()
    }
    pub fn broadcast(cmd: BusCmd) -> Vec<Result<BusReturn>> {
        if let Ok(list) = global_buses().read() {
            list.iter().map(|bus| bus.send(cmd.clone())).collect()
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
        self.iter().map(f).collect()
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
    list.add(handle);
    Ok(())
}
