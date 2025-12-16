//! **owrust** Rust library interfaces with owserver to use 1-wire devices
//!
//! This is a tool in the 1-wire file system **OWFS**
//!
//! This library is the central part of **owrust** -- the _rust language_ OWFS programs
//! * **OWFS** [documentation](https://owfs.org) and [code](https://github.com/owfs/owfs)
//! * **owrust** [repository](https://github.com/alfille/owrust)
//!
//! ## PURPOSE
//! lib.rs is the library code that actually performs the **owserver protocol**.
//! Communication with **owserver** is over TCP/IP (network) using an efficient well-documented protocol.
//!
//! Supported operations are read, write, dir, present and size, with some variations
//!
//! The main struct is OwClient which holds all the configuration information.
//! Typically it is populated by the command line or configuration files
//!
//! ## EXAMPLES
//! ```
//! use owrust ; // basic library
//! use owrust::parse_args ; // configure from command line, file or OsString
//!
//! let mut owserver = owrust::new() ; // create an OwClient struct
//!   // configure from command line and get 1-wire paths
//! let paths = parse_args::command_line( &mut owserver ) ;
//!   // Call any of the OwClient functions like dir, read, write,...
//!   ```

// owrust project
// https://github.com/alfille/owrust
//
// This is a Rust version of my C owfs code for talking to 1-wire devices via owserver
// Basically owserver can talk to the physical devices, and provides network access via my "owserver protocol"
//
// MIT Licence
// {c} 2025 Paul H Alfille

use std::ffi;
use std::io::{Read, Write};
use std::net::TcpStream;
use std::str;

pub use crate::error::OwEResult;

/// ### OwMessageSend
/// message constructed to send to owserver
/// * 24 byte header (6 32-bit integers)
/// * contents (C-string)
/// * needs to be converted to network-endian format before sending
pub(super) struct OwMessageSend {
    pub(super) version: u32,
    pub(super) payload: u32,
    pub(super) mtype: u32,
    pub(super) flags: u32,
    pub(super) size: u32,
    pub(super) offset: u32,
    pub(super) content: Vec<u8>,
}

impl OwMessageSend {
    // Default owserver version (to owserver)
    const SENDVERSION: u32 = 0;

    // Maximum make_size of returned data (pretty arbitrary but matches C implementation)
    const DEFAULTSIZE: u32 = 65536;

    // Message types
    pub const NOP: u32 = 1;
    pub const READ: u32 = 2;
    pub const WRITE: u32 = 3;
    pub const DIR: u32 = 4;
    pub const SIZE: u32 = 5;
    pub const PRESENT: u32 = 6;
    pub const DIRALL: u32 = 7;
    pub const GET: u32 = 8;
    pub const DIRALLSLASH: u32 = 9;
    pub const GETSLASH: u32 = 10;

    /// Create a nominal message (to be modified)
    pub(super) fn new(
        flag: u32,
        mtype: u32,
        path: Option<&str>,
        value: Option<&[u8]>,
    ) -> OwEResult<OwMessageSend> {
        let mut msg = OwMessageSend {
            version: OwMessageSend::SENDVERSION,
            payload: 0,
            mtype,
            flags: flag,
            size: OwMessageSend::DEFAULTSIZE,
            offset: 0,
            content: [].to_vec(),
        };
        if let Some(p) = path {
            msg.add_path(p)?;
        }
        if let Some(v) = value {
            msg.add_data(v);
        }
        Ok(msg)
    }
    
//    pub(super) relay( crate::client::OwMessageReceive 

    /// first element of content and update payload length
    /// * should be null ended string or nothing
    pub(super) fn add_path(&mut self, path: &str) -> OwEResult<()> {
        // Add nul-terminated path (and includes null in payload size)
        let s = ffi::CString::new(path)?;
        self.content = s.as_bytes().to_vec();
        self.payload = self.content.len() as u32;
        Ok(())
    }

    /// Add a second field
    /// * used for WRITE messages
    /// * not null ended
    /// * payload includes both
    /// * size is just this field's
    pub(super) fn add_data(&mut self, data: &[u8]) {
        // Add data after path without nul
        self.content.extend_from_slice(data);
        self.size = data.len() as u32;
        self.payload += self.size;
    }

    /// ### send
    /// * Send rcv_message to owserver
    /// * Converts header to network order
    /// * includes payload
    /// * Will include tokens when available
    pub(super) fn send(&mut self, stream: &mut TcpStream) -> OwEResult<()> {
        let mut msg: Vec<u8> = [
            self.version,
            self.payload,
            self.mtype,
            self.flags,
            self.size,
            self.offset,
        ]
        .iter()
        .flat_map(|&u| u.to_be_bytes())
        .collect();
        if self.payload > 0 {
            msg.extend_from_slice(&self.content);
        }

        // Write to network
        stream.write_all(&msg)?;
        Ok(())
    }
}

impl crate::client::receive::PrintMessage for OwMessageSend {
    fn version(&self) -> u32 {
        self.version
    }
    fn mtype(&self) -> u32 {
        self.mtype
    }
    fn offset(&self) -> u32 {
        self.offset
    }
    fn payload(&self) -> u32 {
        self.payload
    }
    fn size(&self) -> u32 {
        self.size
    }
    fn flags(&self) -> u32 {
        self.flags
    }
    fn content(&self) -> &Vec<u8> {
        &self.content
    }
    /*
    fn tokenlist( &self ) -> u32 {
        self.tokenlist
    }
    */
    fn line2(&self) -> String {
        self.alt_line2()
    }
}
