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

use std::ffi ;
use std::str ;

pub use crate::error::{OwEResult};

/// ### OwMessageSend
/// message constructed to send to owserver
/// * 24 byte header (6 32-bit integers)
/// * contents (C-string)
/// * needs to be converted to network-endian format before sending
pub(super) struct OwMessageSend {
    pub(super) version: u32,
    pub(super) payload: u32,
    pub(super) mtype:   u32,
    pub(super) flags:   u32,
    pub(super) size:    u32,
    pub(super) offset:  u32,
    pub(super) content: Vec<u8>,
}

impl OwMessageSend {
    // Default owserver version (to owserver)
    const SENDVERSION: u32 = 0 ;

    // Maximum make_size of returned data (pretty arbitrary but matches C implementation)
    const DEFAULTSIZE: u32 = 65536 ;

    // Message types
    pub(super) const NOP:         u32 = 1 ;
    pub(super) const READ:        u32 = 2 ;
    pub(super) const WRITE:       u32 = 3 ;
    pub(super) const DIR:         u32 = 4 ;
    pub(super) const SIZE:        u32 = 5 ;
    pub(super) const PRESENT:     u32 = 6 ;
    pub(super) const DIRALL:      u32 = 7 ;
    pub(super) const GET:         u32 = 8 ;
    pub(super) const DIRALLSLASH: u32 = 9 ;
    pub(super) const GETSLASH:    u32 = 10 ;

    /// Create a nominal message (to be modified)
    pub(super) fn new(flag: u32, mtype: u32)-> OwMessageSend {
        OwMessageSend {
            version: OwMessageSend::SENDVERSION,
            payload: 0,
            mtype,
            flags:   flag,
            size:    OwMessageSend::DEFAULTSIZE,
            offset:  0,
            content: [].to_vec(),
        }
    }

    /// Name the message types
    pub(super) fn message_name( mtype: u32 ) -> &'static str {
        match mtype {
            OwMessageSend::NOP => "NOP",
            OwMessageSend::READ => "READ",
            OwMessageSend::WRITE => "WRITE",
            OwMessageSend::DIR => "DIR",
            OwMessageSend::SIZE => "SIZE",
            OwMessageSend::PRESENT => "PRESENT",
            OwMessageSend::DIRALL => "DIRALL",
            OwMessageSend::GET => "GET",
            OwMessageSend::DIRALLSLASH => "DIRALLSLASH",
            OwMessageSend::GETSLASH => "GETSLASH",
            _ => "UNKNOWN",
        }
    }

    /// first element of content and update payload length
    /// * should be null ended string or nothing
    pub(super) fn add_path( &mut self, path: &str ) -> OwEResult<()> {
        // Add nul-terminated path (and includes null in payload size)
        let s = ffi::CString::new(path) ? ;
        self.content = s.as_bytes().to_vec() ;
        self.payload = self.content.len() as u32 ;
        Ok(())
    }
    
    /// Add a second field
    /// * used for WRITE messages
    /// * not null ended
    /// * payload includes both
    /// * size is just this field's 
    pub(super) fn add_data( &mut self, data: &[u8] ) {
        // Add data after path without nul
        self.content.extend_from_slice(data) ;
        self.size = data.len() as u32 ;
        self.payload += self.size ;
    }
}
