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

pub use crate::error::OwEResult;
use crate::message::print_message::PrintMessage;
use std::io::{Read, Write};
use std::net::TcpStream;

/// message with answers
/// * header (24 bytes) and content
/// * differs from query in **ret** value rather than **message type**
#[derive(Debug, PartialEq, Clone)]
pub(super) struct OwResponse {
    pub(super) version: u32,
    pub(super) payload: i32,
    pub(super) ret: i32,
    pub(super) flags: u32,
    pub(super) size: u32,
    pub(super) offset: u32,
    pub(super) content: Vec<u8>,
}
impl OwResponse {
    #[allow(unused)]
    pub(super) fn new(flags: u32) -> Self {
        OwResponse {
            version: 1,
            payload: 0,
            ret: 0,
            flags,
            size: 0,
            offset: 0,
            content: [].to_vec(),
        }
    }

    /// ### get_plus_ping
    /// Get a RESPONSE message from the network and parse it:
    /// * read header ( 6 words), translated from network order
    /// * read payload
    /// * include pings
    pub fn get_plus_ping(stream: &mut TcpStream) -> OwEResult<OwResponse> {
        static HSIZE: usize = 24;
        let mut buffer: [u8; HSIZE] = [0; HSIZE];

        // Take first 24 bytes of buffer to fill header
        stream.read_exact(&mut buffer)?;
        let mut rcv = OwResponse {
            version: u32::from_be_bytes(buffer[0..4].try_into().unwrap()),
            payload: i32::from_be_bytes(buffer[4..8].try_into().unwrap()),
            ret: u32::from_be_bytes(buffer[8..12].try_into().unwrap()) as i32,
            flags: u32::from_be_bytes(buffer[12..16].try_into().unwrap()),
            size: u32::from_be_bytes(buffer[16..20].try_into().unwrap()),
            offset: u32::from_be_bytes(buffer[20..24].try_into().unwrap()),
            content: [].to_vec(),
        };

        // read payload
        if rcv.payload > 0 {
            // create Vec with just the right size (based on payload)
            rcv.content = Vec::with_capacity(rcv.payload as usize);
            rcv.content.resize(rcv.payload as usize, 0);

            stream.read_exact(&mut rcv.content)?;
        }

        Ok(rcv)
    }

    /// ### get
    /// Get a RESPONSE message from the network and parse it:
    /// * read header ( 6 words), translated from network order
    /// * read payload
    /// * ignore pings
    pub fn get(stream: &mut TcpStream) -> OwEResult<OwResponse> {
        loop {
            let rcv = Self::get_plus_ping(stream)?;
            if rcv.payload >= 0 {
                // non-ping
                return Ok(rcv);
            }
        }
    }

    /// ### send
    /// * Send RESPONSE message to an owserver
    /// * Converts header to network order
    /// * includes payload
    pub(super) fn send(&mut self, stream: &mut TcpStream) -> OwEResult<()> {
        let mut msg: Vec<u8> = [
            self.version,
            self.payload as u32,
            self.ret as u32,
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

impl PrintMessage for OwResponse {
    fn version(&self) -> u32 {
        self.version
    }
    fn ret(&self) -> i32 {
        self.ret
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
    fn line_2( &self ) -> String {
		self.return_line_2()
	}
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_blank_response() {
        let resp = OwResponse::new(0x10101010 as u32);
        let desc = resp.print_all("Test Response").join("\n").to_string();
        assert_eq!( desc, "Test Response  Version: 1\nUNKNOWN message number 0\nFlags: C psi f.i   safe   \nPayload:0 Size:0 Offset:0\n".to_string() );
    }
}
