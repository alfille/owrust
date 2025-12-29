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
//! The main struct is OwMessage which holds all the configuration information.
//! Typically it is populated by the command line or configuration files
//!
//! ## EXAMPLES
//! ```
//! use owrust ; // basic library
//! use owrust::parse_args::{Parser,OwLib} ; // configure from command line, file or OsString
//!
//! let mut owserver = owrust::new() ; // create an OwMessage struct
//! let prog = OwLib ;
//!   // configure from command line and get 1-wire paths
//! let paths = prog.command_line( &mut owserver ) ;
//!   // Call any of the OwMessage functions like dir, read, write,...
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

pub use crate::error::{OwEResult, OwError};

// for Token management
use crate::message::Token;

/// ### OwQuery
/// message constructed to ask an owserver
/// * 24 byte header (6 32-bit integers)
/// * contents (C-string)
/// * needs to be converted to network-endian format before sending
/// * Add our token
#[derive(Debug, PartialEq, Clone)]
pub(super) struct OwQuery {
    pub(super) version: u32,
    pub(super) payload: i32,
    pub(super) mtype: u32,
    pub(super) flags: u32,
    pub(super) size: u32,
    pub(super) offset: u32,
    pub(super) content: Vec<u8>,
    pub(super) tokenlist: Vec<Token>,
}

impl OwQuery {
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
        token: Token,
    ) -> OwEResult<OwQuery> {
        let mut msg = OwQuery {
            version: OwQuery::SENDVERSION,
            payload: 0,
            mtype,
            flags: flag,
            size: OwQuery::DEFAULTSIZE,
            offset: 0,
            content: [].to_vec(),
            tokenlist: [].to_vec(),
        };
        if let Some(p) = path {
            msg.add_path(p)?;
        }
        if let Some(v) = value {
            msg.add_data(v);
        }
        msg.add_token(token);
        Ok(msg)
    }

    /// first element of content and update payload length
    /// * should be null ended string or nothing
    fn add_path(&mut self, path: &str) -> OwEResult<()> {
        // Add nul-terminated path (and includes null in payload size)
        let s = ffi::CString::new(path)?;
        self.content = s.as_bytes().to_vec();
        self.payload = self.content.len() as i32;
        Ok(())
    }

    /// Add a second field
    /// * used for WRITE messages
    /// * not null ended
    /// * payload includes both
    /// * size is just this field's length
    fn add_data(&mut self, data: &[u8]) {
        // Add data after path without nul
        self.content.extend_from_slice(data);
        self.size = data.len() as u32;
        self.payload += self.size as i32;
    }

    /// ### get_plus_ping
    /// Get a QUERY message from the network and parse it:
    /// * read header ( 6 words), translated from network order
    /// * read payload
    /// * read tokens
    /// * check for our token on list (==loop)
    /// * DO NOT ignore pings
    pub fn get_plus_ping(stream: &mut TcpStream, token: Token) -> OwEResult<OwQuery> {
        // get a single non-ping message.
        // May need multiple for directories
        static HSIZE: usize = 24;
        let mut buffer: [u8; HSIZE] = [0; HSIZE];

        stream.read_exact(&mut buffer)?;
        let mut rcv = OwQuery {
            version: u32::from_be_bytes(buffer[0..4].try_into().unwrap()),
            payload: i32::from_be_bytes(buffer[4..8].try_into().unwrap()),
            mtype: u32::from_be_bytes(buffer[8..12].try_into().unwrap()),
            flags: u32::from_be_bytes(buffer[12..16].try_into().unwrap()),
            size: u32::from_be_bytes(buffer[16..20].try_into().unwrap()),
            offset: u32::from_be_bytes(buffer[20..24].try_into().unwrap()),
            content: [].to_vec(),
            tokenlist: [].to_vec(),
        };

        // read payload
        if rcv.payload > 0 {
            // create Vec with just the right size (based on payload)
            rcv.content = Vec::with_capacity(rcv.payload as usize);
            rcv.content.resize(rcv.payload as usize, 0);

            stream.read_exact(&mut rcv.content)?;
        }

        // read tokens
        if (rcv.version & crate::message::SERVERMESSAGE) == crate::message::SERVERMESSAGE {
            let toks = rcv.version & crate::message::SERVERTOKENS;
            for _ in 0..toks {
                let mut tok: Token = [0u8; 16];
                stream.read_exact(&mut tok)?;
                rcv.tokenlist.push(tok)
            }
        }

        // test token
        if rcv.tokenlist.contains(&token) {
            return Err(OwError::General("Loop in owserver topology".to_string()));
        }

        // Add our token
        rcv.add_token(token);

        Ok(rcv)
    }

    /// ### get
    /// Get a QUERY message from the network and parse it:
    /// * read header ( 6 words), translated from network order
    /// * read payload
    /// * read tokens
    /// * check for our token on list (==loop)
    /// * ignore pings
    pub fn get(stream: &mut TcpStream, token: Token) -> OwEResult<OwQuery> {
        // get a single non-ping message.
        // May need multiple for directories
        loop {
            let rcv = Self::get_plus_ping(stream, token)?;
            if rcv.payload >= 0 {
                return Ok(rcv);
            }
        }
    }

    /// ### send
    /// * Send QUERY message to an owserver
    /// * Converts header to network order
    /// * includes payload
    /// * includes tokens
    /// * Will include tokens when available
    /// * own token included
    pub(super) fn send(&mut self, stream: &mut TcpStream) -> OwEResult<()> {
        let mut msg: Vec<u8> = [
            self.version,
            self.payload as u32,
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
        // use bytemuck to reinterpret sequential lumps of bytes to sequential bytes
        msg.extend_from_slice(bytemuck::cast_slice(&self.tokenlist));

        // Write to network
        stream.write_all(&msg)?;
        Ok(())
    }
    pub fn add_token(&mut self, token: Token) {
        let toks = match self.version & crate::message::SERVERMESSAGE {
            crate::message::SERVERMESSAGE => self.version & crate::message::SERVERTOKENS,
            _ => 0,
        };
        self.version = crate::message::SERVERMESSAGE | (toks + 1);
        self.tokenlist.push(token);
    }
}

impl crate::message::response::PrintMessage for OwQuery {
    fn version(&self) -> u32 {
        self.version
    }
    fn mtype(&self) -> u32 {
        self.mtype
    }
    fn offset(&self) -> u32 {
        self.offset
    }
    fn payload(&self) -> i32 {
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
#[cfg(test)]
mod tests {
    use super::*;
    use crate::message::response::PrintMessage;

    #[test]
    fn test_blank_query() {
        let query =
            OwQuery::new(0x10101010 as u32, OwQuery::READ, Some("/"), None, [0u8; 16]).unwrap();
        let desc = query.print_all("Test Query").join("\n").to_string();
        assert_eq!( desc, "Test Query  Version: 10001 tokens=1\nReturn code = 2\nFlags: C psi f.i   safe   \nPayload:1 Size:65536 Offset:0".to_string() );
    }
}
