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
}
