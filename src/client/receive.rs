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

use std::net::TcpStream ;
use std::io::{Read,Write} ;
pub use crate::error::{OwError,OwEResult};


/// message received back from owserver
/// * header (24 bytes) and content
pub(super) struct OwMessageReceive {
    pub(super) version: u32,
    pub(super) payload: u32,
    pub(super) ret:     i32,
    pub(super) flags:   u32,
    pub(super) size:    u32,
    pub(super) offset:  u32,
    pub(super) content: Vec<u8>,
}
impl OwMessageReceive {
    const HSIZE: usize = 24 ;
    /// Take first 24 bytes of buffer to fill header
    pub(super) fn new( buffer: [u8;OwMessageReceive::HSIZE] ) -> Self {
        OwMessageReceive {          
            version: u32::from_be_bytes(buffer[ 0.. 4].try_into().unwrap()),
            payload: u32::from_be_bytes(buffer[ 4.. 8].try_into().unwrap()),
            ret:     u32::from_be_bytes(buffer[ 8..12].try_into().unwrap()) as i32,
            flags:   u32::from_be_bytes(buffer[12..16].try_into().unwrap()),
            size:    u32::from_be_bytes(buffer[16..20].try_into().unwrap()),
            offset:  u32::from_be_bytes(buffer[20..24].try_into().unwrap()),
            content: [].to_vec(),
        }
    }
    pub(super) fn tell( &self) {
        eprintln!( "ver {:X}, pay {}, ret {}, flg {:X}, siz {}, off {}",self.version,self.payload,self.ret,self.flags,self.size,self.offset);
    }
    
    pub fn get_packet( stream: &mut TcpStream ) -> OwEResult<OwMessageReceive> {
        // get a single non-ping message.
        // May need multiple for directories
        static HSIZE: usize = 24 ;
        let mut buffer: [u8; HSIZE ] = [ 0 ; HSIZE ];
                
        loop {
            stream.read_exact( &mut buffer ) ? ;
            let mut rcv = OwMessageReceive::new(buffer);
            
            if (rcv.payload as i32) < 0 {
                // ping
                continue ;
            }
            if rcv.payload > 0 {
                // create Vec with just the right size (based on payload)
                rcv.content = Vec::with_capacity(rcv.payload as usize) ;
                rcv.content.resize(rcv.payload as usize,0);
                
                stream.read_exact(&mut rcv.content ) ? ;
            }
            return Ok(rcv) ;
        }
    }
}
